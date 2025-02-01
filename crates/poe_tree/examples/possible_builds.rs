use poe_tree::{consts::get_level_one_nodes, edges::Edge, type_wrappings::NodeId, PassiveTree};
use rayon::prelude::*;
use std::time::Instant;

fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    PassiveTree::from_value(&tree_data).unwrap()
}

fn main() {
    pretty_env_logger::init();
    let mut tree = quick_tree();
    tree.remove_hidden();

    // quick re-collect so we can rayon.
    let nodes: Vec<(&'static str, &[u32; 2])> = get_level_one_nodes()
        .iter()
        .map(|(name, ids)| (*name, ids))
        .collect();

    // St version, split par on starting nodes
    nodes.par_iter().for_each(|(character, node_ids)| {
        let char_start = Instant::now();
        println!("{}:", character);
        node_ids.iter().for_each(|&start_node| {
            println!("\tStart node: {}", start_node);
            for &steps in &[12] {
                let paths = tree.walk_n_steps(start_node, steps);
                assert!(
                    !paths.is_empty(),
                    "No paths found for start {} and {} steps",
                    start_node,
                    steps
                );
                for path in &paths {
                    assert_eq!(path.len() - 1, steps, "Invalid path length in {:?}", path);
                }
                println!("\t\tLevels {}: {} possible paths", steps, paths.len());
                paths.iter().for_each(|path| {
                    path.windows(2).for_each(|pair| {
                        let (from, to) = (pair[0], pair[1]);
                        let edge = Edge {
                            start: from,
                            end: to,
                        };
                        let rev_edge = Edge {
                            start: to,
                            end: from,
                        };
                        assert!(
                            tree.edges.contains(&edge) || tree.edges.contains(&rev_edge),
                            "Invalid edge in path: {:?}",
                            path
                        );
                    });
                });
            }
        });
        println!("{} finished in: {:?}\n", character, char_start.elapsed());
    });

    // WIP:
    // fully par version, walking is par
    //     let a_tree = std::sync::Arc::new(tree);
    //     nodes.iter().for_each(|(character, node_ids)| {
    //         let char_start = Instant::now();
    //         println!("{}:", character);
    //         node_ids.iter().for_each(|&start_node| {
    //             println!("\tStart node: {}", start_node);
    //             for &steps in &[20] {
    //                 let paths = a_tree
    //                     .clone()
    //                     .par_walk_n_steps_use_chains(start_node, steps);
    //                 // assert!(
    //                 //     !paths.is_empty(),
    //                 //     "No paths found for start {} and {} steps",
    //                 //     start_node,
    //                 //     steps
    //                 // );
    //                 for path in &paths {
    //                     assert_eq!(path.len() - 1, steps, "Invalid path length in {:?}", path);
    //                 }
    //                 println!("\t\tLevels {}: {} possible paths", steps, paths.len());
    //                 paths.iter().for_each(|path| {
    //                     path.windows(2).for_each(|pair| {
    //                         let (from, to) = (pair[0], pair[1]);
    //                         let edge = Edge {
    //                             start: from,
    //                             end: to,
    //                         };
    //                         let rev_edge = Edge {
    //                             start: to,
    //                             end: from,
    //                         };
    //                         assert!(
    //                             a_tree.edges.contains(&edge) || a_tree.edges.contains(&rev_edge),
    //                             "Invalid edge in path: {:?}",
    //                             path
    //                         );
    //                     });
    //                 });
    //             }
    //         });
    //         println!(
    //             "par walk for {} finished in: {:?}\n",
    //             character,
    //             char_start.elapsed()
    //         );
    //     });
}
