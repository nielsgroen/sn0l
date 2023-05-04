use chess::{Board, ChessMove, Color, EMPTY, MoveGen};
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::{is_default_move, score};
use crate::core::score::score_tables::{piece_value};
use crate::core::search::transpositions::TranspositionTable;



pub fn order_moves(
    board: &Board,
    current_evaluation: BoardEvaluation,
    // transposition_table: &mut impl TranspositionTable,
    already_found_move: Option<ChessMove>,
    mut move_generator: &mut MoveGen,
    captures_only: bool,
) -> Vec<ChessMove> {
    // First: Check if a best move exists

    // let search_info = transposition_table.get_transposition(board, None);

    let mut best_move = None;
    if let Some(m) = already_found_move {
        if !is_default_move(&m) { // In the case an ended game (e.g. checkmate) ends up in the Transposition Table
            best_move = already_found_move;
        }
    }
    // if let Some(s) = search_info {
    //     if !is_default_move(&s.best_move) { // In the case an ended game (e.g. checkmate) ends up in the Transposition Table)
    //         best_move = Some(s.best_move);
    //     } else {
    //         best_move = None;
    //     }
    // } else {
    //     best_move = None;
    // }

    let our_color = board.side_to_move();
    let mut capture_moves: Vec<(ChessMove, Centipawns)> = match best_move {
        Some(b) => vec![(b, Centipawns(1_000_000))], // must be the best move when reordering
        _ => vec![],
    };

    // Make extra sure we're only looking at the capture moves
    move_generator.set_iterator_mask(*board.color_combined(!board.side_to_move()));

    for chess_move in &mut move_generator {
        if Some(chess_move) != best_move { // best move already in the ordering
            let source_piece = board.piece_on(chess_move.get_source()).expect("capture move has no source piece");
            let target_piece = board.piece_on(chess_move.get_dest()).expect("capture move has no captured piece");
            let promotion = chess_move.get_promotion();

            let mut chess_move_score = match promotion {
                Some(promo) => score::piece_value(target_piece) - score::piece_value(source_piece) + score::piece_value(promo) + Centipawns::new(-1),
                _ => score::piece_value(target_piece) - score::piece_value(source_piece),
            };

            // Need to know the running score to properly compare with the positions that
            // are already in the transposition table
            capture_moves.push((
                chess_move,
                chess_move_score,
            ));
        }
    }

    capture_moves.sort_by_key(|(_, a)| *a);
    let mut capture_moves_new;
    if our_color == Color::White {
        capture_moves_new = capture_moves.into_iter().map(|(x, _)| x).collect();
    } else {
        capture_moves_new = capture_moves.into_iter().map(|(x, _)| x).rev().collect();
    }

    if captures_only {
        return capture_moves_new;
    }

    // Now: ordering the non captures
    let mut non_capture_moves: Vec<(ChessMove, Centipawns)> = vec![];

    // Make extra sure we're looking at all left over moves
    move_generator.set_iterator_mask(!EMPTY);

    for chess_move in &mut move_generator {
        if Some(chess_move) != best_move { // best move already in the ordering
            let source_square = chess_move.get_source();
            let piece = board.piece_on(source_square).expect("move has no source piece");
            let source_score = piece_value(our_color, piece, source_square.to_index());

            let target_square = chess_move.get_dest();
            let target_piece = match chess_move.get_promotion() {
                Some(promo) => promo,
                _ => piece,
            };
            let target_score = piece_value(our_color, target_piece, target_square.to_index());

            let chess_move_score = Centipawns::new(target_score as i64 - source_score as i64);

            non_capture_moves.push((
                chess_move,
                chess_move_score,
            ));
        }
    }

    non_capture_moves.sort_by_key(|(_, a)| *a);
    let mut non_capture_moves_new;
    if our_color == Color::White {
        non_capture_moves_new = non_capture_moves.into_iter().map(|(x, _)| x).collect();
    } else {
        non_capture_moves_new = non_capture_moves.into_iter().map(|(x, _)| x).rev().collect();
    }

    capture_moves_new.append(&mut non_capture_moves_new);

    capture_moves_new
}

