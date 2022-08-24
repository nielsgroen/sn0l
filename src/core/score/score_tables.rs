use chess;
use chess::{Board, Color, Piece, Square};

use crate::core::score::Centipawns;


#[rustfmt::skip]
pub const PAWN_TABLES: [[u64; 64]; 2] = [
    // First item is bottom left of board: corresponds to each bit in a BitBoard
    // So board is upside down, but left side is still left
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        100, 100, 90, 50, 50, 100, 100, 100,
        100, 105, 110, 95, 95, 90, 105, 110,
        105, 110, 118, 130, 130, 110, 105, 110,
        110, 115, 126, 145, 145, 120, 118, 120,
        125, 125, 150, 170, 170, 150, 150, 150,
        250, 250, 290, 300, 300, 290, 250, 250,
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        250, 250, 290, 300, 300, 290, 250, 250,
        125, 125, 150, 170, 170, 150, 150, 150,
        110, 115, 126, 145, 145, 120, 118, 120,
        105, 110, 118, 130, 130, 110, 105, 110,
        100, 105, 110, 95, 95, 90, 105, 110,
        100, 100, 90, 50, 50, 100, 100, 100,
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
];

#[rustfmt::skip]
pub const KNIGHT_TABLES: [[u64; 64]; 2] = [
    [
        250, 280, 290, 290, 290, 290, 280, 250,
        260, 270, 280, 290, 290, 280, 270, 260,
        265, 305, 316, 311, 311, 316, 305, 265,
        288, 308, 325, 320, 320, 325, 308, 288,
        288, 308, 325, 322, 322, 325, 308, 288,
        280, 303, 326, 335, 335, 326, 303, 280,
        275, 290, 325, 330, 330, 325, 290, 275,
        250, 270, 282, 295, 295, 282, 270, 250,
    ],
    [
        250, 270, 282, 295, 295, 282, 270, 250,
        275, 290, 325, 330, 330, 325, 290, 275,
        280, 303, 326, 335, 335, 326, 303, 280,
        288, 308, 325, 322, 322, 325, 308, 288,
        288, 308, 325, 320, 320, 325, 308, 288,
        265, 305, 316, 311, 311, 316, 305, 265,
        260, 270, 280, 290, 290, 280, 270, 260,
        250, 280, 290, 290, 290, 290, 280, 250,
    ]
    // [
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    // ],
    // [
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    //     300, 300, 300, 300, 300, 300, 300, 300,
    // ]
];

#[rustfmt::skip]
pub const BISHOP_TABLES: [[u64; 64]; 2] = [
    [
        270, 290, 250, 260, 260, 250, 290, 270,
        300, 325, 302, 300, 300, 302, 325, 300,
        300, 320, 305, 302, 302, 305, 320, 300,
        300, 300, 325, 332, 332, 325, 300, 300,
        300, 304, 325, 335, 335, 325, 304, 300,
        290, 295, 300, 310, 310, 300, 295, 290,
        290, 280, 290, 290, 290, 290, 280, 290,
        290, 275, 280, 290, 290, 280, 275, 290,
    ],
    [
        290, 275, 280, 290, 290, 280, 275, 290,
        290, 280, 290, 290, 290, 290, 280, 290,
        290, 295, 300, 310, 310, 300, 295, 290,
        300, 304, 315, 325, 325, 315, 304, 300,
        300, 300, 315, 322, 322, 315, 300, 300,
        300, 320, 305, 302, 302, 305, 320, 300,
        300, 320, 302, 300, 300, 302, 320, 300,
        270, 290, 250, 260, 260, 250, 290, 270,
    ]
];

#[rustfmt::skip]
pub const ROOK_TABLES: [[u64; 64]; 2] = [
    [
        460, 480, 480, 518, 518, 480, 480, 460,
        460, 475, 505, 512, 512, 505, 475, 460,
        460, 475, 505, 512, 512, 505, 475, 460,
        490, 475, 505, 512, 512, 505, 475, 490,
        490, 475, 505, 512, 512, 505, 475, 490,
        490, 475, 505, 512, 512, 505, 475, 490,
        540, 540, 540, 540, 540, 540, 540, 540,
        540, 540, 540, 540, 540, 540, 540, 540,
    ],
    [
        540, 540, 540, 540, 540, 540, 540, 540,
        540, 540, 540, 540, 540, 540, 540, 540,
        490, 475, 505, 512, 512, 505, 475, 490,
        490, 475, 505, 512, 512, 505, 475, 490,
        490, 475, 505, 512, 512, 505, 475, 490,
        460, 475, 505, 512, 512, 505, 475, 460,
        460, 475, 505, 512, 512, 505, 475, 460,
        460, 480, 480, 518, 518, 480, 480, 460,
    ]
];

#[rustfmt::skip]
pub const QUEEN_TABLES: [[u64; 64]; 2] = [
    [
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
    ],
    [
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
        900, 900, 900, 900, 900, 900, 900, 900,
    ]
];

// Relative value only
#[rustfmt::skip]
pub const KING_TABLES: [[u64; 64]; 2] = [
    [
        155, 155, 130, 110, 110, 110, 155, 155,
        145, 050, 000, 000, 000, 000, 050, 145,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
    ],
    [
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        000, 000, 000, 000, 000, 000, 000, 000,
        145, 050, 000, 000, 000, 000, 050, 145,
        155, 155, 130, 110, 110, 110, 155, 155,
    ]
];

pub fn piece_table(color: Color, piece: Piece) -> [u64; 64] {
    match (color, piece) {
        (Color::White, Piece::Pawn) => PAWN_TABLES[0],
        (Color::Black, Piece::Pawn) => PAWN_TABLES[1],
        (Color::White, Piece::Knight) => KNIGHT_TABLES[0],
        (Color::Black, Piece::Knight) => KNIGHT_TABLES[1],
        (Color::White, Piece::Bishop) => BISHOP_TABLES[0],
        (Color::Black, Piece::Bishop) => BISHOP_TABLES[1],
        (Color::White, Piece::Rook) => ROOK_TABLES[0],
        (Color::Black, Piece::Rook) => ROOK_TABLES[1],
        (Color::White, Piece::Queen) => QUEEN_TABLES[0],
        (Color::Black, Piece::Queen) => QUEEN_TABLES[1],
        (Color::White, Piece::King) => KING_TABLES[0],
        (Color::Black, Piece::King) => KING_TABLES[1],
    }
}

pub fn color_pawn_table(color: chess::Color) -> [u64; 64] {
    match color {
        Color::White => PAWN_TABLES[0],
        Color::Black => PAWN_TABLES[1],
    }
}


pub fn determine_piece_score(square: Square, color: chess::Color, piece: chess::Piece) -> Centipawns {
    Centipawns::new(piece_table(color, piece)[square])
}