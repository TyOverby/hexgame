use super::*;
use hexagon::HexPosition;
use hexagon::grid::{Grid, HexGrid, Map};
use ::std::f32::{INFINITY, NEG_INFINITY};

const WIN: f32 = 1000.0;
const LOSS: f32 = -1000.0;
const MOVE_PENALTY: f32 = 0.1;

pub trait Ai {
    fn choose(&mut self, state: &GameState, player: Player) -> HexPosition;
}

pub trait Ranker {
    fn rank(&mut self, state: &GameState, player: Player) -> f32;
}

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


impl <R: Ranker> Ai for RankerAi<R> {
    fn choose(&mut self, state: &GameState, player: Player) -> HexPosition {
        fn eval<R: Ranker>(
            rai: &mut RankerAi<R>,
            state: GameState,
            depth: usize,
            mut alpha: f32,
            mut beta: f32,
            maximizing_player: bool,
            player: Player)
            -> (f32, Option<HexPosition>) {

                if depth == 0 || state.map().is_full() || state.is_game_over() {
                    return (rai.ranker.rank(&state, player) - rai.ranker.rank(&state, player.inverse()), None);
                }

                let available_moves = state.map().grid().iter().filter(|pos| !state.map().contains(pos));

                let mut best_value;
                let mut best_pos = None;
                if maximizing_player {
                    best_value = NEG_INFINITY;
                    for mv in available_moves {
                        let mut state = state.clone();
                        let res = state.make_move(&mv);
                        debug_assert!(res != MoveResult::Bad);

                        let (v, _) = eval(rai, state, depth - 1, alpha, beta, false, player);
                        best_value = best_value.max(v);
                        alpha = alpha.max(best_value);
                        if best_value == v {
                            best_pos = Some(mv);
                        }
                        if beta < alpha {
                            break;
                        }
                    }
                } else {
                    best_value = INFINITY;
                    for mv in available_moves {
                        let mut state = state.clone();
                        let res = state.make_move(&mv);
                        debug_assert!(res != MoveResult::Bad);

                        let (v, _) = eval(rai, state, depth - 1, alpha, beta, true, player);
                        best_value = best_value.min(v);
                        beta = beta.min(best_value);
                        if best_value == v {
                            best_pos = Some(mv);
                        }
                        if beta <= alpha {
                            break;
                        }
                    }
                }

                return (best_value - MOVE_PENALTY, best_pos);
            }

        let rec_lim = self.recursion_limit;
        let mut alpha = NEG_INFINITY;
        let mut beta = INFINITY;
        let (v, p) = eval(self, state.clone(), rec_lim, alpha, beta, true, player);
        println!("{}, {:?}", v, p);
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
        for (piece, _) in map.iter().filter(|&(pos, &p)| player == p) {
            f(piece, map, &mut acc);
        }
        acc / div
    }

    pub fn count_triads(game: &GameState, player: Player) -> u32 {
        FeatureRanker::count_generic(game, player, 2, |pos, map, acc| {
            for i in 0 .. 6 {
                if map.get(&pos.neighbor(i)) == Some(&player) &&
                   map.get(&pos.neighbor(i + 1)) == Some(&player) {
                    *acc += 1;
                } 
            }
        })
    }

    pub fn count_windows(game: &GameState, player: Player) -> u32 {
        FeatureRanker::count_generic(game, player, 2, |pos, map, acc| {
            for mut ray in pos.rays().iter().cloned() {
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
        FeatureRanker::count_generic(game, player, 2, |pos, map, acc| {
            for mut ray in pos.rays().iter().cloned() {
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
