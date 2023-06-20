
// reminder: make the arrays or the conspiracy_counter json or something
// `optional` means `nullable`

// TABLE Run config:
// id, max_search_depth, algorithm_used, conspiracy_search_used, bucket_size, num_buckets, conspiracy_merge_fn, transposition_table_used, minimum_transposition_depth, timestamp

// TABLE Run starts:
// id, foreign key Run config, uci_position (e.g. `startpos moves b1c3`), opening_name (optional), timestamp,

// TABLE Position search:
// id, foreign key Run starts, uci position, depth, time_taken, nodes_evaluated, evaluation, conspiracy_counter (optional), timestamp

// TABLE MT search:
// id, foreign key Position search, test_value, time_taken, nodes_evaluated, eval_boundary_type, evaluation, conspiracy_counter (optional), timestamp


// TODO: make conspiracy_search work with TT when researching at lower depths
// when playing a game where you already searched at a given depth on a previous turn: keep that conspiracy_search around!
// this is important for determining the test_values
