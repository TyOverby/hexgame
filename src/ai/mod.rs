use super::*;
use hexagon::HexPosition;
use hexagon::grid::{Grid, HexGrid, Map};
use ::std::f32::{INFINITY, NEG_INFINITY};

const WIN: f32 = INFINITY;
const LOSS: f32 = NEG_INFINITY;

pub trait Ai {
    fn choose(&mut self, state: &GameState, player: Player) -> HexPosition;
}

pub trait Ranker {
    fn rank(&mut self, state: &GameState, player: Player) -> f32;
}

#[derive(Debug, Clone, Copy)]
pub struct RankerAi<R: Ranker> {
    recursion_limit: usize,
    pub ranker: R
}

impl <R: Ranker> RankerAi<R> {
    pub fn new(ranker: R, recursion_limit: usize) -> RankerAi<R> {
        RankerAi {
            recursion_limit: ::std::cmp::max(recursion_limit, 1),
            ranker: ranker
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
struct Score(f32, i32);

impl ::std::ops::Neg for Score {
    type Output = Score;
    fn neg(self) -> Score {
        Score(-self.0, -self.1)
    }
}

impl <R: Ranker> Ai for RankerAi<R> {
    fn choose(&mut self, state: &GameState, player: Player) -> HexPosition {
        fn eval<R: Ranker>(
            rai: &mut RankerAi<R>,
            state: GameState,
            depth: i32,
            mut alpha: Score,
            beta: Score,
            player: Player)
            -> (Score, Option<HexPosition>) {
                if depth == 0 || state.map().is_full() || state.is_game_over() {
                    let score = Score(rai.ranker.rank(&state, player) - rai.ranker.rank(&state, player.inverse()), -depth);
                    return (score, None);
                }

                let available_moves = state.map().grid().iter().filter(|pos| !state.map().contains(pos));
                let mut best = None;

                for (state, mv) in available_moves.map(|mv| (state.with_move(&mv), mv)) {
                    let (score, _) = eval(rai, state, depth - 1, -beta, -alpha, player.inverse());
                    let score = -score;
                    if score >= beta {
                        best = Some(mv);
                        alpha = beta;
                        break;
                    }
                    if score > alpha {
                        best = Some(mv);
                        alpha = score;
                    }
                }

                return (alpha, best);
            }

        let rec_lim = self.recursion_limit;
        let alpha = NEG_INFINITY;
        let beta = INFINITY;
        let (r, p) = eval(self, state.clone(), rec_lim as i32, Score(alpha, 0), Score(beta, 0), player);

        p.unwrap()
    }
}

pub struct NullRanker;

impl Ranker for NullRanker {
    fn rank(&mut self, state: &GameState, player: Player) -> f32 {
        match state.is_over() {
            MoveResult::End(p) if p == player => WIN,
            MoveResult::End(p) if p != player => LOSS,
            MoveResult::Tie => -100.0,
            _ => 0.0
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FeatureRanker {
    // x  x
    pub window_score: f32,
    // xx
    // x
    pub triad_score: f32,
    // x x
    pub slot_score: f32,
    // xx
    pub double_score: f32,
}

impl FeatureRanker {
    pub fn normalize(&mut self) {
        let total =
            self.window_score +
            self.triad_score +
            self.slot_score +
            self.double_score;

        self.window_score = self.window_score / total;
        self.triad_score = self.triad_score / total;
        self.slot_score = self.slot_score / total;
        self.double_score = self.double_score / total;
    }

    fn count_generic<F>(game: &GameState, player: Player, div: u32, f: F) -> u32
    where F: Fn(HexPosition, &Map<Player, HexGrid>, &mut u32) {
        let map = game.map();
        let mut acc = 0;
        for (piece, _) in map.iter().filter(|&(_, &p)| player == p) {
            f(piece, map, &mut acc);
        }
        acc / div
    }

    pub fn count_triads(game: &GameState, player: Player) -> u32 {
        FeatureRanker::count_generic(game, player, 1, |pos, map, acc| {
            let top = map.get(&pos.neighbor(1)) == Some(&player) &&
                      map.get(&pos.neighbor(2)) == Some(&player);

            let bot = map.get(&pos.neighbor(4)) == Some(&player) &&
                      map.get(&pos.neighbor(5)) == Some(&player);

            if top {
                *acc += 1;
            }

            if bot {
                *acc += 1;
            }

        })
    }

    pub fn count_windows(game: &GameState, player: Player) -> u32 {
        FeatureRanker::count_generic(game, player, 1, |pos, map, acc| {
            for mut ray in pos.rays().iter().cloned().skip(3) {
                ray.next(); // this is ourself
                if map.get(&ray.next().unwrap()) != None {
                    continue;
                }
                if map.get(&ray.next().unwrap()) != None {
                    continue;
                }
                if map.get(&ray.next().unwrap()) == Some(&player) {
                    *acc += 1;
                }
            }
        })
    }

    pub fn count_slots(game: &GameState, player: Player) -> u32 {
        FeatureRanker::count_generic(game, player, 1, |pos, map, acc| {
            for mut ray in pos.rays().iter().cloned().take(3) {
                ray.next(); // this is ourself
                if map.get(&ray.next().unwrap()) != None {
                    continue;
                }
                if map.get(&ray.next().unwrap()) == Some(&player) {
                    *acc += 1;
                }
            }
        })
    }

    pub fn count_doubles(game: &GameState, player: Player) -> u32 {
        FeatureRanker::count_generic(game, player, 1, |pos, map, acc| {
            for mut ray in pos.rays().iter().cloned() {
                ray.next(); // this is ourself
                if map.get(&ray.next().unwrap()) != Some(&player) {
                    continue;
                }
                if map.get(&ray.next().unwrap()) == None {
                    *acc += 1;
                }
            }
        })
    }
}

impl Ranker for FeatureRanker {
    fn rank(&mut self, state: &GameState, player: Player) -> f32 {
        match state.is_over() {
            MoveResult::End(p) if p == player => return WIN,
            MoveResult::End(p) if p != player => return LOSS,
            MoveResult::Tie => return -100.0,
            _ => {  }
        }

        self.triad_score * FeatureRanker::count_triads(state, player) as f32 +
        self.window_score * FeatureRanker::count_windows(state, player) as f32 +
        self.slot_score * FeatureRanker::count_slots(state, player) as f32 +
        self.double_score * FeatureRanker::count_doubles(state, player) as f32
    }
}
