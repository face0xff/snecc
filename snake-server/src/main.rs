// Squelette de base du serveur TCP inspiré de https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
// Threads, mutex, arc: aidé de https://doc.rust-lang.org/book/ch16-03-shared-state.html

extern crate rand;

use rand::seq::SliceRandom;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::env;
use std::process::exit;

mod game_serv;
use game_serv::*;

mod protocol;

/// Taille de la map (carrée) en blocs
const MAP_SIZE: u16 = 64;

/// Taille des blocs en pixels
const BLOCK_SIZE: u16 = 10;

/// Fréquence initiale des joueurs
const INITIAL_SPEED: u8 = 1;

/// Période initiale (envoi de frames)
const INITIAL_PERIOD: time::Duration = time::Duration::from_millis(50);

/// Temps d'attente entre deux cycles de jeu en millisecondes
const GAME_SLEEP: u64 = 1;

/// Temps de timeout pendant un read en millisecondes
const READ_TO: u64 = 1;

/// Temps entre deux réceptions d'un mouvement du client
const INPUT_PERIOD: time::Duration = time::Duration::from_millis(10);

/// Durée du boost de vitesse (Mangue) en secondes
const BOOST_DURATION: time::Duration = time::Duration::from_millis(750);

/// Mode développeur "No Death" qui empêche les joueurs de mourir
const DEV_NO_DEATH : bool = false;

/// Gère un client.
///
/// # Arguments
///
/// * `stream` - Référence mutable vers le flux TCP
/// * `player_id` - Identifiant du joueur
/// * `n_players` - Nombre de joueurs dans la partie
/// * `game_mutex` - Arc de mutex vers la partie
/// * `snake_color` - Couleur du serpent (triplet RGB)
fn handle_client(stream: &mut TcpStream, player_id: u8, n_players: u8, game_mutex: Arc<Mutex<Game>>, snake_color: (u8, u8, u8)) -> () {
    // On envoie le player_id au client
    protocol::send_player_id(stream, player_id);
    println!("[{}] Sent player id", player_id);

    // On génère les données du nouveau joueur
    // Détermination de la position initiale du joueur
    let offset_x : u16 = 4;
    let offset_y : u16 = 4;
    let x : u16;
    let y : u16;
    match player_id {
        1 => {
            x = offset_x;
            y = offset_y;
        },
        2 => {
            x = offset_x;
            y = MAP_SIZE - offset_y;
        },
        3 => {
            x = MAP_SIZE - offset_x;
            y = offset_y;
        },
        4 => {
            x = MAP_SIZE - offset_x;
            y = MAP_SIZE - offset_y;
        }
        _ => panic!("Max number of players supported is 4 for now."),
    }

    // Ajout du nouveau joueur
    {
        let mut game = game_mutex.lock().unwrap();
        game.add_player(player_id, snake_color, x, y);
    }

    println!("[{}] Added player", player_id);

    // On attend que la partie soit prête à démarrer (i.e. que tous les threads aient reçu un joueur)
    loop {
        {
            let game = game_mutex.lock().unwrap();
            if game.players.len() == n_players as usize {
                break;
            }
        }
        // On dort un peu pour pas monopoliser le verrou
        thread::sleep(time::Duration::from_millis(200 as u64));
    }
    println!("[{}] Ready", player_id);

    // Les n clients sont prêts ; on peut envoyer les paramètres du jeu
    let mut players: Vec<(u8, (u8, u8, u8), i16, i16)> = vec![];
    {
        let mut game = game_mutex.lock().unwrap();
        for i in 0..(n_players as usize) {
            let player = game.players.get_mut(i).unwrap();
            players.push((player.id, player.color, player.head.x, player.head.y));
        }
    }
    protocol::send_game_params(stream, MAP_SIZE, BLOCK_SIZE, INITIAL_SPEED, n_players, players);
    
    println!("[{}] Sent game params", player_id);

    match stream.set_read_timeout(Some(time::Duration::from_millis(READ_TO))) {
        Err(e) => panic!("{}", e), // erreur ?
        Ok(_) => (),
    }

    let mut last_frame = time::Instant::now();
    let mut last_input = time::Instant::now();
    let mut last_boost = time::Instant::now();
    let mut alive: Vec<u8>;
    let snake_index: usize = {
        let game = game_mutex.lock().unwrap();
        game.get_player(player_id).unwrap()
    };

    // Boucle principale
    loop {

        // Réception d'un mouvement du client
        if last_input.elapsed() > INPUT_PERIOD {
            let player_move: Option<Move> = protocol::get_move_empty_buff(stream);
            match player_move {
                None => (),
                Some(mv) => {
                    last_input = time::Instant::now();
                    let mut game = game_mutex.lock().unwrap();
                    // On change l'intention du serpent du client
                    game.players[snake_index].change_intent(mv);
                },
            }
        }
        
        thread::sleep(time::Duration::from_millis(1));
        
        // Assez de temps s'est écoulé depuis la dernière frame
        if last_frame.elapsed() > INITIAL_PERIOD {
            {
                let mut game = game_mutex.lock().unwrap();
                
                match game.update_snake(snake_index) {
                    None => (),
                    Some(instant) => {
                        last_boost = instant;
                    }
                }

                if game.players[snake_index].boost {
                    let additional_updates: usize = 1;
                    for _i in 0..additional_updates {
                        game.update_snake(snake_index);
                    }
                }

                if last_boost.elapsed() > BOOST_DURATION {
                    game.players[snake_index].boost = false;
                }

                protocol::send_frame(stream, &game.food, &game.players_alive());
            }
            last_frame = time::Instant::now();
        }

        thread::sleep(time::Duration::from_millis(1));

        // Vérification du nombre de joueurs encore vivants
        {
            let mut game = game_mutex.lock().unwrap();
            alive = game.get_alive();
            if alive.len() < 2 {
                game.update_snake(snake_index);
                break;
            }
        }
        
        thread::sleep(time::Duration::from_millis(GAME_SLEEP));
    }

    thread::sleep(time::Duration::from_millis(GAME_SLEEP));

    // On envoie au moins une nouvelle frame pour signaler la ou les dernières morts
    {
        let game = game_mutex.lock().unwrap();
        protocol::send_frame(stream, &game.food, &game.players_alive());
    }

    if alive.len() == 0 {
        // Tout le monde est mort
        println!("Tout le monde est mort !");
    } else if alive.len() == 1 {
        // On a un gagnant
        println!("{} a gagné", alive[0]);
    }

    drop(stream);
    println!("Fermeture de la connection avec {}.", player_id);
}

