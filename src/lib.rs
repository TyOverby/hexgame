extern crate hexagon;
extern crate rand;

pub mod ai;

use hexagon::grid::{Map, HexGrid};
use hexagon::HexPosition;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct GameState {
    current_player: Player,
    map: Map<Player, HexGrid>,
    last_move: Option<HexPosition>,
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
            map: Map::new(HexGrid::new(4)),
            last_move: None,
        }
    }

    pub fn make_move(&mut self, pos: &HexPosition) -> MoveResult {
        if self.map.could_contain(&pos) && !self.map.contains(&pos) {
            self.map.insert(pos, self.current_player);
            self.current_player = self.current_player.inverse();
            self.last_move = Some(*pos);
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

        let last_move = if let Some(last_move) = self.last_move {
            last_move
        } else {
            return MoveResult::Good;
        };

        let player = self.map.get(&last_move).unwrap();

        let mut status = None;
        for i in 0 .. 3 {
            let (a, b) = last_move.bidirectional_ray(i);
            let a_cnt = a.skip(1).take_while(|p| self.map.get(&p) == Some(&player)).count();
            let b_cnt = b.skip(1).take_while(|p| self.map.get(&p) == Some(&player)).count();

            status = match (status, a_cnt + b_cnt + 1) {
                (None, 3) => Some(player.inverse()),
                (_, 4) | (_, 5) => Some(*player),
                (other, _) => other,
            };
        }

        if let Some(who_won) = status {
            MoveResult::End(who_won)
        } else {
            MoveResult::Good
        }
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
