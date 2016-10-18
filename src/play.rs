extern crate hexagon;
extern crate lux;
extern crate hexgame;

use hexgame::{GameState, Player, MoveResult};
use hexgame::ai::*;
use lux::prelude::*;
use lux::interactive::Event;
use lux::graphics::{ColorVertex, PrimitiveType, PrimitiveCanvas};
use hexagon::*;
use hexagon::screen::ScreenSpace;
use hexagon::grid::Grid;

const TIME_BETWEEN_GAMES: u32 = 3_000;

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
    }, 4);

    let mut screenspace = ScreenSpace {
        size: 50.0,
        origin: (500.0, 500.0),
    };

    while window.is_open() {
        //screenspace.origin = (window.width() / 2.0, window.height() / 2.0);

        let mut frame = window.cleared_frame((0.0, 0.0, 0.0));
        let (x, y) = window.mouse_pos();

        render_game(&mut frame, &game, &screenspace);
        if let MoveResult::End(p) = game.is_over() {
            display_gameover(frame, p);
            ::std::thread::sleep_ms(TIME_BETWEEN_GAMES);
            game = GameState::new();
            continue;
        }

        let near_cursor = &screenspace.nearest_hex(x, y);
        if game.map().could_contain(&near_cursor) && !game.map().contains(&near_cursor) {
            draw_hex(&mut frame, &screenspace, near_cursor, game.current_player().color(), 45.0);
            draw_hex(&mut frame, &screenspace, near_cursor, [1.0, 1.0, 1.0, 1.0], 40.0);
        }

        if window.events().filter(|e| match e { &Event::MouseUp(_) => true, _ => false}).count() != 0 {
            if game.map().could_contain(&near_cursor) {
                if let MoveResult::End(_) = game.make_move(near_cursor) {
                    continue;
                }

                let next = ai.choose(&game, game.current_player());
                game.make_move(&next);
            }
        }
    }
}

fn display_gameover(mut frame: Frame, player: Player) {
    frame.draw(Rectangle {
        x: 0.0,
        y: 0.0,
        w: 50.0,
        h: 50.0,
        color: player.color(),
        .. Default::default()
    });
}
