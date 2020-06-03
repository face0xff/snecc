use std::net::TcpStream;
use std::io::{Write, Read};
use std::ops::Shr;

use crate::Snake;
use crate::SnakeNode;
use crate::Food;
use crate::Move;

/// Différents types de messages
#[derive(PartialEq, Debug)]
enum Msg {
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

/// Transforme un entier en un vecteur d'octets (little-endian)
/// 
/// # Arguments
/// 
/// `n` - Entier 32 bits à transformer
/// `sz` - Taille en octets du vecteur de sortie
pub fn split_bytes(mut n: u32, sz: usize) -> Vec<u8> {
    let mut out = vec![];
    for _i in 0..sz {
        out.push((n % 256) as u8);
        n /= 256;
    }
    out
}

/// Transforme un entier en un vecteur d'octets (little-endian)
/// 
/// # Arguments
/// 
/// `data` - Entier 16 bits signé à transformer
/// `n` - Taille en octets du vecteur de sortie
fn vec_as_n_bytes(data: i16, n: usize) -> Vec<u8> {
    let mut v: Vec<u8> = vec![0; n];
    for i in 0..n {
        let octet_i = data.shr(i * 8) & 0b1111_1111;
        v[i] = octet_i as u8;
    }
    return v;
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

/// Envoie un message au serveur.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `msg` - Type de message à envoyer
/// `msg_data` - Slice vers les octets constituant le contenu du message à envoyer
fn send_data(stream: &mut TcpStream, msg: Msg, msg_data: &[u8]) -> () {
    let n = msg_data.len();
    // println!("Message len of {:?} = {}", msg, n);
    match stream.write(&[&[msg_to_id(msg), (n % 256) as u8, (n / 256) as u8], msg_data].concat()) {
        Err (e) => panic!("Erreur send_data: {}", e),
        Ok (_) => (),
    }
}

/// Reçoit un message du serveur. Toujours faire un peek avant.
/// Renvoie un triplet (type du message, taille, vecteur d'octets du contenu)
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
fn get_data(stream: &mut TcpStream) ->  (Msg, u32, Vec<u8>) {
    let header: &mut [u8] = &mut [0; 3];
    stream.read(header).unwrap();

    let msg_id: u32 = read_int_from_n_bytes(header, 0, 1);
    let msg_len: u32 = read_int_from_n_bytes(header, 1, 2);

    let mut buf = vec![0 as u8; msg_len as usize];
    stream.read_exact(&mut buf).unwrap();

    (id_to_msg(msg_id as u8), msg_len, buf)
}

/// Envoie l'identifiant du joueur.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `player_id` - Identifiant du joueur
pub fn send_player_id(stream: &mut TcpStream, player_id: u8) -> () {
    send_data(stream, Msg::PlayerId, &[player_id]);
}

/// Envoie les paramètres de la partie.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `map_size` - Taille de la map en blocs
/// `block_size` - Taille d'un bloc en pixels
/// `initial_speed` - Vitesse initiale
/// `n_players` - Nombre de joueurs
/// `players` - Vecteur des paramètres des joueurs
pub fn send_game_params(stream: &mut TcpStream, map_size: u16, block_size: u16, initial_speed: u8, n_players: u8, players: Vec<(u8, (u8, u8, u8), i16, i16)>) {
    let mut players_formatted: Vec<u8> = vec![];
    for i in 0..(n_players as usize) {
        players_formatted.push(players[i].0);
        players_formatted.push((players[i].1).0);
        players_formatted.push((players[i].1).1);
        players_formatted.push((players[i].1).2);
        let x0 = split_bytes(players[i].2 as u32, 2);
        players_formatted.push(x0[0]);
        players_formatted.push(x0[1]);
        let y0 = split_bytes(players[i].3 as u32, 2);
        players_formatted.push(y0[0]);
        players_formatted.push(y0[1]);
        println!("{} length of players_formatted", players_formatted.len());
    }

    send_data(
        stream,
        Msg::GameParams,
        &[
            split_bytes(map_size as u32, 2),
            split_bytes(block_size as u32, 2),
            vec![initial_speed],
            vec![n_players],
            players_formatted,
        ].concat()
    );
}

/// Reçoit un mouvement. Renvoie une option du mouvement.
/// Renvoie None en cas d'erreur de peek ou si `get_move` a renvoyé None.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
pub fn get_move_empty_buff(stream: &mut TcpStream) -> Option<Move> {
    let mut ret = None;
    while {
        match stream.peek(&mut[0; 3]) {
            Err(_) => false,
            Ok(n) => n > 0,
        }
    } {
        ret = get_move(stream);
    }
    return ret;
    
}

/// Reçoit un mouvement. Renvoie une option du mouvement.
/// Renvoie None s'il n'y a rien à lire.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
pub fn get_move(stream: &mut TcpStream) -> Option<Move> {
    match stream.peek(&mut [0; 3]) {
        Err(_) => None,
        Ok(n) => { 
            if n == 0 {
                // Rien à lire
                return None;
            }
            
            let (msg_id, msg_len, content): (Msg, u32, Vec<u8>) = get_data(stream);
            
            if msg_id != Msg::Move || msg_len != 1 {
                panic!("get_move: malformed message; id:{}; len:{}", msg_to_id(msg_id), msg_len);
            }
            
            match content[0] {
                1 => Some(Move::Up),
                2 => Some(Move::Down),
                3 => Some(Move::Left),
                4 => Some(Move::Right),
                _ => {
                    panic!("get_move: malformed move ({})", content[0]);
                }
            }
        }
    }
}

/// Envoie une frame au client.
/// 
/// # Arguments
/// 
/// `stream` - Référence mutable vers le flux TCP
/// `list_food` - Référence vers un vecteur de nourritures
/// `list_snake` - Référence vers un vecteur de références vers les serpents
pub fn send_frame(stream: &mut TcpStream, list_food: &Vec<Food>, list_snake: &Vec<&Snake>) -> () {
    send_data(
        stream,
        Msg::Frame, 
        &[
            food_to_bytes(list_food),
            snake_to_bytes(list_snake),
        ].concat()
    );
}

/// Transforme une liste de nourriture en vecteur d'octets pour le message
/// 
/// # Arguments
/// 
/// `list_food` - Référence vers un vecteur de nourritures
fn food_to_bytes(list_food: &Vec<Food>) -> Vec<u8> {
    let mut data: Vec<u8> = vec![];
    data.push(list_food.len() as u8);
    for food in list_food {
        let (f_id, x, y): (u8, i16, i16) = food.get_info_for_data_trs();
        data.push(f_id);
        data.append(&mut vec_as_n_bytes(x, 2));
        data.append(&mut vec_as_n_bytes(y, 2));
    }
    return data;
}

/// Transforme une liste de serpents en vecteur d'octets pour le message
/// 
/// # Arguments
/// 
/// `list_snake` - Référence vers un vecteur de références vers des serpents
fn snake_to_bytes(list_snake: &Vec<&Snake>) -> Vec<u8> {
    let mut data: Vec<u8> = vec![];
    data.push(list_snake.len() as u8);
    for snake in list_snake {
        data.push(snake.id);
        data.push(if snake.has_lost { 1 as u8 } else { 0 as u8 });
        data.push(snake.stomach);
        let len = snake.head.len();
        data.push(len as u8);
        
        let mut node: &SnakeNode = &snake.head;
        for i in 0..len {
            if i > 0 {
                node = node.get_next().unwrap();
            }
            data.append(&mut vec_as_n_bytes(node.x, 2));
            data.append(&mut vec_as_n_bytes(node.y, 2));
        }
    }
    return data;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FoodType;
    
    #[test]
    fn test_vec_as_n_bytes() {
        let data = 2;
        let v = vec_as_n_bytes(data, 2);
        assert_eq!(v, [2, 0]);

        let data = 9832;
        let v = vec_as_n_bytes(data, 1);
        assert_eq!(v, [0x68]);

        let data = 9832;
        let v = vec_as_n_bytes(data, 2);
        assert_eq!(v, [0x68, 0x26]);

        let data: i16 = -1;
        let v = vec_as_n_bytes(data, 2);
        assert_eq!(v, [0xff, 0xff]);
    }
    
    #[test]
    fn test_snake_to_bytes() {
        let sn = &Snake::new(1, (2, 3, 4), 10, 20, Move::Right, Move::Right);
        let snake_lst = vec![sn];
        assert_eq!(snake_to_bytes(&snake_lst), [1,1, 0, 10, 2, 10, 0 , 20, 0 ,10, 0, 20, 0]);
    }  
    #[test]
    fn test_snake_to_bytes2() {
        let sn = &Snake::new(1, (2, 3, 4), 10, 20, Move::Right, Move::Right);
        let sn2 = &Snake::new(2, (2, 3, 4), 30, 40, Move::Right, Move::Right);
        let snake_lst = vec![sn, sn2];
        assert_eq!(snake_to_bytes(&snake_lst), [2,1, 0, 10, 2, 10, 0 , 20, 0 ,10, 0, 20, 0, 2, 0, 10, 2, 30, 0 ,40, 0, 30, 0, 40, 0]);
    }  
    
    #[test]
    fn test_food_to_bytes() {
        let food_lst = vec![Food::new(10, 20, FoodType::Apple), Food::new(30, 40, FoodType::Apple)];
        assert_eq!(food_to_bytes(&food_lst), [2, 1, 10, 0, 20, 0, 1, 30, 0, 40, 0]);
    }
}
