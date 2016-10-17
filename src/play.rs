extern crate hexagon;
extern crate lux;
extern crate hexgame;

use hexgame::{GameState, Player};
use hexgame::ai::*;
use lux::prelude::*;
use lux::interactive::Event;
use lux::graphics::{ColorVertex, PrimitiveType, PrimitiveCanvas};
use hexagon::*;
use hexagon::screen::ScreenSpace;
use hexagon::grid::Grid;

fn draw_hex(frame: &mut Frame, screen: &ScreenSpace, hex: &HexPosition, color: [f32; 4], size: f32) {
    fn get_color_vertex((x, y): (f32, f32), color: [f32; 4]) -> ColorVertex {
        ColorVertex {
            pos: [x, y],
            color: color,
        }
    }

    let hex_positions = screen.points_on_tile_custom_size(&hex, size);

    frame.draw_colored(PrimitiveType::TriangleFan, &[
        get_color_vertex(hex_positions[0], color),
        get_color_vertex(hex_positions[1], color),
        get_color_vertex(hex_positions[2], color),
        get_color_vertex(hex_positions[3], color),
        get_color_vertex(hex_positions[4], color),
        get_color_vertex(hex_positions[5], color),
    ], None, None).unwrap();
}

fn render_game(frame: &mut Frame, state: &GameState, screen: &ScreenSpace) {
    let size = 45.0;

    for tile in state.map().grid().iter() {
        draw_hex(frame, screen, &tile, [1.0, 1.0, 1.0, 1.0], size);
    }

    for (tile, player) in state.map().iter() {
        draw_hex(frame, screen, &tile, player.color(), size);
    }
}

fn main() {
    let mut window = Window::new_with_defaults().unwrap();

    let mut game = GameState::new();

    let mut ai = RankerAi::new(FeatureRanker {
        window_score: 2.0,
        triad_score: 5.0,
        slot_score: 1.0,
        double_score: 1.0,
    }, 3);
    //let mut ai = RankerAi::new(NullRanker, 3);

    let screenspace = ScreenSpace {
        size: 50.0,
        origin: (500.0, 500.0),
    };

    while window.is_open() {
        let mut frame = window.cleared_frame((0.0, 0.0, 0.0));
        let (x, y) = window.mouse_pos();

        render_game(&mut frame, &game, &screenspace);

        let near_cursor = &screenspace.nearest_hex(x, y);
        if game.map().could_contain(&near_cursor) && !game.map().contains(&near_cursor) {
            draw_hex(&mut frame, &screenspace, near_cursor, game.current_player().color(), 45.0);
            draw_hex(&mut frame, &screenspace, near_cursor, [1.0, 1.0, 1.0, 1.0], 40.0);
        }

        if window.events().filter(|e| match e { &Event::MouseUp(_) => true, _ => false}).count() != 0 {
            if game.map().could_contain(&near_cursor) {
                println!("{:?}", game.make_move(near_cursor));
                println!("{:?}", game.is_over());
                println!("# triads: {}", FeatureRanker::count_triads(&game, Player::Red));
                println!("# windows: {}", FeatureRanker::count_windows(&game, Player::Red));
                println!("# slots: {}", FeatureRanker::count_slots(&game, Player::Red));
                println!("# doubles: {}", FeatureRanker::count_doubles(&game, Player::Red));
                println!("");

                /*
                let next = ai.choose(&game, game.current_player());
                game.make_move(&next);
                */
            }
        }
    }
}
