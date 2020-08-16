use piston_window::types::Color;
use piston_window::*;

use crate::game::Game;
use crate::game::SnakeNode;
use crate::ClientState;
use crate::DEFAULT_BLOCK_SIZE;
use crate::DEFAULT_GAME_DIMENSIONS;
use crate::HUD_WIDTH;

/// Couleur du fond du HUD
const BLACK_HUD: Color = [0.14, 0.14, 0.14, 0.8];

/// Couleur du texte
const TEXT_COLOR: Color = [0.96, 0.96, 0.96, 1.0];

/// Couleur du texte pour les joueurs morts
const DEAD_COLOR: Color = [0.94, 0.08, 0.08, 1.0];

/// Taille en pixels de la police d'écriture
const FONT_SIZE: u32 = 16;

/// Taille en pixels d'une ligne de texte dans le HUD
const LINE_HEIGHT: f64 = (FONT_SIZE + 8) as f64;

/// Dessine un rectangle de couleur à partir de données en pixels
///
/// # Arguments
///
/// * `c` - Une référence vers le contexte de la fenêtre
/// * `g` - Une référence mutable vers l'objet graphique 2D
/// * `col` - La couleur du rectangle
/// * `x`, `y` - Coordonnées du point en haut à gauche du rectangle en **pixels**
/// * `h`, `w` - Hauteur et largeur du rectangle en **pixels**
pub fn draw_rectangle_raw(c: &Context, g: &mut G2d, col: Color, x: i16, y: i16, h: i16, w: i16) {
    rectangle(
        col,
        [x as f64, y as f64, w as f64, h as f64],
        c.transform,
        g,
    );
}

/// Dessine un rectangle de couleur à partir de données en blocs
///
/// # Arguments
///
/// * `c` - Une référence vers le contexte de la fenêtre
/// * `g` - Une référence mutable vers l'objet graphique 2D
/// * `col` - La couleur du rectangle
/// * `x`, `y` - Coordonnées du point en haut à gauche du rectangle en **nombre de blocs**
/// * `h`, `w` - Hauteur et largeur du rectangle en **nombre de blocs**
/// * `block_size` - Taille d'un bloc en pixels
pub fn draw_rectangle(
    c: &Context,
    g: &mut G2d,
    col: Color,
    x: i16,
    y: i16,
    h: i16,
    w: i16,
    block_size: u16,
) {
    rectangle(
        col,
        [
            x as f64 * block_size as f64,
            y as f64 * block_size as f64,
            w as f64 * block_size as f64,
            h as f64 * block_size as f64,
        ],
        c.transform,
        g,
    );
}

/// Dessine une section entre deux noeuds d'un serpent
///
/// # Arguments
///
/// * `c` - Une référence vers le contexte de la fenêtre
/// * `g` - Une référence mutable vers l'objet graphique 2D
/// * `col` - La couleur du serpent
/// * `node1`, `node2` - Des références vers les noeuds du serpent
/// * `block_size` - Taille d'un bloc en pixels
pub fn draw_section(
    c: &Context,
    g: &mut G2d,
    col: Color,
    node1: &SnakeNode,
    node2: &SnakeNode,
    block_size: u16,
) {
    if node1.x != node2.x && node1.y != node2.y {
        println!("Node {:?} and node {:?} are not aligned.", node1, node2);
        panic!("");
    } else {
        if node1.x == node2.x {
            if node1.y < node2.y {
                draw_rectangle(
                    c,
                    g,
                    col,
                    node1.x,
                    node1.y,
                    1 + node2.y - node1.y,
                    1,
                    block_size,
                );
            } else {
                draw_rectangle(
                    c,
                    g,
                    col,
                    node2.x,
                    node2.y,
                    1 + node1.y - node2.y,
                    1,
                    block_size,
                );
            }
        } else {
            if node1.x < node2.x {
                draw_rectangle(
                    c,
                    g,
                    col,
                    node1.x,
                    node1.y,
                    1,
                    1 + node2.x - node1.x,
                    block_size,
                );
            } else {
                draw_rectangle(
                    c,
                    g,
                    col,
                    node2.x,
                    node2.y,
                    1,
                    1 + node1.x - node2.x,
                    block_size,
                );
            }
        }
    }
}

/// Dessine une bordure de map d'épaisseur un bloc en partant de (0, 0)
///
/// # Arguments
///
/// * `c` - Une référence vers le contexte de la fenêtre
/// * `g` - Une référence mutable vers l'objet graphique 2D
/// * `col` - La couleur de la bordure
/// * `dimensions` - Dimensions de la map (carrée) en blocs
/// * `block_size` - Taille d'un bloc en pixels
pub fn draw_borders(c: &Context, g: &mut G2d, col: Color, dimensions: u16, block_size: u16) {
    draw_rectangle(c, g, col, 0, 0, dimensions as i16, 1, block_size);
    draw_rectangle(c, g, col, 0, 0, 1, dimensions as i16, block_size);
    draw_rectangle(
        c,
        g,
        col,
        dimensions as i16 - 1,
        0,
        dimensions as i16,
        1,
        block_size,
    );
    draw_rectangle(
        c,
        g,
        col,
        0,
        dimensions as i16 - 1,
        1,
        dimensions as i16,
        block_size,
    );
}

