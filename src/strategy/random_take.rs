use std::random::{Rng, SystemRng};
use crate::{Board, Move};
use crate::strategy::ChessStrategy;

pub struct RandomTakeStrategy;

impl RandomTakeStrategy {
    pub fn new() -> Self {
        RandomTakeStrategy
    }
}

impl ChessStrategy for RandomTakeStrategy {
    fn select_move(&mut self, board: Board) -> Move {
        let (takes, moves): (Vec<Move>, Vec<Move>) = board
            .movegen()
            .into_iter()
            .partition(|m| m.takes);
        if !takes.is_empty() {
            *select(&takes)
        } else {
            *select(&moves)
        }
    }
}

fn select<T>(selection: &[T]) -> &T {
    let mut idx = [0u8];
    SystemRng.fill_bytes(&mut idx);
    let idx = idx[0] as usize % selection.len();
    &selection[idx]
}