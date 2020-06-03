use std::ops::Deref;
use std::ops::DerefMut;
use std::cmp;
use std::time;
use rand::distributions::{Distribution, Uniform};

use crate::DEV_NO_DEATH;


/// Allongement du serpent par pomme mangée
const FOOD_BY_APPLE: u8 = 4;

/// Nombre de types de nourritures
const N_FOOD_TYPES: u64 = 2;

/// Nombre de nourritures maximale sur la carte
const MAX_FOOD: usize = 20;

/// Différents mouvements
#[derive(PartialEq, Debug)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

/// Différents types de nourriture
#[derive(Clone, PartialEq)]
pub enum FoodType {
    /// [1] Pomme
    Apple,
    /// [2] Mangue
    Mango,
}


/// Renvoie un objet FoodType à partir de l'identifiant numérique
///
/// # Arguments
///
/// * `food_type` - L'identifiant de la nourriture
pub fn food_id_to_type(food_type: u64) -> FoodType {
    match food_type {
        1 => FoodType::Apple,
        2 => FoodType::Mango,
        _ => {
            panic!("Unknown food type {}", food_type);
        }
    }
}

/// Différents types de cases
enum TileType {
    FoodTile(FoodType),
    SnakeTile(u8),
    Wall,
    Nothing,
}

/// Noeuds du serpent (liste chaînée)
#[derive(Debug, Clone, PartialEq)]
pub struct SnakeNode {
    pub x: i16,
    pub y: i16,
    pub next_node: Option<Box<SnakeNode>>,
}

/// Structure de serpent
pub struct Snake {
    /// Identifiant unique du serpent au sein de la partie
    pub id: u8,
    /// Couleur du serpent
    pub color: (u8, u8, u8),
    /// Tête du serpent (premier noeud de la liste chaînée)
    pub head: SnakeNode,
    /// Direction (vérifiée) du serpent
    pub direction: Move,
    /// Intention de mouvement du serpent
    /// *Avoir une différence entre direction et moving (intention)*
    /// *permet d'insérer un nouveau noeud derrière la tête du serpent*
    /// *lorsque celui-ci tourne.*
    pub moving: Move,
    /// Booléen codant si le joueur a perdu
    pub has_lost: bool,
    /// Nombre d'unités dans l'estomac du serpent
    pub stomach: u8,
    /// Actif durant le power-up de boost (Mangue)
    pub boost: bool,
}

/// Structure de nourriture
pub struct Food {
    x: i16,
    y: i16,
    food_type: FoodType,
}

/// Structure de partie
pub struct Game {
    /// Taille en blocs de la map
    dimensions: u16,
    /// Vecteur des nourritures présentes sur la map
    pub food: Vec<Food>,
    pub waiting: bool,
    pub period: f64,
    pub progress: f64,
    /// Vecteur des serpents joueurs
    pub players: Vec<Snake>,
}

/// Implémentation d'un noeud de serpent
impl SnakeNode {
    /// Renvoie un nouveau noeud sans successeur
    ///
    /// # Arguments
    ///
    /// * `x`, `y` - Coordonnées du noeud
    pub fn new(x: i16, y: i16) -> Self {
        SnakeNode {
            x,
            y,
            next_node: None,
        }
    }

    /// Renvoie si le noeud a un successeur
    fn has_next(&self) -> bool {
        match self.next_node {
            None => return false,
            Some(_) => return true,
        }
    }

    /// Renvoie la longueur de la chaîne de noeuds
    pub fn len(&self) -> i16 {
        let mut node: &Self = self;
        let mut i: i16 = 1;
        loop {
            match node.next_node {
                None => return i,
                Some (ref n) => {
                    i += 1;
                    node = n.deref();
                },
            }
        }
    }

    /// Renvoie une option du prochain noeud
    pub fn get_next(&self) -> Option<&Self> {
        match self.next_node {
            None => None,
            Some (ref n) => Some(n.deref())
        }
    }
   
}