/// Dessine le HUD
///
/// # Arguments
///
/// * `c` - Une référence vers le contexte de la fenêtre
/// * `g` - Une référence mutable vers l'objet graphique 2D
/// * `glyphs` - Une référence mutable vers le cache Glyph pour la police d'écriture
/// * `game` - Une référence mutable vers le jeu
/// * `client_state` - Une référence vers l'état du client de jeu
/// * `address` - Une slice vers l'adresse IP du serveur
/// * `id` - L'identifiant du joueur
/// * `alive_assoc` - Une référence vers un vecteur d'association codant les joueurs encore en vie
pub fn draw_hud(
    c: &Context,
    g: &mut G2d,
    glyphs: &mut Glyphs,
    game: &mut Game,
    client_state: &ClientState,
    address: &str,
    id: u8,
    alive_assoc: &Vec<(u8, bool)>,
) {
    let window_size = DEFAULT_GAME_DIMENSIONS;

    draw_rectangle_raw(
        c,
        g,
        BLACK_HUD,
        (window_size * DEFAULT_BLOCK_SIZE) as i16,
        0,
        (window_size * DEFAULT_BLOCK_SIZE) as i16,
        HUD_WIDTH as i16,
    );

    // Affichage de l'adresse du serveur
    let mut line: i16 = 0;
    text::Text::new_color(TEXT_COLOR, FONT_SIZE)
        .draw(
            address,
            glyphs,
            &c.draw_state,
            c.transform.trans(
                (window_size * DEFAULT_BLOCK_SIZE + 15) as f64,
                25.0 + line as f64 * LINE_HEIGHT,
            ),
            g,
        )
        .unwrap();
    line += 1;

    // Affichage du statut
    match client_state {
        ClientState::OnGoing | ClientState::Waiting => {
            text::Text::new_color(TEXT_COLOR, FONT_SIZE)
                .draw(
                    match client_state {
                        ClientState::OnGoing => "Connecté",
                        ClientState::Waiting => "En attente de joueurs",
                        _ => "ERREUR",
                    },
                    glyphs,
                    &c.draw_state,
                    c.transform.trans(
                        (window_size * DEFAULT_BLOCK_SIZE + 15) as f64,
                        25.0 + line as f64 * LINE_HEIGHT,
                    ),
                    g,
                )
                .unwrap();
            line += 1;
        }
        ClientState::EndOfGame => {
            text::Text::new_color(TEXT_COLOR, FONT_SIZE)
                .draw(
                    "Partie terminée",
                    glyphs,
                    &c.draw_state,
                    c.transform.trans(
                        (window_size * DEFAULT_BLOCK_SIZE + 15) as f64,
                        25.0 + line as f64 * LINE_HEIGHT,
                    ),
                    g,
                )
                .unwrap();
            line += 1;
            text::Text::new_color(TEXT_COLOR, FONT_SIZE)
                .draw(
                    "[R]: Rejouer",
                    glyphs,
                    &c.draw_state,
                    c.transform.trans(
                        (window_size * DEFAULT_BLOCK_SIZE + 15) as f64,
                        25.0 + line as f64 * LINE_HEIGHT,
                    ),
                    g,
                )
                .unwrap();
            line += 1;
        }
    }
    // Affichage des joueurs et leur couleur
    let mut line_number = line;
    for snake in &game.players {
        draw_rectangle_raw(
            &c,
            g,
            TEXT_COLOR,
            (window_size * DEFAULT_BLOCK_SIZE + 25) as i16,
            25 + (LINE_HEIGHT as i16) * (2 + line_number),
            LINE_HEIGHT as i16,
            LINE_HEIGHT as i16,
        );
        draw_rectangle_raw(
            &c,
            g,
            snake.color,
            (window_size * DEFAULT_BLOCK_SIZE + 25 + 1) as i16,
            25 + (LINE_HEIGHT as i16) * (2 + line_number) + 1,
            (LINE_HEIGHT - 2.0) as i16,
            (LINE_HEIGHT - 2.0) as i16,
        );

        let player_text: String = if snake.id == id {
            format!("Joueur {}  (vous)", snake.id)
        } else {
            format!("Joueur {}", snake.id)
        };
        let mut player_text_color: Color = DEAD_COLOR;
        for (current_id, _) in alive_assoc {
            if *current_id == snake.id {
                player_text_color = TEXT_COLOR;
                break;
            }
        }

        text::Text::new_color(player_text_color, FONT_SIZE)
            .draw(
                &player_text,
                glyphs,
                &c.draw_state,
                c.transform.trans(
                    (window_size * DEFAULT_BLOCK_SIZE + 20) as f64 + LINE_HEIGHT * 1.8,
                    25.0 + LINE_HEIGHT * (2.75 + (line_number as f64)),
                ),
                g,
            )
            .unwrap();
        line_number += 2;
    }
}
