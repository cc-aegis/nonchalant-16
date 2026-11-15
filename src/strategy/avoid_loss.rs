use crate::{Board, Move, Pieces};
use crate::strategy::ChessStrategy;

pub struct AvoidLossStrategy(u8);

impl AvoidLossStrategy {
    pub fn new(depth: u8) -> Self {
        AvoidLossStrategy(depth)
    }
}

impl ChessStrategy for AvoidLossStrategy {
    fn select_move(&mut self, board: Board) -> Move {
        board
            .movegen()
            .into_iter()
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

    let mut scores = Vec::new();

    for mv in board.movegen() {
        let board = board.apply_move(mv);
        let mut opponent_moves = board.movegen()
            .into_iter()
            .map(|mv| {
                let board = board.apply_move(mv);
                let score = score_after_perfect_sequence(&board, search_depth - 1);
                (score, board)
            })
            .collect::<Vec<_>>();
        opponent_moves.sort_by(|(a, _), (b, _)| a.total_cmp(b)); // TODO: use min_by on iterator instead
        let score = match opponent_moves.get(0) {
            Some((_, board)) => score_after_perfect_sequence(board, search_depth - 1),
            None => 1000.0,
        };
        scores.push(score);
    }

    scores.into_iter().max_by(f32::total_cmp).unwrap_or(-1000.0)
}


fn score_pieces(pieces: &Pieces) -> f32 {
    let kings = pieces.kings.count_ones();
    let wazirs = pieces.wazirs.count_ones();
    let ferzes = pieces.ferzes.count_ones();
    let cannons = pieces.cannons.count_ones();
    let center_king_penalty = (pieces.kings & 0x6FF6).count_ones() + (pieces.kings & 0x0660).count_ones();

    if kings == 0 {
        -1000.0
    } else {
        (ferzes * 2 + wazirs * 3 + cannons * 4 + kings * 6) as f32 - (center_king_penalty as f32 * 0.5)
    }
}

fn score_board(board: &Board) -> f32 {
    let curr = board.curr_player();
    let other = board.curr_opponent();

    score_pieces(curr) - score_pieces(other)
}