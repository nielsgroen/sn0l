use chess::{Board, ChessMove, Color, EMPTY, MoveGen};
use crate::core::score::{BoardEvaluation, Centipawns, piece_value};
use crate::core::score::score_tables::piece_table;
use crate::core::search::transposition::TranspositionTable;

pub fn order_captures(
    board: &Board,
    current_evaluation: BoardEvaluation,
    transposition_table: &mut TranspositionTable,
    move_generator: &mut MoveGen,
) -> Vec<ChessMove> {
    // Order by `val captee` - `val capturer` (+ `val promotion`)

    // Make extra sure we're only looking at the capture moves
    move_generator.set_iterator_mask(*board.color_combined(!board.side_to_move()));

    let mut moves: Vec<(ChessMove, BoardEvaluation)> = Vec::new();

    let mut board_new = board.clone();

    for chess_move in move_generator {
        board.make_move(chess_move, &mut board_new);

        if let Some(chess_move_info) = transposition_table.get(&board_new) {
            moves.push((chess_move, chess_move_info.evaluation));
        } else {
            let source_piece = board.piece_on(chess_move.get_source()).expect("capture move has no source piece");
            let target_piece = board.piece_on(chess_move.get_dest()).expect("capture move has no captured piece");
            let promotion = chess_move.get_promotion();

            let mut chess_move_score = match promotion {
                Some(promo) => piece_value(target_piece) - piece_value(source_piece) + piece_value(promo),
                _ => piece_value(target_piece) - piece_value(source_piece),
            };

            // Need to know the running score to properly compare with the positions that
            // are already in the transposition table
            let grounded_piece_score = match current_evaluation {
                BoardEvaluation::WhiteMate(_) => None,
                BoardEvaluation::PieceScore(x) => Some(x),
                BoardEvaluation::BlackMate(_) => None,
            };

            if let Some(score) = grounded_piece_score {
                chess_move_score += score;
            }

            moves.push((chess_move, BoardEvaluation::PieceScore(chess_move_score)));
        }
    }

    moves.sort_by_key(|(_, a)| a);

    match our_color { // Make sure to reverse order if black is making the move
        Color::White => moves.iter().map(|(x, _)| x).collect(),
        Color::Black => moves.iter().map(|(x, _)| x).rev().collect(),
    }
}

/// Tries to optimally the non-capture moves
/// Assumes captures have already been run exhausted from the MoveGen
pub fn order_non_captures(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    move_generator: &mut MoveGen,
) -> Vec<ChessMove> {
    let our_color = board.side_to_move();

    // Make extra sure we're looking at all left over moves
    move_generator.set_iterator_mask(!EMPTY);

    let mut moves: Vec<(ChessMove, BoardEvaluation)>;

    let mut board_new = board.clone();

    for chess_move in move_generator {
        board.make_move(chess_move, &mut board_new);

        if let Some(chess_move_info) = transposition_table.get(&board_new) {
            moves.push((chess_move, chess_move_info.evaluation));
        } else {
            let source_square = chess_move.get_source();
            let piece = board.piece_on(source_square).expect("move has no source piece");
            let source_score = piece_table(our_color, piece)[source_square.to_index()];

            let target_square = chess_move.get_dest();
            let target_piece = match chess_move.get_promotion() {
                Some(promo) => promo,
                _ => piece,
            };
            let target_score = piece_table(our_color, target_piece)[target_square.to_index()];

            // Need to know the running score to properly compare with the positions that
            // are already in the transposition table
            let grounded_piece_score = match current_evaluation {
                BoardEvaluation::WhiteMate(_) => None,
                BoardEvaluation::PieceScore(x) => Some(x),
                BoardEvaluation::BlackMate(_) => None,
            };

            if let Some(grouned_score) = grounded_piece_score {
                moves.push((chess_move, BoardEvaluation::PieceScore(Centipawns::new(grouned_score as i64 + target_score as i64 - source_score))));
            } else {
                moves.push((chess_move, BoardEvaluation::PieceScore(Centipawns::new(target_score as i64 - source_score))));
            }
        }
    }

    moves.sort_by_key(|_, a| a);

    match our_color { // Make sure to reverse order if black is making the move
        Color::White => moves.iter().map(|(x, _)| x).collect(),
        Color::Black => moves.iter().map(|(x, _)| x).rev().collect(),
    }
}