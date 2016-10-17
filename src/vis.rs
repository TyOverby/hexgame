extern crate hexagon;
extern crate lux;
extern crate hexgame;

use hexgame::{GameState};
use std::io::BufRead;
use lux::prelude::*;
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

enum Command {
    Move(HexPosition),
    Reset,
}

fn main() {
    use std::sync::mpsc::channel;
    use std::thread::spawn;
    let mut window = Window::new_with_defaults().unwrap();
    let mut game = GameState::new();

    let screenspace = ScreenSpace {
        size: 50.0,
        origin: (500.0, 500.0),
    };

    let (s, r) = channel();
    spawn(move || {
        let stdin = ::std::io::stdin();
        let lines = stdin.lock().lines();
        for line in lines {
            let line = line.ok();
            let mv = 
                line.clone().and_then(|parts| {
                    if parts.starts_with("new game") {
                        Some(Command::Reset)
                    } else {
                        let mut parts = parts.split(",");
                        parts.next().and_then(|part_1| {
                            part_1.trim().parse().ok().and_then(|part_1| {
                                parts.next().and_then(|part_2| {
                                    part_2.trim().parse().ok().and_then(|part_2| {
                                        Some(Command::Move(HexPosition::from_axial(part_1, part_2)))
                                    })
                                })
                            })
                        })
                    }
                });
            if let Some(mv) = mv {
                s.send(mv).unwrap();
            } 
            else {
                println!("{}", line.unwrap());
            }
        }
    });

    while window.is_open() {
        let mut frame = window.cleared_frame((0.0, 0.0, 0.0));
        render_game(&mut frame, &game, &screenspace);

        match r.try_recv() {
            Ok(Command::Move(mov)) => {
                game.make_move(&mov);
                ::std::thread::sleep_ms(500);
            },
            Ok(Command::Reset) => {
                game = GameState::new();
            },
            Err(_) => {}
        }
    }
}
