use chess::{Board, ChessMove, Color, MoveGen};
use crate::core::evaluation::{bubble_evaluation, game_status};
use crate::core::evaluation::incremental::incremental_evaluation;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::common::check_game_over;
use crate::core::search::move_ordering::order_moves;
use crate::core::search::search_result::SearchResult;
use crate::core::search::SearchDepth;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};

/// MT: The MT in MTD, meaning Memory-Enhanced Test
/// An alteration of Pearl's Test with memory (a transposition table)


pub fn search_mt<T: SearchResult + Default + Clone> (
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    mut visited_boards: Vec<u64>,
    simple_evaluation: Centipawns,
    test_value: EvalBound, // The value to test
    current_depth: u32,
    max_depth: u32,
    // max_selective_depth: u32,
) -> T {
    let current_evaluation = BoardEvaluation::PieceScore(simple_evaluation);
    let mut nodes_searched: u32 = 1;

    let mut move_gen = MoveGen::new_legal(board);
    let board_status = game_status(board, move_gen.len() != 0);

    visited_boards.push(board.get_hash());
    if let Some(search_result) = check_game_over(board, board_status, &visited_boards) {
        return search_result;
    }

    let mut transposition_move = None;
    if let Some(solution) = transposition_table.get_transposition(
        board,
        Some(SearchDepth::Depth(max_depth - current_depth)),
    ) {
        transposition_move = Some(solution.best_move);

        if solution.evaluation > test_value || solution.evaluation < test_value {
            return T::make_search_result(
                solution.best_move,
                solution.evaluation,
                Some(1),
                None,
            )
        }
    }

    if current_depth >= max_depth {
        // TODO: if want to add in quiescence search add that in
        return T::make_search_result(
            ChessMove::default(),
            EvalBound::Exact(current_evaluation),
            None,
            None
        );
    }

    let all_moves = order_moves(
        board,
        transposition_move,
        &mut move_gen,
        false,
    );

    let mut best_eval: EvalBound;
    if board.side_to_move() == Color::White {
        best_eval = EvalBound::Exact(BoardEvaluation::BlackMate(0));
    } else {
        best_eval = EvalBound::Exact(BoardEvaluation::WhiteMate(0));
    }

    let mut best_move = ChessMove::default();
    let mut best_search_result= T::default();
    for chess_move in all_moves.into_iter() {
        let new_board = &board.make_move_new(chess_move);
        let improvement = incremental_evaluation(
            &board,
            &chess_move,
            board.side_to_move(),
        );

        if board.side_to_move() == Color::White {
            let search_result: T = search_mt(
                new_board,
                transposition_table,
                visited_boards.clone(),
                simple_evaluation + improvement,  // + because white
                test_value,
                current_depth + 1,
                max_depth,
                // max_selective_depth,
            );
            best_eval.set_board_evaluation(bubble_evaluation(best_eval.board_evaluation()));

            if search_result.board_evaluation() >= best_eval {
                best_eval = search_result.board_evaluation();
                best_move = chess_move;
                // best_path = search_result.critical_path();
                best_search_result = search_result.clone();
            }
            nodes_searched += search_result.nodes_searched().unwrap_or(1);

            if search_result.board_evaluation() > test_value {
                best_search_result.prepend_move(chess_move);
                let eval_bound = EvalBound::LowerBound(best_eval.board_evaluation());

                transposition_table.update(
                    board,
                    SearchDepth::Depth(max_depth - current_depth),
                    eval_bound,
                    best_move,
                );

                return T::make_search_result(
                    best_move,
                    eval_bound,
                    Some(nodes_searched),
                    best_search_result.critical_path(),
                );
            }
        } else { // Black to move
            let search_result: T = search_mt(
                new_board,
                transposition_table,
                visited_boards.clone(),
                simple_evaluation - improvement,  // - because black
                test_value,
                current_depth + 1,
                max_depth,
                // max_selective_depth,
            );
            best_eval.set_board_evaluation(bubble_evaluation(best_eval.board_evaluation()));

            if search_result.board_evaluation() <= best_eval {
                best_eval = search_result.board_evaluation();
                best_move = chess_move;
                // best_path = search_result.critical_path();
                best_search_result = search_result.clone();
            }
            nodes_searched += search_result.nodes_searched().unwrap_or(1);

            if search_result.board_evaluation() < test_value {
                best_search_result.prepend_move(chess_move);
                let eval_bound = EvalBound::UpperBound(best_eval.board_evaluation());

                transposition_table.update(
                    board,
                    SearchDepth::Depth(max_depth - current_depth),
                    eval_bound,
                    best_move,
                );

                return T::make_search_result(
                    best_move,
                    eval_bound,
                    Some(nodes_searched),
                    best_search_result.critical_path(),
                );
            }
        }
    }


    // TODO: rework set eval_bound, update TT, return exact result
    // let eval_bound = EvalBound::Exact()


    todo!()
}

