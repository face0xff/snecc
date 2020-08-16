use std::ops::DerefMut;

use piston_window::types::Color;
use piston_window::Context;
use piston_window::G2d;
use piston_window::Key;

use crate::draw::*;

/// Différents mouvements
#[derive(PartialEq)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

/// Différents types de nourriture
pub enum FoodType {
    /// [1] Pomme
    Apple,
    /// [2] Mangue
    Mango,
    /// Autres
    Unknown,
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
    pub color: Color,
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
    pub dimensions: u16,
    /// Taille en pixels d'un bloc
    pub block_size: u16,
    /// Couleur de la bordure de la map
    border_color: Color,
    /// Vecteur des nourritures présentes sur la map
    pub food: Vec<Food>,
    pub waiting: bool,
    /// Nombre de joueurs dans la partie
    pub n_players: u8,
    pub can_send_move: bool,
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
    pub fn new(
        id: u8,
        color: (u8, u8, u8),
        x: i16,
        y: i16,
        initial_direction: Move,
        initial_moving: Move,
    ) -> Self {
        let mut head: SnakeNode = SnakeNode::new(x, y);
        let tail: SnakeNode = SnakeNode::new(x, y);
        head.next_node = Some(Box::new(tail));
        let color: Color = [
            (color.0 as f32) / 256.0,
            (color.1 as f32) / 256.0,
            (color.2 as f32) / 256.0,
            1.0,
        ];
        Snake {
            id,
            color,
            head,
            direction: initial_direction,
            moving: initial_moving,
            has_lost: false,
            stomach: 10,
        }
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
    /// Dessine le serpent
    ///
    /// # Arguments
    ///
    /// * `c` - Référence vers le contexte de la fenêtre
    /// * `g` - Référence mutable vers l'objet graphique 2D
    /// * `block_size` - Taille d'un bloc en pixels
    pub fn draw(&self, c: &Context, g: &mut G2d, block_size: u16) {
        // Dessine le serpent
        let mut snake_node: &SnakeNode = &self.head;
        let mut prev_node: &SnakeNode = &self.head;
        draw_section(c, g, self.color, snake_node, prev_node, block_size);
        loop {
            match snake_node.next_node {
                None => {
                    break;
                }
                Some(ref snake) => {
                    prev_node = snake_node;
                    snake_node = snake;
                    draw_section(c, g, self.color, snake_node, prev_node, block_size);
                }
            }
        }
    }

    /// Réinitialise un serpent à partir d'un noeud.
    /// Utile pour reconstruire le serpent lors de la réception d'une image.
    ///
    /// # Arguments
    ///
    /// * `node` - Le nouveau noeud
    pub fn reset_snake_with_node(&mut self, node: SnakeNode) {
        self.head = node;
    }

    /// Ajoute un nouveau noeud en tant que queue
    ///
    /// # Arguments
    ///
    /// * `node` - Le nouveau noeud
    pub fn add_node_as_tail(&mut self, node: SnakeNode) {
        self.get_tail().next_node = Some(Box::new(node));
    }
}

/// Implémentation d'une nourriture
impl Food {
    /// Renvoie un objet FoodType à partir de l'identifiant numérique
    ///
    /// # Arguments
    ///
    /// * `i` - L'identifiant de la nourriture
    pub fn int_to_food_type(i: u8) -> FoodType {
        match i {
            1 => FoodType::Apple,
            2 => FoodType::Mango,
            _ => FoodType::Unknown,
        }
    }

    /// Renvoie la couleur associée à la nourriture
    fn food_type_to_color(&self) -> Color {
        match &self.food_type {
            FoodType::Apple => [1.00, 0.00, 0.00, 1.0],
            FoodType::Mango => [0.88, 0.65, 0.04, 1.0],
            FoodType::Unknown => panic!("Trying to convert unknown food type"),
        }
    }

    /// Dessine la nourriture
    ///
    /// # Arguments
    ///
    /// * `c` - Référence vers le contexte de la fenêtre
    /// * `g` - Référence mutable vers l'objet graphique 2D
    /// * `block_size` - Taille d'un bloc en pixels
    pub fn draw_food(&self, c: &Context, g: &mut G2d, block_size: u16) {
        draw_rectangle(
            c,
            g,
            self.food_type_to_color(),
            self.x,
            self.y,
            1,
            1,
            block_size,
        );
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
        Game {
            dimensions,
            food: vec![],
            block_size: 0,
            n_players: 0,
            waiting: false,
            period: 1.0 / (frequency as f64), // Période entre deux mouvements (~framerate)
            progress: 0.0,
            players: vec![],
            can_send_move: true,
            border_color: [1.0, 1.0, 1.0, 0.8],
        }
    }