/// Implémentation d'un serpent
impl Snake {    
    /// Renvoie un nouveau serpent
    ///
    /// # Arguments
    ///
    /// * `id` - Identifiant du joueur
    /// * `color` - Couleur du serpent (triplet RGB)
    /// * `x`, `y` - Coordonnées initiales du serpent
    /// * `initial_direction` - Direction initiale du serpent
    /// * `initial_moving` - Intention initiale de mouvement du serpent (devrait être égale à `initial_direction`)
    pub fn new(id: u8, color: (u8, u8, u8), x: i16, y: i16, initial_direction: Move, initial_moving: Move) -> Self {
        let mut head: SnakeNode = SnakeNode::new(x, y);
        let tail: SnakeNode = SnakeNode::new(x, y);
        head.next_node = Some(Box::new(tail));
        Snake {
            id,
            color,
            head,
            direction: initial_direction,
            moving: initial_moving,
            has_lost: false,
            stomach: 10,
            boost: false,
        }
    }

    /// Crée un nouveau noeud lors d'un virage
    fn add_snake_node(&mut self) {
        if self.moving != self.direction {
            let mut new_node: SnakeNode = SnakeNode::new(self.head.x, self.head.y);
            self.insert_node(&mut new_node);
        }
    }

    /// Insère un nouveau noeud au serpent
    ///
    /// # Arguments
    ///
    /// * `node` - Le nouveau noeud
    fn insert_node(&mut self, node: &mut SnakeNode) {
        match &self.head.next_node {
            None => (),
            Some(n) => node.next_node = Some(n.clone()),
        }
        self.head.next_node = Some(Box::new(node.clone()));
    }

    /// Renvoie une référence mutable vers la queue du serpent
    fn get_tail(&mut self) -> &mut SnakeNode {
        let mut node: &mut SnakeNode = &mut self.head;
        loop {
            match node.next_node {
                None => return node,
                Some(ref mut n) => {
                    node = n.deref_mut();
                }
            }
        }
    }

    /// Enlève la queue du serpent
    fn back_tail(&mut self) {
        let mut len: i16 = self.head.len() - 1;
        let mut node: &mut SnakeNode = &mut self.head;
        while len > 1 {
            match node.next_node {
                None => panic!("SnakeNode::len misfunctioned"),
                Some(ref mut n) => {
                    node = n.deref_mut();
                    len -= 1;
                }
            }            
        }
        node.next_node = None;
    }

    /// Renvoie les coordonnées de l'avant-dernier noeud (avant la queue)
    fn get_position_of_node_before_tail(&self) -> (i16, i16) {
        let mut node: &SnakeNode = &self.head;
        let mut next_node: &SnakeNode = &node.next_node.as_ref().unwrap().deref();
        loop {
            match next_node.next_node {
                None => return (node.x, node.y),
                Some(ref n) => {
                    node = next_node;
                    next_node = n.deref();
                }
            }
        }
    }

    /// Change l'intention de mouvement du serpent
    /// 
    /// # Arguments
    /// 
    /// `player_move` - Le mouvement reçu
    pub fn change_intent(&mut self, player_move: Move) {
        match player_move {
            Move::Up => {
                if self.direction != Move::Down {
                    self.moving = Move::Up;
                }
            }
            Move::Down => {
                if self.direction != Move::Up {
                    self.moving = Move::Down;
                }
            }
            Move::Left => {
                if self.direction != Move::Right {
                    self.moving = Move::Left;
                }
            }
            Move::Right => {
                if self.direction != Move::Left {
                    self.moving = Move::Right;
                }
            }
        }
    }
    
