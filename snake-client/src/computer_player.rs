use crate::Game;
use crate::Move;
use crate::Snake;
use rand::Rng;

fn random_direction() -> Move {
    let mut rng = rand::thread_rng();
    let move_rng = rng.gen_range(0, 4);
    if move_rng > 3 {
        Move::Up
    } else if move_rng > 2 {
        Move::Left
    } else if move_rng > 1 {
        Move::Down
    } else {
        Move::Right
    }
}

pub struct ComputerPlayer {
    player_id: u8,
}

impl ComputerPlayer {
    pub fn init(player_id: u8) -> Self {
        return ComputerPlayer { player_id };
    }
    pub fn play(&self, game: &Game) -> Move {
        // let my_snake_index = game.get_player_index(self.player_id);
        // let players: &Vec<Snake> = &game.players;
        return random_direction();
    }
}
