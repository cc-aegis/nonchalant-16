use std::fmt::{Display, Formatter};
use std::ops::Fn;

/*
RULES:
- like chess, but king takes variant

PIECES:
- king
- wazir: moves one step orthogonally
- ferz: moves one step diagonally
 */

/*
    4 0 1 2 3
    3 4 5 6 7
    2 8 9 a b
    1 c d e f
      A B C D
 */

pub const A1234: u16 = 0x8888;
pub const B1234: u16 = 0x4444;
pub const C1234: u16 = 0x2222;
pub const D1234: u16 = 0x1111;

pub const BCD1234: u16 = 0x7777;
pub const ABC1234: u16 = 0xEEEE;

pub const ABCD1: u16 = 0x000F;
pub const ABCD2: u16 = 0x00F0;
pub const ABCD3: u16 = 0x0F00;
pub const ABCD4: u16 = 0xF000;

pub const ABCD234: u16 = 0xFFF0;
pub const ABCD123: u16 = 0x0FFF;

pub const BCD234: u16 = 0x7770;
pub const BCD123: u16 = 0x0777;
pub const ABC234: u16 = 0xEEE0;
pub const ABC123: u16 = 0x0EEE;

#[derive(Copy, Clone, Debug)]
enum Player {
    White,
    Black,
}

impl Player {
    fn opponent(self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Piece {
    King,
    Wazir,
    Ferz,
}

#[derive(Clone, Copy, Debug)]
struct Pieces {
    kings: u16,
    wazirs: u16,
    ferzes: u16,
    occupied: u16,
}

impl Pieces {
    fn get(&mut self, kind: Piece) -> &mut u16 {
        match kind {
            Piece::King => &mut self.kings,
            Piece::Wazir => &mut self.wazirs,
            Piece::Ferz => &mut self.ferzes,
        }
    }

    #[inline(always)]
    fn apply(&mut self, op: impl Fn(u16) -> u16) {
        self.kings = op(self.kings);
        self.wazirs = op(self.wazirs);
        self.ferzes = op(self.ferzes);
        self.occupied = op(self.occupied);
    }
}

struct PieceIterator {
    state: u16,
}

impl Iterator for PieceIterator {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(highest_one) = self.state.highest_one() else {
            return None;
        };
        let result = 1u16 << highest_one;
        self.state ^= result;
        Some(result)
    }
}

trait IntoPieceIterator {
    fn pieces(&self) -> PieceIterator;
}

impl IntoPieceIterator for u16 {
    fn pieces(&self) -> PieceIterator {
        PieceIterator { state: *self }
    }
}

#[derive(Clone, Debug)]
struct Board {
    white: Pieces,
    black: Pieces,
    occupied: u16,
    turn: Player,
}

#[derive(Copy, Clone, Debug)]
struct Move {
    kind: Piece,
    from: u16,
    to: u16,
    takes: bool,
}

impl Board {
    fn white_move(&self, m: Move) -> Self {
        let mut white = self.white;
        let mut bit_board = white.get(m.kind);
        *bit_board = (*bit_board & !m.from) | m.to;
        let mut black = self.black;
        if m.takes {
            black.apply(|board| board & !m.to);
        }

        Board { white, black, occupied: white.occupied | black.occupied, turn: Player::Black }
    }

    // TODO: const generic color as input
    // TODO: compiler macro for moving (move!(from, (2, -1)))
    // TODO: compiler macro to generate multiple moves
    fn white_king_moves(&self, from: u16) -> impl Iterator<Item = Move> {
        [
            (from & BCD123) << 5,
            (from & ABCD123) << 4,
            (from & ABC123) << 3,
            (from & BCD1234) << 1,
            (from & ABC1234) >> 1,
            (from & BCD234) >> 3,
            (from & ABCD234) >> 4,
            (from & ABC234) >> 5,
        ]
            .into_iter()
            .map(|pos| pos & !self.white.occupied)
            .filter(|&pos| pos != 0)
            .map(move |to| Move {
                kind: Piece::King,
                from,
                to,
                takes: (self.black.occupied & to) != 0,
            })
    }

    fn white_wazir_moves(&self, from: u16) -> impl Iterator<Item = Move> {
        [
            (from & BCD123) << 5,
            (from & ABC123) << 3,
            (from & BCD234) >> 3,
            (from & ABC234) >> 5,
        ]
            .into_iter()
            .map(|pos| pos & !self.white.occupied)
            .filter(|&pos| pos != 0)
            .map(move |to| Move {
                kind: Piece::Ferz,
                from,
                to,
                takes: (self.black.occupied & to) != 0,
            })
    }

    fn white_ferz_moves(&self, from: u16) -> impl Iterator<Item = Move> {
        [
            (from & ABCD123) << 4,
            (from & BCD1234) << 1,
            (from & ABC1234) >> 1,
            (from & ABCD234) >> 4,
        ]
            .into_iter()
            .map(|pos| pos & !self.white.occupied)
            .filter(|&pos| pos != 0)
            .map(move |to| Move {
                kind: Piece::Wazir,
                from,
                to,
                takes: (self.black.occupied & to) != 0,
            })
    }


    fn white_movegen(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        moves.extend(self.white.kings.pieces().flat_map(|piece| self.white_king_moves(piece)));
        moves.extend(self.white.wazirs.pieces().flat_map(|piece| self.white_wazir_moves(piece)));
        moves.extend(self.white.ferzes.pieces().flat_map(|piece| self.white_ferz_moves(piece)));
        moves
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            white: Pieces {
                kings: 0x0008,
                wazirs: 0x0080,
                ferzes: 0x0004,
                occupied: 0b0000_0000_1000_1100,
            },
            black: Pieces {
                kings: 0x1000,
                wazirs: 0x2000,
                ferzes: 0x0100,
                occupied: 0b0011_0001_0000_0000,
            },
            occupied: 0b0011_0001_1000_1100,
            turn: Player::White,
        }
    }
}

fn bitboard_as_idx(bitboard: u16) -> u8 {
    assert_eq!(bitboard.count_ones(), 1);
    let rev_idx = bitboard.highest_one().unwrap();
    15 - rev_idx as u8
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut grid = ['.'; 16];
        self.white.kings.pieces().for_each(|piece| grid[bitboard_as_idx(piece) as usize] = '♚');
        self.white.ferzes.pieces().for_each(|piece| grid[bitboard_as_idx(piece) as usize] = '♝');
        self.white.wazirs.pieces().for_each(|piece| grid[bitboard_as_idx(piece) as usize] = '♜');
        self.black.kings.pieces().for_each(|piece| grid[bitboard_as_idx(piece) as usize] = '♔');
        self.black.ferzes.pieces().for_each(|piece| grid[bitboard_as_idx(piece) as usize] = '♗');
        self.black.wazirs.pieces().for_each(|piece| grid[bitboard_as_idx(piece) as usize] = '♖');
        grid
            .chunks(4)
            .enumerate()
            .try_for_each(|(idx, chunk)| write!(f, "{} | ", 4 - idx)
                .and_then(|_| chunk
                    .iter()
                    .try_for_each(|c| write!(f, "{c} "))
                    .and_then(|_| writeln!(f, ""))
                )
            )
            .and_then(|_| write!(f, "--+--------\n{} | A B C D\n", match self.turn {
                Player::White => '◆',
                Player::Black => '◇',
            }))
    }
}

fn main() {
    let board = Board::default();
    println!("{board}");
    let white_move = board.white_movegen()[0];
    let board = board.white_move(white_move);
    println!("{board}");
}
