use std::net::TcpStream;
use std::io::{Read, Write};
use crate::Game;
use crate::Snake;
use crate::SnakeNode;
use crate::Food;
use crate::Move;

/// Différents types de messages
#[derive(PartialEq)]
pub enum Msg {
    PlayerId,
    GameParams,
    GameStart,
    Frame,
    Move,
    UnknownId,
}

/// Renvoie un objet message à partir de son identifiant
fn id_to_msg(id: u8) -> Msg {
    match id {
        0 => Msg::PlayerId,
        1 => Msg::GameParams,
        2 => Msg::GameStart,
        3 => Msg::Frame,
        4 => Msg::Move,
        _ => Msg::UnknownId,
    }
}

/// Renvoie l'identifiant numérique associé à un message
fn msg_to_id(id: Msg) -> u8 {
    match id {
        Msg::PlayerId => 0,
        Msg::GameParams => 1,
        Msg::GameStart => 2,
        Msg::Frame => 3,
        Msg::Move => 4,
        Msg::UnknownId => {
            panic!("Unknown message ID");
        },
    }
}

/// Lit et renvoie un entier 32 bits à partir d'un tampon d'octets
/// 
/// # Arguments
/// 
/// `buf` - Slice vers le tampon d'octets
/// `index` - Indice à partir duquel lire l'entier
/// `n` - Nombre d'octets à lire
pub fn read_int_from_n_bytes(buf: &[u8], index: u32, n: u32) -> u32 {
    if (buf.len() as u32) < index + n {
        panic!("read_int_from_n_bytes: Asked to read further than actual buffer length");
    }
    
    let mut res: u32 = 0;
    for i in 0..n {
        res += (256 as u32).pow(i) * ((buf[(index + i) as usize]) as u32);
    }

    res
}

/// Reçoit un message du serveur.
/// Renvoie un triplet (type du message, taille, vecteur d'octets du contenu)
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
fn get_data(stream: &mut TcpStream) ->  (Msg, u32, Vec<u8>) {
    let header: &mut [u8] = &mut [0; 3];
    if stream.peek(header).unwrap() > 0 {
        stream.read(header).unwrap();
    }

    let msg_id: u8 = read_int_from_n_bytes(header, 0, 1) as u8;
    let msg_len: u16 = read_int_from_n_bytes(header, 1, 2) as u16;

    let mut buf = vec![0 as u8; msg_len as usize];
    stream.read_exact(&mut buf).unwrap();

    (id_to_msg(msg_id as u8), msg_len as u32, buf)
}

/// Envoie un message au serveur.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `msg` - Type de message à envoyer
/// `msg_data` - Slice vers les octets constituant le contenu du message à envoyer
fn send_data(stream: &mut TcpStream, msg: Msg, msg_data: &[u8]) -> () {
    let n = msg_data.len();
    match stream.write(&[&[msg_to_id(msg), (n % 256) as u8, (n / 256) as u8], msg_data].concat()) {
        Err (e) => panic!("Erreur send_data: {}", e),
        Ok (_) => (),
    }
}

/// Reçoit l'identifiant attribué au joueur.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
pub fn get_player_id(stream: &mut TcpStream) -> u8 {
    let (msg_id, msg_len, content): (Msg, u32, Vec<u8>) = get_data(stream);

    if msg_id != Msg::PlayerId || msg_len != 1 {
        panic!("get_player_id: malformed message; id:{}; len:{}", msg_to_id(msg_id), msg_len);
    }
    
    content[0]
}

/// Vérifie et renvoie si les paramètres du jeu ont été reçus.
/// Si oui, traite les paramètres reçus et modifie la partie en conséquence.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `game` - Référence mutable vers la partie
/// `client_player_id` - Identifiant du joueur
pub fn check_if_params(stream: &mut TcpStream, game: &mut Game, client_player_id: u8) -> bool {
    let head :&mut [u8] = &mut [0; 3];
    match stream.peek(head) {
        Err(_e) => false,
        Ok(n) => {
            if n > 0 {
                if head[0] == msg_to_id(Msg::GameParams) {
                    get_game_params(stream, game, client_player_id);
                    return true;
                } else {
                    return false;
                }
            } else {
                false
            }       
        },
    }
}

