extern crate hexagon;
extern crate hexgame;
extern crate rand;

use hexgame::{GameState, Player, MoveResult};
use hexgame::ai::*;
use hexagon::*;
use hexagon::screen::ScreenSpace;
use hexagon::grid::Grid;


type FeatureRankerAi = RankerAi<FeatureRanker>;

#[derive(Debug)]
enum GameResult {
    Tie,
    Player1,
    Player2,
}

fn random_ai(rec_depth: usize) -> FeatureRankerAi {
    let mut ranker = FeatureRanker {
        window_score: rand::random(),
        triad_score: rand::random(),
        slot_score: rand::random(),
        double_score: rand::random(),
    };

    ranker.normalize();

    RankerAi::new(ranker, rec_depth)
}

fn breed(a: &FeatureRanker, b: &FeatureRanker) -> FeatureRanker {
    let mut ranker = FeatureRanker {
        window_score: a.window_score + b.window_score,
        triad_score: a.triad_score + b.triad_score,
        slot_score: a.slot_score + b.slot_score,
        double_score: a.double_score + b.double_score,
    };

    ranker.normalize();

    ranker
}

fn randomize(a: &FeatureRanker) -> FeatureRanker {
    let mut new = FeatureRanker {
        window_score: a.window_score + rand::random::<f32>() - 0.5,
        triad_score: a.triad_score + rand::random::<f32>() - 0.5,
        slot_score: a.slot_score + rand::random::<f32>() - 0.5,
        double_score: a.double_score + rand::random::<f32>() - 0.5,
    };
    new.normalize();
    new
}

fn play_game(a: &mut FeatureRankerAi, b: &mut FeatureRankerAi) -> GameResult {
    let mut game = GameState::new();
    let mut toggle = true;
    loop {
        let next = if toggle {
            a.choose(&game, game.current_player())
        } else {
            b.choose(&game, game.current_player())
        };
        let (q, r) = next.as_axial();
        println!("{},{}", q, r);
        match game.make_move(&next) {
            MoveResult::Good => {},
            MoveResult::Bad => panic!("bad move"),
            MoveResult::End(Player::Red) => return GameResult::Player1,
            MoveResult::End(Player::Green) => return GameResult::Player2,
            MoveResult::Tie => return GameResult::Tie,
        }

        toggle = !toggle;
    }
}

fn main() {
    let mut a = random_ai(3);
    let mut b = random_ai(3);
    println!("{:?}", play_game(&mut a, &mut b));
}