    /// Met à jour la position du serpent (tête et queue)
    pub fn update(&mut self) {
        if self.has_lost{
            return ();
        }
        // Met à jour la position du serpent (tête et queue)
        match self.moving {
            // Mouvement de la tête
            Move::Up => {
                // L'ordre est très important.
                self.add_snake_node();
                self.head.y -= 1;
                self.direction = Move::Up;
            }
            Move::Down => {
                self.add_snake_node();
                self.head.y += 1;
                self.direction = Move::Down;
            }
            Move::Right => {
                self.add_snake_node();
                self.head.x += 1;
                self.direction = Move::Right;
            }

            Move::Left => {
                self.add_snake_node();
                self.head.x -= 1;
                self.direction = Move::Left;
            }
            
        }
        
        // Mouvement de la queue
        if self.stomach > 0 {
            // Le serpent grandit
            self.stomach -= 1;
        } else {
            // La queue doit suivre le serpent
            let (x, y) = self.get_position_of_node_before_tail();
            let tail: &mut SnakeNode = self.get_tail();
            if tail.x == x && tail.y == y {
                self.back_tail();
            } 

            let (x, y) = self.get_position_of_node_before_tail();
            let tail: &mut SnakeNode = self.get_tail();
            if tail.x == x {
                tail.y += (y - tail.y) / (y - tail.y).abs();
            } else if tail.y == y {
                tail.x += (x - tail.x) / (x - tail.x).abs();
            }
        }
    }

    /// Indique si une coordonnée appartient au serpent
    /// 
    /// # Arguments
    /// 
    /// `x`, `y` - Les coordonnées dont on veut vérifier l'appartenance
    /// `id` - L'identifiant du serpent qui rentre potentiellement en collision 
    fn contains(&self, x: i16, y: i16, id: u8) -> bool {
        let mut node = &self.head;
        let mut next_node = node.next_node.as_ref().unwrap().deref();
        if id == self.id {
            if next_node.has_next() {
                node = next_node;
                next_node = node.next_node.as_ref().unwrap().deref();
            } else {
                return false;
            }
        }

        loop {
            // Segment vertical
            if node.x == next_node.x {
                if x == node.x {
                    if cmp::min(node.y, next_node.y) <= y && y <= cmp::max(node.y, next_node.y) {
                        return true;
                    }
                }
            }
            
            // Segment horizontal
            if node.y == next_node.y {
                if y == node.y {
                    if cmp::min(node.x, next_node.x) <= x && x <= cmp::max(node.x, next_node.x) {
                        return true
                    }
                }
            }

            if next_node.has_next() {
                node = next_node;
                next_node = node.next_node.as_ref().unwrap().deref();
            } else {
                break;
            }
        }
        
        false
    }
}

/// Implémentation d'une nourriture
impl Food {
    /// Renvoie une nouvelle nourriture
    pub fn new(x: i16, y: i16, food_type: FoodType) -> Self {
        Food {
            x,
            y,
            food_type,
        }
    }
    
    /// Renvoie l'identifiant numérique de la nourriture
    fn food_type_to_food_id(&self) -> u8 {
        match &self.food_type {
            FoodType::Apple => 1,
            FoodType::Mango => 2,
        }
    }

    /// Renvoie un triplet codant la nourriture adapté au protocole
    pub fn get_info_for_data_trs(&self) -> (u8, i16, i16) {
        (self.food_type_to_food_id(), self.x, self.y)
    }

    /// Réinitialise la nourriture (change ses coordonnées aléatoirement)
    ///
    /// # Arguments
    /// 
    /// `dimensions` - Taille de la map en blocs
    fn reset(&mut self, dimensions: u16) {
        let mut rng = rand::thread_rng();
        let rd = Uniform::from(2..dimensions - 2);
        let x = rd.sample(&mut rng);
        let y = rd.sample(&mut rng);
        self.x = x as i16;
        self.y = y as i16;
    }
}

/// Implémentation d'une partie
impl Game {
    /// Renvoie une nouvelle partie
    ///
    /// # Arguments
    ///
    /// * `dimensions` - Taille de la map en blocs
    /// * `frequency` - Fréquence de mouvement
    pub fn new(dimensions: u16, frequency: u8) -> Self {
        let mut rng = rand::thread_rng();
        let rd = Uniform::from(2..dimensions - 2);
        let food_x0 = rd.sample(&mut rng) as i16;
        let food_y0 = rd.sample(&mut rng) as i16;
        Game {
            dimensions,
            food: vec![Food::new(food_x0, food_y0, FoodType::Apple)],
            waiting: false,
            period: 1.0 / (frequency as f64), // Période entre deux mouvements (~framerate)
            progress: 0.0,
            players: vec![],
        }
    }

