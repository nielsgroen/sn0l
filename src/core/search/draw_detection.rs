

/// Returns whether the position has been seen twice before
pub fn detect_draw_incremental(visited_boards: &[u64]) -> bool {
    match visited_boards.last() {
        None => false,
        Some(board_hash) => {
            let mut total = 0u32;
            for visited_board in visited_boards {
                if *visited_board == *board_hash {
                    total += 1;
                }
            }

            total >= 3
        },
    }
}