/// Reçoit et traite les paramètres du jeu.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `game` - Référence mutable vers la partie
/// `client_player_id` - Identifiant du joueur
pub fn get_game_params(stream: &mut TcpStream, game: &mut Game, client_player_id: u8) {
    let (msg_id, msg_len, content): (Msg, u32, Vec<u8>) = get_data(stream);
    
    if msg_id != Msg::GameParams {
        panic!("get_game_params: malformed message; id:{}; len:{}", msg_to_id(msg_id), msg_len);
    }

    let map_size: u16 = read_int_from_n_bytes(&content, 0, 2) as u16;
    let block_size: u16 = read_int_from_n_bytes(&content, 2, 2) as u16;
    // let initial_speed: u8 = read_int_from_n_bytes(&content, 4, 1) as u8;
    let n_players: u8 = read_int_from_n_bytes(&content, 5, 1) as u8;

    let mut player_params: Vec<(u8, u8, u8, u8, i16, i16)> = vec![];
    for i in 0..(n_players as u32) {
        let player_id: u8 = read_int_from_n_bytes(&content, 6 + 8 * i, 1) as u8;
        let player_red: u8 = read_int_from_n_bytes(&content, 6 + 8 * i + 1, 1) as u8;
        let player_green: u8 = read_int_from_n_bytes(&content, 6 + 8 * i + 2, 1) as u8;
        let player_blue: u8 = read_int_from_n_bytes(&content, 6 + 8 * i + 3, 1) as u8;
        let player_x0: i16 = read_int_from_n_bytes(&content, 6 + 8 * i + 4, 2) as i16;
        let player_y0: i16 = read_int_from_n_bytes(&content, 6 + 8 * i + 6, 2) as i16;
        player_params.push(
            (player_id, player_red, player_green, player_blue, player_x0, player_y0)
        );
        if player_id == client_player_id {
            game.set_border_color([player_red as f32/256.0, player_green as f32/256.0, player_blue as f32/256.0, 1.0]);
        }
    }

    game.dimensions = map_size;
    game.block_size = block_size;
    game.n_players = n_players;
    game.init_players(player_params);    
}

/// Envoie un mouvement au serveur.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `player_move` - Référence vers le mouvement
pub fn send_move(stream: &mut TcpStream, player_move: &Move) -> () {
    let move_byte: u8 = match player_move {
        Move::Up => 1,
        Move::Down => 2,
        Move::Left => 3,
        Move::Right => 4,
    };
    send_data(stream, Msg::Move, &[move_byte]);
}

/// Vérifie si une frame a été reçue.
/// Si oui, déconstruit la frame, la traite et renvoie une option de vecteur d'association codant les joueurs encore en vie.
/// Si non, renvoie *None*.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `game` - Référence mutable vers la partie
pub fn check_if_frame(stream: &mut TcpStream, game: &mut Game) -> Option<Vec<(u8, bool)>> {
    let head :&mut [u8] = &mut [0; 3];
    match stream.peek(head) {
        Err(_e) => (),
        Ok(n) => {
            if n > 0 {
                if head[0] == msg_to_id(Msg::Frame) {
                    let (msg, _len, data) = get_data(stream);
                    return Some(deconstruct_frame(msg, &data, game));
                }
            }        
        },
    }
    None
}

/// Déconstruit une frame à partir du contenu du message reçu par le serveur,
/// la traite et renvoie un vecteur d'association codant les joueurs encore en vie.
/// 
/// # Arguments
/// 
/// `msg` - Le type de message
/// `buf` - Slice vers le contenu du message
/// `game` - Référence mutable vers la partie
pub fn deconstruct_frame(msg: Msg, buf: &[u8], game: &mut Game) -> Vec<(u8, bool)> {
    if msg != Msg::Frame {
        panic!("deconstruct_frame: expected id {}, got {}", msg_to_id(Msg::Frame), msg_to_id(msg));
    }

    let mut index: usize = 0;
    let food_number: u8 = buf[index];
    index += 1;
    game.clear_food();
    
    for _i in 0..food_number {
        let food_type = buf[index];
        index += 1;
        let food_x = read_int_from_n_bytes(buf, index as u32, 2);
        index += 2;
        let food_y = read_int_from_n_bytes(buf, index as u32, 2);
        index += 2;
        game.add_food(food_x as i16, food_y as i16, Food::int_to_food_type(food_type));
    }
    
    let snake_number: u8 = buf[index];
    index += 1;
    
    let mut alive = vec![];

    for _i in 0..snake_number {
        let id = buf[index];
        index += 1;
        let has_lost = buf[index] == 1;
        index += 1;

        alive.push((id, has_lost));

        let food_in_stomach = buf[index];
        index += 1;
        let node_number = buf[index];
        index += 1;
        
        let snake: &mut Snake;
        match game.get_player_index(id) {
            Some(snake_index) => {
                snake = match game.players.get_mut(snake_index) {
                    None => panic!("Could not get_mut from players: Vec<Snake>"),
                    Some(snk) => snk,
                }
            },
            None => panic!("deconstruct_frame: Received an unknown ID: {}", id),
        }

        for i in 0..node_number {
            let node_x = read_int_from_n_bytes(buf, index as u32, 2);
            index += 2;
            let node_y = read_int_from_n_bytes(buf, index as u32, 2);
            index += 2;
            
            let node: SnakeNode = SnakeNode::new(node_x as i16, node_y as i16);
            if i > 0 {
                snake.add_node_as_tail(node);
            } else {
                snake.reset_snake_with_node(node);
            }
        }

        snake.has_lost = has_lost;
        snake.stomach = food_in_stomach;
    }

    alive
}