    /// Définit la couleur de la bordure de la map
    ///
    /// # Arguments
    ///
    /// * `col` - La couleur
    pub fn set_border_color(&mut self, col: Color) {
        self.border_color = col;
    }

    /// Dessine la partie
    ///
    /// # Arguments
    ///
    /// * `c` - Référence vers le contexte de la fenêtre
    /// * `g` - Référence mutable vers l'objet graphique 2D
    /// * `id` - Identifiant du joueur
    pub fn draw_game(&self, c: &Context, g: &mut G2d, id: u8) {
        draw_borders(c, g, self.border_color, self.dimensions, self.block_size);
        for snake in &self.players {
            snake.draw(c, g, self.block_size);
            if id == snake.id {
                draw_rectangle(
                    c,
                    g,
                    [1.0, 1.0, 1.0, 1.0],
                    snake.head.x,
                    snake.head.y,
                    1,
                    1,
                    self.block_size,
                );
            }
        }
        for food in &self.food {
            food.draw_food(c, g, self.block_size);
        }
    }

    /// Vide la liste de nourriture
    pub fn clear_food(&mut self) {
        self.food = vec![];
    }

    /// Ajoute une nourriture
    ///
    /// # Arguments
    ///
    /// * `x`, `y` - Coordonnées de la nouvelle nourriture
    /// * `food_type` - Le type de la nouvelle nourriture
    pub fn add_food(&mut self, x: i16, y: i16, food_type: FoodType) {
        self.food.push(Food { x, y, food_type });
    }

    /// Renvoie une option de l'index du joueur dans la liste des joueurs.
    /// **Cette donnée est différente de l'identifiant du joueur !**
    ///
    /// # Arguments
    ///
    /// * `player_id` - L'identifiant du joueur
    pub fn get_player_index(&self, player_id: u8) -> Option<usize> {
        for i in 0..self.players.len() {
            if self.players[i].id == player_id {
                return Some(i);
            }
        }
        None
    }

    /// Initialise les joueurs de la partie
    ///
    /// # Arguments
    ///
    /// * `player_params` - Vecteur des paramètres des joueurs (identifiant, RGB, coordonnées)
    pub fn init_players(&mut self, player_params: Vec<(u8, u8, u8, u8, i16, i16)>) {
        for (id, red, green, blue, x, y) in player_params {
            let (initial_direction, initial_moving): (Move, Move) = match (
                x > (self.dimensions as i16) / 2,
                y > (self.dimensions as i16) / 2,
            ) {
                (false, false) => (Move::Down, Move::Down),
                (false, true) => (Move::Right, Move::Right),
                (true, false) => (Move::Left, Move::Left),
                (true, true) => (Move::Up, Move::Up),
            };
            let snake = Snake::new(
                id,
                (red, green, blue),
                x,
                y,
                initial_direction,
                initial_moving,
            );
            self.players.push(snake);
        }
    }

    /// Traite l'appui sur une touche de type flèche directionnelle
    ///
    /// # Arguments
    ///
    /// * `id` - Identifiant du joueur
    /// * `key` - Touche appuyée par le joueur
    pub fn key_pressed(&mut self, id: u8, key: Key) {
        let index = self
            .get_player_index(id)
            .expect("game.key_pressed, not found id");
        let snake_by_id: &mut Snake = match self.players.get_mut(index) {
            None => panic!("game.key_pressed, not found id: {}", id),
            Some(snake) => snake,
        };
        match key {
            Key::Up | Key::Z => {
                snake_by_id.moving = Move::Up;
            }
            Key::Down | Key::S => {
                snake_by_id.moving = Move::Down;
            }
            Key::Left | Key::Q => {
                snake_by_id.moving = Move::Left;
            }
            Key::Right | Key::D => {
                snake_by_id.moving = Move::Right;
            }
            _ => (),
        }
    }
}
