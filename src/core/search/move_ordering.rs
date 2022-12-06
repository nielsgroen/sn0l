use chess::{Board, ChessMove, Color, EMPTY, MoveGen};
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::score;
use crate::core::score::score_tables::{piece_table, piece_value};
use crate::core::search::transpositions::TranspositionTable;

pub fn order_captures(
    board: &Board,
    current_evaluation: BoardEvaluation,
    transposition_table: &mut impl TranspositionTable,
    move_generator: &mut MoveGen,
) -> Vec<(ChessMove, BoardEvaluation)> {
    // Order by `val captee` - `val capturer` (+ `val promotion`)

    let our_color = board.side_to_move();

    // Make extra sure we're only looking at the capture moves
    move_generator.set_iterator_mask(*board.color_combined(!board.side_to_move()));

    let mut moves: Vec<(ChessMove, BoardEvaluation)> = Vec::new();

    let mut board_new = board.clone();

    for chess_move in move_generator {
        board.make_move(chess_move, &mut board_new);

        if let Some(chess_move_info) = transposition_table.get_transposition(&board_new, None) {
            moves.push((chess_move, chess_move_info.evaluation));
        } else {
            // note: this ordering is not based on incremental static evaluation
            // but instead on `val captee` - `val capturer`
            // Since, it may generally be desirable to take with the lesser piece first.
            let source_piece = board.piece_on(chess_move.get_source()).expect("capture move has no source piece");
            let target_piece = board.piece_on(chess_move.get_dest()).expect("capture move has no captured piece");
            let promotion = chess_move.get_promotion();

            let mut chess_move_score = match promotion {
                Some(promo) => score::piece_value(target_piece) - score::piece_value(source_piece) + score::piece_value(promo) + Centipawns::new(-1),
                _ => score::piece_value(target_piece) - score::piece_value(source_piece),
            };

            // Need to know the running score to properly compare with the positions that
            // are already in the transposition table
            moves.push((
                chess_move,
                match current_evaluation {
                    BoardEvaluation::PieceScore(x) => BoardEvaluation::PieceScore(x + chess_move_score),
                    // This is a degeneracy in move ordering
                    // if the parent board position is a "mate in n" => move ordering is disabled for positions not in Transposition table
                    // All boards not in Transposition table are irrelevant since mate is already guaranteed
                    // => Return worst possible eval => These boards (should) get pruned
                    _ => match board.side_to_move() {
                        Color::White => BoardEvaluation::BlackMate(0),
                        Color::Black => BoardEvaluation::WhiteMate(0),
                    }
                },
            ));
        }
    }

    moves.sort_by_key(|(_, a)| *a);

    match our_color { // Make sure to reverse order if black is making the move
        Color::White => moves,
        Color::Black => moves.into_iter().rev().collect(),
    }
}

/// Tries to optimally the non-capture moves
/// Assumes captures have already been run exhausted from the MoveGen
pub fn order_non_captures(
    board: &Board,
    current_evaluation: BoardEvaluation,
    transposition_table: &mut impl TranspositionTable,
    move_generator: &mut MoveGen,
) -> Vec<(ChessMove, BoardEvaluation)> {
    let our_color = board.side_to_move();

    // Make extra sure we're looking at all left over moves
    move_generator.set_iterator_mask(!EMPTY);

    let mut moves: Vec<(ChessMove, BoardEvaluation)> = Vec::new();

    let mut board_new = board.clone();

    for chess_move in move_generator {
        board.make_move(chess_move, &mut board_new);

        if let Some(chess_move_info) = transposition_table.get_transposition(&board_new, None) {
            moves.push((chess_move, chess_move_info.evaluation));
        } else {
            let source_square = chess_move.get_source();
            let piece = board.piece_on(source_square).expect("move has no source piece");
            let source_score = piece_value(our_color, piece, source_square.to_index());

            let target_square = chess_move.get_dest();
            let target_piece = match chess_move.get_promotion() {
                Some(promo) => promo,
                _ => piece,
            };
            let target_score = piece_value(our_color, target_piece, target_square.to_index());

            // Need to know the running score to properly compare with the positions that
            // are already in the transposition table
            moves.push((
                chess_move,
                match current_evaluation {
                    BoardEvaluation::PieceScore(x) => BoardEvaluation::PieceScore(x + Centipawns::new(target_score as i64 - source_score as i64)),
                    // This is a degeneracy in move ordering
                    // if the parent board position is a "mate in n" => move ordering is disabled for positions not in Transposition table
                    // All boards not in Transposition table are irrelevant since mate is already guaranteed
                    // => Return worst possible eval => These boards (should) get pruned
                    _ => match board.side_to_move() {
                        Color::White => BoardEvaluation::BlackMate(0),
                        Color::Black => BoardEvaluation::WhiteMate(0),
                    }
                },
            ));
        }
    }

    moves.sort_by_key(|(_, a)| *a);

    match our_color { // Make sure to reverse order if black is making the move
        Color::White => moves,
        Color::Black => moves.into_iter().rev().collect(),
    }
}
