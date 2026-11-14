use std::random::{Rng, SystemRng};
use crate::{Board, Move};
use crate::strategy::ChessStrategy;

pub struct RandomMoveStrategy;

impl RandomMoveStrategy {
    pub fn new() -> Self {
        RandomMoveStrategy
    }
}

impl ChessStrategy for RandomMoveStrategy {
    fn select_move(&mut self, board: Board) -> Move {
        *select(&board.movegen())
    }
}

fn select<T>(selection: &[T]) -> &T {
    let mut idx = [0u8];
    SystemRng.fill_bytes(&mut idx);
    let idx = idx[0] as usize % selection.len();
    &selection[idx]
}