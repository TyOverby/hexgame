extern crate hexagon;

pub mod ai;

use hexagon::grid::{Map, HexGrid};
use hexagon::HexPosition;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct GameState {
    current_player: Player,
    map: Map<Player, HexGrid>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Player {
    Green,
    Red
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MoveResult {
    Good,
    Bad,
    End(Player),
    Tie,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            current_player: Player::Red,
            map: Map::new(HexGrid::new(4))
        }
    }

    pub fn make_move(&mut self, pos: &HexPosition) -> MoveResult {
        if self.map.could_contain(&pos) && !self.map.contains(&pos) {
            self.map.insert(pos, self.current_player);
            self.current_player = self.current_player.inverse();
            self.is_over()
        } else {
            MoveResult::Bad
        }
    }

    pub fn is_game_over(&self) -> bool {
        match self.is_over() {
            MoveResult::End(_) => true,
            _ => false
        }
    }
    pub fn is_over(&self) -> MoveResult {
        if self.map.is_full() {
            return MoveResult::Tie;
        }
        for player in &[Player::Green, Player::Red] {
            let mut status = None;
            for placed in self.map.iter().map(|(pos, _)| pos) {
                for ray in placed.rays().iter().cloned() {
                    let iter = ray.take_while(|pos| self.map.get(pos) == Some(player));
                    match iter.count() {
                        4 => status = Some(MoveResult::End(*player)),
                        3 if status.is_none() => status = Some(MoveResult::End(player.inverse())),
                        _ => {}
                    }
                }
            }

            if let Some(status) = status {
                return status;
            }
        }

        return MoveResult::Good;
    }

    pub fn current_player(&self) -> Player {
        self.current_player
    }

    pub fn map(&self) -> &Map<Player, HexGrid> {
        &self.map
    }
}

impl Player {
    pub fn starting() -> Player {
        Player::Red
    }

    pub fn inverse(&self) -> Player {
        match *self {
            Player::Green => Player::Red,
            Player::Red => Player::Green,
        }
    }

    pub fn color(&self) -> [f32; 4] {
        match *self {
            Player::Green => [0.0, 1.0, 0.0, 1.0],
            Player::Red => [1.0, 0.0, 0.0, 1.0],
        }
    }
}