fn main() {
    if env::args().len() != 3 {
        println!("Usage: ./snake-server port n_players");
        exit(0);
    }

    let args: Vec<String> = env::args().collect();
    let port: u16 = args[1].parse::<u16>().unwrap();
    let n_players: u8 = args[2].parse::<u8>().unwrap();

    if n_players < 2 || n_players > 4 {
        println!("Number of players should be between 2 and 4.");
        exit(0);
    }

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    println!("Listening on port {}", port);

    let mut player_id: u8;
    loop {
        // Permutation aléatoire des couleurs
        let mut snake_colors: Vec<(u8, u8, u8)> = vec![
            (0x4C, 0x3B, 0xE3),
            (0xDA, 0xAD, 0xFF),
            (0xF6, 0x83, 0x03),
            (0xF7, 0x49, 0x80),
            (0x9A, 0xF4, 0x96),
            (0x91, 0x67, 0x9D),
            (0xE1, 0x1C, 0x2F),
            (0x97, 0x99, 0x13),
        ];
        snake_colors.shuffle(&mut rand::thread_rng());

        let game_mutex = Arc::new(Mutex::new(Game::new(MAP_SIZE, INITIAL_SPEED)));
        let mut handles: Vec<std::thread::JoinHandle<()>> = vec![];
        player_id = 0;

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    player_id += 1;
                    println!("New connection: {}", stream.peer_addr().unwrap());

                    let snake_color = snake_colors[player_id as usize];

                    let game_mutex = Arc::clone(&game_mutex);
                    let handle = thread::spawn(move || handle_client(&mut stream, player_id, n_players, game_mutex, snake_color));
                    handles.push(handle);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
            if player_id >= n_players {
                // Les threads des N joueurs sont en cours ; on attend qu'ils finissent leur partie.
                for handle in handles {
                    handle.join().unwrap();
                }
                drop(game_mutex);
                break;
            }
        }
    }
}
