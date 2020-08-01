extern crate piston_window;

use piston_window::*;
use piston_window::types::Color;

use std::net::TcpStream;
use std::{thread::sleep, time};
use std::env;
use std::process::exit;

mod draw;

mod game;
use game::*;

use crate::draw::*;

mod protocol;

/// Couleur du fond
const BLACK: Color = [0.2, 0.2, 0.2, 0.8];

/// Période entre deux réceptions d'input
const INPUT_PERIOD: time::Duration = time::Duration::from_millis(50);

/// Période de réception d'une frame
const UPDATE_PERIOD: time::Duration = time::Duration::from_millis(5);

/// Largeur en pixels du HUD à droite de l'écran
const HUD_WIDTH: u16 = 250;

/// Dimensions par défaut de la map (en blocs). Cette donnée doit être en phase avec le serveur.
const DEFAULT_GAME_DIMENSIONS: u16 = 64;

/// Taille en pixels par défaut d'un bloc. Cette donnée doit être en phase avec le serveur.
const DEFAULT_BLOCK_SIZE: u16 = 10;

/// Différents états d'un client
pub enum ClientState {
    Waiting,
    OnGoing,
    EndOfGame,
}

fn main() {
    if env::args().len() != 3 {
        println!("Usage: ./snake-client ip port");
        exit(0);
    }

    let args: Vec<String> = env::args().collect();
    let ip_addr: &String = &args[1];
    let port: u16 = args[2].parse::<u16>().unwrap();

    let window_size = DEFAULT_GAME_DIMENSIONS;
    let window: &mut PistonWindow = &mut WindowSettings::new("Snake", [(window_size * DEFAULT_BLOCK_SIZE + HUD_WIDTH) as u32, (window_size * DEFAULT_BLOCK_SIZE) as u32])
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    let mut retry: bool = true;

    while retry {
        let mut timeout: u8 = 0;
        println!("Attempting connection to back-end server...");

        loop {
            if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", ip_addr, port)) {
                println!("Connected to the server");
                retry = handle_connection(&mut stream, window);
                if retry {
                    let mut end_retry = false;
                    while let Some(event) = window.next() {
                        if end_retry {
                            break;
                        }
                        window.draw_2d(&event, |_c, g, _d| {
                            clear(BLACK, g);
                            end_retry = true;
                        });
                    }
                }
                break;
            } else {
                println!("Failed, trying again...");
                sleep(time::Duration::from_millis(3000 as u64));
                timeout += 1;
                if timeout > 5 {
                    println!("Couldn't connect.");
                    break;
                }
            }
        }
    }

    println!("Goodbye.");
}

/// Gère une nouvelle connexion. Retourne vrai si le joueur veut relancer la partie.
///
/// # Arguments
///
/// * `stream` - Référence mutable vers le flux TCP
/// * `window` - Référence mutable vers la fenêtre Piston
fn handle_connection(stream: &mut TcpStream, window: &mut PistonWindow) -> bool {
    let game: &mut Game = &mut Game::new(0, 0);
    let address: &str = &stream.peer_addr().unwrap().to_string();

    let assets = find_folder::Search::ParentsThenKids(2, 3).for_folder("assets").unwrap();
    let ref font = assets.join("FiraSans-Regular.ttf");
    let factory = window.factory.clone();
    let glyphs = &mut window.load_font(font).unwrap();
    
    // Récupération de l'identifiant du joueur
    let id: u8 = protocol::get_player_id(stream);
    println!("Id received: {}", id);

    match stream.set_read_timeout(Some(time::Duration::from_millis(50))) {
        Err(e) => panic!("{}", e), // erreur ?
        Ok(_) => (),
    }

    // En attente d'un ou plusieurs joueurs...
    println!("Waiting for opponent(s)...");
    let mut client_state: ClientState = ClientState::Waiting;
    
    let mut last_input = time::Instant::now();
    let mut last_update = time::Instant::now();
    
    let mut index : usize = 0; // Valeur temporaire

    let mut alive_assoc: Vec<(u8, bool)> = vec![];

    // Boucle principale, inspirée d'un projet déjà existant 
    // ainsi que la documentation Piston
    while let Some(event) = window.next() {
        match client_state  {
            ClientState::Waiting => {
                window.draw_2d(&event, |c, g, d| {
                    clear(BLACK, g);
                    draw_hud(&c, g, glyphs, game, &client_state, address, id, &vec![]);
                    glyphs.factory.encoder.flush(d);
                });
                if protocol::check_if_params(stream, game, id) {
                    println!("Received game params. Let's go!");
                    client_state = ClientState::OnGoing;
                    index = game.get_player_index(id).unwrap();
                }
            }

            ClientState::OnGoing => {
                if let Some(Button::Keyboard(key)) = event.press_args() {
                    game.key_pressed(id, key);
                }
                
                // Envoi périodique d'un mouvement
                if last_input.elapsed() > INPUT_PERIOD {
                    let snake : &mut Snake = game.players.get_mut(index).unwrap();
                    protocol::send_move(stream, &snake.moving);
                    game.can_send_move = false;
                    last_input = time::Instant::now();
                }

                // Réception périodique d'une frame du jeu
                if last_update.elapsed() > UPDATE_PERIOD {
                    match protocol::check_if_frame(stream, game) {
                        None => (),
                        Some(alive) => {
                            alive_assoc = alive.clone();
                            let n_alive: usize = alive.into_iter().filter(|&(_, dead)| !dead).count();
                            if n_alive <= std::cmp::min(1, (game.n_players - 1) as usize) {
                                client_state = ClientState::EndOfGame;
                            }
                            last_update = time::Instant::now();
                        }
                    }
                }

                window.draw_2d(&event, |c, g, d| {
                    clear(BLACK, g);
                    // Affichage du jeu
                    game.draw_game(&c, g, id);
                    draw::draw_hud(&c, g, glyphs, game, &client_state, address, id, &alive_assoc);
                    glyphs.factory.encoder.flush(d);

               });
            }

            ClientState::EndOfGame => {
                if let Some(Button::Keyboard(key)) = event.press_args() {
                    if key == Key::R || key == Key::Q || key == Key::Escape {
                        // if key == Key::R {
                        //     window.draw_2d(&event, |_c, g| {
                        //         clear(BLACK, g);
                        //     });
                        // }
                        return key == Key::R;
                    }
                }
                window.draw_2d(&event, |c, g, d| {
                    clear(BLACK, g);
                    
                    // Affichage du jeu
                    game.draw_game(&c, g, id);
                    draw::draw_hud(&c, g, glyphs, game, &client_state, address, id, &alive_assoc);
                    glyphs.factory.encoder.flush(d);

               });
            }
        } 
    }

    false
}
