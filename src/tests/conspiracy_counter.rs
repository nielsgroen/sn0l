use crate::core::score::{BoardEvaluation, Centipawns};
use anyhow::Result;
use crate::core::search::conspiracy_counter::ConspiracyCounter;

enum TreeNode {
    Leaf(BoardEvaluation),
    MaxNode(Vec<TreeNode>),
    MinNode(Vec<TreeNode>),
}


#[test]
fn check_conspiracy_simple_tree() -> Result<()> {
    let tree = TreeNode::MaxNode(vec![
        TreeNode::MinNode(vec![
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(60))),
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(20))),
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(70))),
        ]),
        TreeNode::MinNode(vec![
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(40))),
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(70))),
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(50))),
        ]),
        TreeNode::MinNode(vec![
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(30))),
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(20))),
            TreeNode::Leaf(BoardEvaluation::PieceScore(Centipawns::new(70))),
        ]),
    ]);

    let result = calculate_conspiracy_counter(&tree, 21, 10);
    println!("{:?}", result);

    assert!(result, ConspiracyCounter {
        bucket_size: 10,
        node_value: Centipawns::PieceScore(Centipawns(40)),
        // middle bucket is [-5, 5).
        // if you want to get over 40, you need to change one conspirator
        // for values over 60 you just need two (the 40 and the first 20)
        // for values over 70 you need to change any three
        up_buckets: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0],
        // if you want to go lower than 40 you just need to change the 40 node
        // if you wanna get lower than 20, you will need to change the two nodes of value 20 as well
        down_buckets: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 0, 0, 0],
    });

    Ok(())
}

fn calculate_conspiracy_counter(node: &TreeNode, num_buckets: usize, bucket_size: u32) -> ConspiracyCounter {
    match node {
        TreeNode::Leaf(x) => ConspiracyCounter::from_leaf(bucket_size, num_buckets, x.clone()),
        TreeNode::MaxNode(children) => {
            let mut counter = calculate_conspiracy_counter(&children[0], num_buckets, bucket_size);

            for child in children.into_iter().skip(1) {
                let other_counter = calculate_conspiracy_counter(child, num_buckets, bucket_size);
                counter.merge_max_node_children(&other_counter);
            }

            counter
        }
        TreeNode::MinNode(children) => {
            let mut counter = calculate_conspiracy_counter(&children[0], num_buckets, bucket_size);

            for child in children.into_iter().skip(1) {
                let other_counter = calculate_conspiracy_counter(child, num_buckets, bucket_size);
                counter.merge_min_node_children(&other_counter);
            }

            counter
        }
    }
}
