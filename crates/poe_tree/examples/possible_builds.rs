use poe_tree::{consts::CHAR_START_NODES, edges::Edge, PassiveTree};

fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    PassiveTree::from_value(&tree_data).unwrap()
}

fn main() {
    // how many possible builds (i.e  Paths are there for 'n' number of levels...)
    _ = pretty_env_logger::init();

    let mut tree = quick_tree();

    tree.remove_hidden();

    // let potential_starts = CHAR_START_NODES;

    // let s1 = potential_starts[0];
    let start_node = 10364;
    let steps = 20;
    let paths = tree.walk_n_steps(start_node, steps);
    // let paths = tree.par_walk_n_steps(start_node, steps);

    // Validate that all paths have the correct length
    assert!(!paths.is_empty(), "No paths found!");
    for path in &paths {
        assert_eq!(
            path.len() - 1,
            steps,
            "Path {:?} does not have {} steps",
            path,
            steps
        );
    }

    println!("Num Possible paths: {}, for {} levels.", paths.len(), steps);

    // Validate that all paths follow valid edges
    paths.iter().for_each(|path| {
        // dbg!(path);
        path.windows(2).for_each(|pair| {
            let (from, to) = (pair[0], pair[1]);
            let edge = Edge {
                start: from,
                end: to,
            };
            let reverse_edge = Edge {
                start: to,
                end: from,
            };
            assert!(
                tree.edges.contains(&edge) || tree.edges.contains(&reverse_edge),
                "Invalid edge in path: {:?}",
                path
            );
        });
    });
}