    /// Met à jour un serpent.
    /// Renvoie une option avec l'instant de mise à jour si le serpent a reçu un boost.
    ///
    /// # Arguments
    /// 
    /// `index` - L'indice du serpent dans le vecteur des joueurs
    pub fn update_snake(&mut self, index: usize) -> Option<time::Instant> {
        let snake: &mut Snake = self.players.get_mut(index).unwrap();
        snake.update();
        let id = snake.id;
        let x = snake.head.x;
        let y = snake.head.y;

        match self.check_tile(x, y, id) {
            TileType::Nothing => (),
            TileType::SnakeTile(snake_id) => self.killed(id, snake_id),
            TileType::Wall => self.killed(id, id),
            TileType::FoodTile(food_type) => {
                match food_type {
                    FoodType::Apple => {
                        // Allonge le serpent
                        self.feed(id, FOOD_BY_APPLE);
                        self.reset_food(x, y);
                    },
                    FoodType::Mango => {
                        // Donne un coup de boost temporaire au serpent
                        let snake: &mut Snake = self.players.get_mut(index).unwrap();
                        snake.boost = true;
                        self.delete_food(x, y);
                    },
                };

                if self.food.len() < MAX_FOOD {
                    // Quand de la nourriture est mangée, peu importe son type, il est possible qu'un nouveau apparaisse
                    let mut rng = rand::thread_rng();
                    let rd = Uniform::from(0..2);
                    // Une chance sur deux que ce soit le cas
                    if rd.sample(&mut rng) as u8 == 1 {
                        let rd = Uniform::from(2..self.dimensions - 2);
                        let food_x = rd.sample(&mut rng) as i16;
                        let food_y = rd.sample(&mut rng) as i16;
                        let rd = Uniform::from(1..N_FOOD_TYPES + 1);
                        /* 
                        // Le type de la nouvelle nourriture générée est choisie aléatoirement, mais à partir de 2 (pas de pommes)
                        let rd = Uniform::from(2..N_FOOD_TYPES + 1); 
                        */
                        let food_id = rd.sample(&mut rng) as u64;
                        self.food.push(Food::new(food_x, food_y, food_id_to_type(food_id)));
                    }
                }
                

                if food_type == FoodType::Mango {
                    return Some(time::Instant::now());
                }
            }
        }

        None
    }

    /// Supprime une nourriture
    /// 
    /// # Arguments
    /// 
    /// `x`, `y` - Les coordonnées de la nourriture
    fn delete_food(&mut self, x: i16, y: i16) {
        let mut i: usize = 0;
        while i < self.food.len() {
            let food = self.food.get_mut(i).unwrap();
            if food.x == x && food.y == y {
                self.food.remove(i);
            } else {
                i += 1;
            }
        }
    }


    /// Réinitialise une nourriture (changement aléatoire des coordonnées)
    /// 
    /// # Arguments
    /// 
    /// `food_type` - Le type de la nourriture
    fn reset_food(&mut self, x: i16, y: i16) {
        for i in 0..self.food.len() {
            let food = self.food.get_mut(i).unwrap();
            if food.x == x && food.y == y {
                food.reset(self.dimensions);
                return ();
            }
        }
    }

    /// Renvoie le type d'une case
    /// 
    /// # Arguments
    /// 
    /// `x`, `y` - Les coordonnées de la case à vérifier
    /// `id` - Identifiant du joueur souhaitant vérifier
    fn check_tile(&mut self, x: i16, y: i16, id: u8) -> TileType {
        for snake in &self.players {
            if snake.contains(x, y, id) {
                return TileType::SnakeTile(snake.id);
            }
        }

        for food in &self.food {
            if food.x == x && food.y == y {
                return TileType::FoodTile(food.food_type.clone());        
            }
        }

        if x < 1 || y < 1 || x >= self.dimensions as i16 -1 || y >= self.dimensions as i16 - 1 {
            return TileType::Wall;
        }

        TileType::Nothing
    }

