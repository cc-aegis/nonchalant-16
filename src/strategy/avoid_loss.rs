use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Sub;
use std::sync::{LazyLock, Mutex};
use rayon::prelude::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use crate::{Board, Move, Pieces};
use crate::strategy::ChessStrategy;

pub struct AvoidLossStrategy(u8);

struct BoardCache(HashMap<Board, (u8, f32)>);

static CACHE: LazyLock<Mutex<BoardCache>> = LazyLock::new(|| Mutex::new(BoardCache(HashMap::new())));

impl BoardCache {
    fn lookup(&self, board: &Board, search_depth: u8) -> Option<f32> {
        match self.0.get(board) {
            Some((depth, score)) if *depth >= search_depth => Some(*score),
            _ => None,
        }
    }

    fn insert(&mut self, board: Board, search_depth: u8, score: f32) {
        match self.0.entry(board) {
            Entry::Occupied(mut entry) => {
                if entry.get().0 < search_depth {
                    entry.insert((search_depth, score));
                }
            },
            Entry::Vacant(entry) => {
                entry.insert((search_depth, score));
            },
        }
    }
}

impl AvoidLossStrategy {
    pub fn new(depth: u8) -> Self {
        AvoidLossStrategy(depth)
    }
}

impl ChessStrategy for AvoidLossStrategy {
    fn select_move(&mut self, board: Board) -> Move {
        board
            .movegen()
            .into_par_iter()
            .map(|mv| {
                let board = board.apply_move(mv);
                let opponent_score = score_after_perfect_sequence(&board, self.0);
                (opponent_score, mv)
            })
            .min_by(|(a, _), (b, _)| a.total_cmp(b))
            .unwrap()
            .1
    }
}

fn score_after_perfect_sequence(board: &Board, search_depth: u8) -> f32 {
    if search_depth == 0 {
        return score_board(board);
    }

    if let Some(score) = CACHE.lock().unwrap().lookup(board, search_depth) {
        return score;
    }

    let result = board
        .movegen_iter()
        .map(|mv| {
            let board = board.apply_move(mv);
            let opponent_move = board.movegen_iter()
                .map(|mv| {
                    let board = board.apply_move(mv);
                    let score = score_after_perfect_sequence(&board, search_depth - 1);
                    (score, board)
                })
                .min_by(|(a, _), (b, _)| a.total_cmp(b));
            match opponent_move {
                Some((_, board)) => score_after_perfect_sequence(&board, search_depth - 1),
                None => 1000.0,
            }
        })
        .max_by(f32::total_cmp)
        .unwrap_or(-1000.0)
        .sub(0.1); // prefer fast checkmate, long survival

    CACHE.lock().unwrap().insert(board.to_owned(), search_depth, result);
    result
}


fn score_pieces(pieces: &Pieces) -> f32 {
    let kings = pieces.kings.count_ones();
    let wazirs = pieces.wazirs.count_ones();
    let ferzes = pieces.ferzes.count_ones();
    let knights = pieces.knights.count_ones();
    let cannons = pieces.cannons.count_ones();
    let center_king_penalty = (pieces.kings & 0x6FF6).count_ones() + (pieces.kings & 0x0660).count_ones();
    let center_piece_bonus = ((pieces.wazirs | pieces.ferzes) & 0x0660).count_ones();

    if kings == 0 {
        -1000.0
    } else {
        (ferzes * 2 + wazirs * 3 + knights * 2 + cannons * 3 + kings * 6) as f32
            + (center_piece_bonus as f32 * 0.3)
            - (center_king_penalty as f32 * 0.5)
    }
}

fn score_board(board: &Board) -> f32 {
    let curr = board.curr_player();
    let other = board.curr_opponent();

    score_pieces(curr) - score_pieces(other)
}