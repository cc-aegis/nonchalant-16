pub mod random_move;
pub mod random_take;

use crate::{Board, Move};

pub trait ChessStrategy {
    fn select_move(&mut self, board: Board) -> Move;
}