    /// Traite le meurtre d'un serpent.
    /// Si `DEV_NO_DEATH`, aucun serpent ne meurt effectivement.
    /// 
    /// # Arguments
    /// 
    /// `murdered` - identifiant du joueur tué
    /// `murderer` - identifiant du joueur qui a tué
    fn killed(&mut self, murdered: u8, murderer: u8) {
        println!("Le serpent {} a tué le serpent {} !", murderer, murdered);
        if DEV_NO_DEATH {
            ()
        } else {
            self.set_lost(murdered);
        }
    }
    
    /// Nourrit un serpent (rajoute des unités à son estomac)
    /// 
    /// # Arguments
    /// 
    /// `player_id` - Identifiant du joueur à nourrir
    /// `food` - Nombre d'unités à rajouter à l'estomac
    fn feed(&mut self, player_id: u8, food: u8) {
        let index = self.get_player(player_id).unwrap();
        self.players.get_mut(index).unwrap().stomach += food;
    }

    /// Ajoute un nouveau joueur à la partie
    /// 
    /// # Arguments
    /// 
    /// `player_id` - Identifiant du joueur
    /// `color` - Couleur du joueur (triplet RGB)
    /// `x0`, `y0` - Coordonnées initiales du joueur 
    pub fn add_player(&mut self, player_id: u8, color: (u8, u8, u8), x0: u16, y0: u16) -> () {
        let (initial_direction, initial_moving): (Move, Move) = match (x0 > self.dimensions / 2, y0 > self.dimensions / 2) {
            (false, false) => (Move::Down, Move::Down),
            (false, true) => (Move::Right, Move::Right),
            (true, false) => (Move::Left, Move::Left),
            (true, true) => (Move::Up, Move::Up),
        };

        self.players.push(
            Snake::new(player_id, color, x0 as i16, y0 as i16, initial_direction, initial_moving)
        );
    }

    /// Renvoie une option de l'index du joueur dans la liste des joueurs.
    /// **Cette donnée est différente de l'identifiant du joueur !**
    ///
    /// # Arguments
    ///
    /// * `player_id` - L'identifiant du joueur
    pub fn get_player(&self, player_id: u8) -> Option<usize> {
        for i in 0..self.players.len() {
            if self.players[i].id == player_id {
                return Some(i);
            }
        }
        
        None
    }

    /// Considère qu'un joueur a perdu
    /// 
    /// # Arguments
    /// 
    /// `player_id` - L'identifiant du joueur ayant perdu
    pub fn set_lost(&mut self, player_id: u8) -> () {
        let index: usize = self.get_player(player_id).unwrap();
        self.players[index].has_lost = true;
    }

    /// Renvoie un vecteur des identifiants des joueurs encore en vie
    pub fn get_alive(&self) -> Vec<u8> {
        let mut alive: Vec<u8> = vec![];
        for player_id in 1..((self.players.len() + 1) as u8) {
            let index: usize = self.get_player(player_id).unwrap();
            if !self.players[index].has_lost {
                alive.push(player_id);
            }
        }
        alive
    }

    /// Renvoie un vecteur de références vers les serpents encore en vie
    pub fn players_alive(&self) -> Vec<&Snake> {
        let alive: Vec<u8> = self.get_alive();
        let mut snakes_alive : Vec<&Snake> = vec![];
        for id in alive {
            snakes_alive.push(self.players.get(self.get_player(id).unwrap()).unwrap());
        }
        snakes_alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_next1() {
        let node1 = SnakeNode::new(0,0);
        assert!(!node1.has_next());
    }

    #[test]
    fn test_has_next2() {
        let mut node1 = SnakeNode::new(0,0);
        let node2 = SnakeNode::new(0,0);
        node1.next_node = Some (Box::new(node2));
        assert!(node1.has_next());
    }
    
    #[test]
    fn test_len1() {
        let mut node1 = SnakeNode::new(0, 0);
        let mut node2 = SnakeNode::new(0, 0);
        let node3 = SnakeNode::new(0, 0);
        node2.next_node = Some(Box::new(node3));
        node1.next_node = Some(Box::new(node2));
        assert_eq!(node1.len(), 3);
    }

    #[test]
    fn test_len2() {
        let node1 = SnakeNode::new(0, 0);
        assert_eq!(node1.len(), 1);
    }
}
