extern crate hexagon;
extern crate hexgame;
extern crate rand;
extern crate pbr;

use hexgame::{GameState, Player, MoveResult};
use hexgame::ai::*;

use std::cell::RefCell;

type FeatureRankerAi = RankerAi<FeatureRanker>;

#[derive(Debug)]
enum GameResult {
    Tie,
    Player1,
    Player2,
}

fn random_ai(rec_depth: usize) -> FeatureRankerAi {
    let mut ranker = FeatureRanker {
        window_score: rand::random::<f32>() - 0.5,
        triad_score: rand::random::<f32>() - 0.5,
        slot_score: rand::random::<f32>() - 0.5,
        double_score: rand::random::<f32>() - 0.5,
    };

    ranker.normalize();

    RankerAi::new(ranker, rec_depth)
}

fn breed(a: &FeatureRanker, b: &FeatureRanker) -> FeatureRanker {
    let mut ranker = FeatureRanker {
        window_score: 2.0 * a.window_score + b.window_score,
        triad_score: 2.0 * a.triad_score + b.triad_score,
        slot_score: 2.0 * a.slot_score + b.slot_score,
        double_score: 2.0 * a.double_score + b.double_score,
    };

    ranker.normalize();

    ranker
}

fn randomize(a: &FeatureRanker) -> FeatureRanker {
    let mut new = FeatureRanker {
        window_score: a.window_score + rand::random::<f32>() / 2.0 - 0.25,
        triad_score: a.triad_score + rand::random::<f32>() / 2.0 - 0.25,
        slot_score: a.slot_score + rand::random::<f32>() / 2.0 - 0.25,
        double_score: a.double_score + rand::random::<f32>() /2.0 - 0.25,
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

fn round(round_id: u32, a: FeatureRankerAi, b: FeatureRankerAi, rec_depth: usize) -> (FeatureRankerAi, FeatureRankerAi) {
    let c = RankerAi::new(breed(&a.ranker, &b.ranker), rec_depth);
    let d = RankerAi::new(breed(&b.ranker, &a.ranker), rec_depth);
    let e = RankerAi::new(randomize(&a.ranker), rec_depth);
    let f = RankerAi::new(randomize(&b.ranker), rec_depth);
    let g = random_ai(rec_depth);
    let h = random_ai(rec_depth);

    let inset: Vec<_> = vec![a, b, c, d, e, f, g, h].into_iter().map(RefCell::new).collect();
    let mut outset: Vec<_> = (0 .. inset.len()).map(|i| (0, i)).collect();

    let mut pbr = pbr::ProgressBar::new((inset.len() * inset.len() - inset.len()) as u64);

    for (i, ai_i) in inset.iter().enumerate() {
        for (k, ai_k) in inset.iter().enumerate() {
            if i == k { continue }
            pbr.inc();
            let mut ai_i = ai_i.borrow_mut();
            let mut ai_k = ai_k.borrow_mut();

            println!("\nnew game");
            match play_game(&mut *ai_i, &mut *ai_k) {
                GameResult::Player1 => {
                    outset[i].0 += 1;
                    outset[k].0 -= 1;
                }
                GameResult::Player2 => {
                    outset[i].0 -= 1;
                    outset[k].0 += 1;
                }
                GameResult::Tie => {}
            }

            // invert order
            println!("new game");
            match play_game(&mut *ai_k, &mut *ai_i) {
                GameResult::Player1 => {
                    outset[i].0 -= 1;
                    outset[k].0 += 1;
                }
                GameResult::Player2 => {
                    outset[i].0 += 1;
                    outset[k].0 -= 1;
                }
                GameResult::Tie => {}
            }
        }
    }

    (&mut outset[..]).sort_by_key(|&(wins, _)| wins);

    let best = &inset[outset[0].1];
    let next_best = &inset[outset[1].1];
    let best = &*best.borrow();
    let next_best = &*next_best.borrow();
    (best.clone(), next_best.clone())
}

const REC_DEPTH: usize = 4;
fn main() {
    let mut a = random_ai(REC_DEPTH);
    let mut b = random_ai(REC_DEPTH);
    for i in 0 .. {
        let (ar, br) = round(i, a, b, REC_DEPTH);
        a = ar;
        b = br;

        println!("ROUND: {}", i);
        println!("BEST: {:#?}", a);
        println!("NEXT: {:#?}", b);
    }
}