// pub fn order_captures(
//     board: &Board,
//     current_evaluation: BoardEvaluation,
//     transposition_table: &mut impl TranspositionTable,
//     move_generator: &mut MoveGen,
// ) -> Vec<(ChessMove, BoardEvaluation)> {
//     // Order by `val captee` - `val capturer` (+ `val promotion`)
//
//     let our_color = board.side_to_move();
//
//     // Make extra sure we're only looking at the capture moves
//     move_generator.set_iterator_mask(*board.color_combined(!board.side_to_move()));
//
//     let mut moves: Vec<(ChessMove, BoardEvaluation)> = Vec::new();
//
//     let mut board_new = board.clone();
//
//     for chess_move in move_generator {
//         board.make_move(chess_move, &mut board_new);
//
//         if let Some(chess_move_info) = transposition_table.get_transposition(&board_new, None) {
//             moves.push((chess_move, chess_move_info.evaluation));
//         } else {
//             // note: this ordering is not based on incremental static evaluation
//             // but instead on `val captee` - `val capturer`
//             // Since, it may generally be desirable to take with the lesser piece first.
//             let source_piece = board.piece_on(chess_move.get_source()).expect("capture move has no source piece");
//             let target_piece = board.piece_on(chess_move.get_dest()).expect("capture move has no captured piece");
//             let promotion = chess_move.get_promotion();
//
//             let mut chess_move_score = match promotion {
//                 Some(promo) => score::piece_value(target_piece) - score::piece_value(source_piece) + score::piece_value(promo) + Centipawns::new(-1),
//                 _ => score::piece_value(target_piece) - score::piece_value(source_piece),
//             };
//
//             // Need to know the running score to properly compare with the positions that
//             // are already in the transposition table
//             moves.push((
//                 chess_move,
//                 match current_evaluation {
//                     BoardEvaluation::PieceScore(x) => BoardEvaluation::PieceScore(x + chess_move_score),
//                     // This is a degeneracy in move ordering
//                     // if the parent board position is a "mate in n" => move ordering is disabled for positions not in Transposition table
//                     // All boards not in Transposition table are irrelevant since mate is already guaranteed
//                     // => Return worst possible eval => These boards (should) get pruned
//                     _ => match board.side_to_move() {
//                         Color::White => BoardEvaluation::BlackMate(0),
//                         Color::Black => BoardEvaluation::WhiteMate(0),
//                     }
//                 },
//             ));
//         }
//     }
//
//     moves.sort_by_key(|(_, a)| *a);
//
//     match our_color { // Make sure to reverse order if black is making the move
//         Color::White => moves,
//         Color::Black => moves.into_iter().rev().collect(),
//     }
// }
//
// /// Tries to optimally the non-capture moves
// /// Assumes captures have already been run exhausted from the MoveGen
// pub fn order_non_captures(
//     board: &Board,
//     current_evaluation: BoardEvaluation,
//     transposition_table: &mut impl TranspositionTable,
//     move_generator: &mut MoveGen,
// ) -> Vec<(ChessMove, BoardEvaluation)> {
//     let our_color = board.side_to_move();
//
//     // Make extra sure we're looking at all left over moves
//     move_generator.set_iterator_mask(!EMPTY);
//
//     let mut moves: Vec<(ChessMove, BoardEvaluation)> = Vec::new();
//
//     let mut board_new = board.clone();
//
//     for chess_move in move_generator {
//         board.make_move(chess_move, &mut board_new);
//
//         if let Some(chess_move_info) = transposition_table.get_transposition(&board_new, None) {
//             moves.push((chess_move, chess_move_info.evaluation));
//         } else {
//             let source_square = chess_move.get_source();
//             let piece = board.piece_on(source_square).expect("move has no source piece");
//             let source_score = piece_value(our_color, piece, source_square.to_index());
//
//             let target_square = chess_move.get_dest();
//             let target_piece = match chess_move.get_promotion() {
//                 Some(promo) => promo,
//                 _ => piece,
//             };
//             let target_score = piece_value(our_color, target_piece, target_square.to_index());
//
//             // Need to know the running score to properly compare with the positions that
//             // are already in the transposition table
//             moves.push((
//                 chess_move,
//                 match current_evaluation {
//                     BoardEvaluation::PieceScore(x) => BoardEvaluation::PieceScore(x + Centipawns::new(target_score as i64 - source_score as i64)),
//                     // This is a degeneracy in move ordering
//                     // if the parent board position is a "mate in n" => move ordering is disabled for positions not in Transposition table
//                     // All boards not in Transposition table are irrelevant since mate is already guaranteed
//                     // => Return worst possible eval => These boards (should) get pruned
//                     _ => match board.side_to_move() {
//                         Color::White => BoardEvaluation::BlackMate(0),
//                         Color::Black => BoardEvaluation::WhiteMate(0),
//                     }
//                 },
//             ));
//         }
//     }
//
//     moves.sort_by_key(|(_, a)| *a);
//
//     match our_color { // Make sure to reverse order if black is making the move
//         Color::White => moves,
//         Color::Black => moves.into_iter().rev().collect(),
//     }
// }
