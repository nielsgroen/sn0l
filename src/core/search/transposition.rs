use std::collections::HashMap;
use chess::Board;
use nohash::BuildNoHashHasher;
use crate::core::search::{SearchInfo};

pub type TranspositionTable = HashMap<Board, SearchInfo, BuildNoHashHasher<u64>>;