//$ src/data/mod.rs
pub mod poe_tree;

pub mod prelude {
    pub use super::poe_tree::PassiveTree;
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::data::poe_tree::{edges::Edge, stats::Operand, PassiveTree};

    #[test]
    fn path_between_flow_like_water_and_chaos_inoculation() {
        let file = File::open("data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();
        let tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

        // Use fuzzy search to find nodes
        let flow_ids = tree.fuzzy_search_nodes("flow like water");
        let chaos_ids = tree.fuzzy_search_nodes("chaos inoculation");

        assert!(!flow_ids.is_empty(), "No node found for 'flow like water'");
        assert!(
            !chaos_ids.is_empty(),
            "No node found for 'chaos inoculation'"
        );

        let start_id = flow_ids[0];
        let target_id = chaos_ids[0];

        // Find shortest path using Dijkstra's Algorithm
        let path = tree.find_shortest_path(start_id, target_id);
        if path.is_empty() {
            println!("No path found between {} and {}", start_id, target_id);
        } else {
            println!("Path found: {:?}", path);
            for node_id in path.iter() {
                let n = tree.nodes.get(node_id).unwrap();
                if !n.name.contains("Attribute") {
                    print!("(ID:{} NAME: {}) ->", node_id, n.name);
                } else {
                    print!("[ID:{}] ->", node_id);
                }
            }
        }
        // Update this value based on expected path length after refactoring
        assert_eq!(path.len(), 15, "Path length mismatch");
        println!("{:#?}", path);
    }

    #[test]
    fn bidirectional_edges() {
        let file = File::open("data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();
        let tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

        for edge in &tree.edges {
            let reverse_edge = Edge {
                from: edge.to,
                to: edge.from,
            };
            assert!(
                tree.edges.contains(&reverse_edge),
                "Edge from {} to {} is not bidirectional",
                edge.to,
                edge.from
            );
        }
        println!("All edges are bidirectional.");
    }

    #[test]
    fn path_between_avatar_of_fire_and_over_exposure() {
        let file = File::open("data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();
        let tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

        // Use fuzzy search to find nodes
        let avatar_ids = tree.fuzzy_search_nodes("Avatar of Fire");
        let over_exposure_ids = tree.fuzzy_search_nodes("Overexposure");

        assert!(!avatar_ids.is_empty(), "No node found for 'Avatar of Fire'");
        assert!(
            !over_exposure_ids.is_empty(),
            "No node found for 'OverExposure'"
        );

        let start_id = avatar_ids[0];
        let target_id = over_exposure_ids[0];

        // Find shortest path using Dijkstra's Algorithm
        let path = tree.find_shortest_path(start_id, target_id);

        if path.is_empty() {
            panic!(
                "No path found between {} and {}",
                tree.nodes[&start_id].name, tree.nodes[&target_id].name
            );
        } else {
            println!("Path found: {:?}", path);
            for node_id in path.iter() {
                let n = tree.nodes.get(node_id).unwrap();
                println!("(ID:{} NAME: {})", node_id, n.name);
            }
        }
        // Update this value based on expected path length after refactoring
        assert_eq!(path.len(), 27, "Path length mismatch");
    }

    #[test]
    fn collect_life_nodes_from_real_tree() {
        let file = File::open("data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();
        let tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

        let mut life_nodes = Vec::new();
        let mut total_life = 0.0;

        tree.nodes.values().for_each(|node| {
            node.stats.iter().for_each(|stat| {
                //NOTE: we should do 'something' to allow any of ["Maximum Life", "max life", 'maximum life', "maximum_life"] to work..
                // maybe a stat.name.supported_patterns_of($needle) -> bool, replacing ' ' with '_' should probs get us 99% of the way there
                if stat.name.contains("maximum_life") && matches!(stat.operand, Operand::Add) {
                    life_nodes.push(node.node_id);
                    total_life += stat.value;
                } else if stat.name.contains("life") {
                    eprintln!("'life' keyword found in {}, maybe we should modify on entry to make nicer..", stat.name);
                }
            });
        });

        println!(
            "Life Nodes Count: {}, Total Life Added: {}",
            life_nodes.len(),
            total_life
        );

        assert!(!life_nodes.is_empty(), "Expected at least one life node");
        assert!(total_life > 0.0, "Total life should be greater than zero");
    }

    #[test]
    fn collect_evasion_percentage_nodes_from_real_tree() {
        let file = File::open("data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();

        let tree = PassiveTree::from_value(&u).unwrap();
        let mut evasion_nodes = Vec::new();
        let mut total_evasion_percent = 0.0;

        tree.nodes.values().for_each(|node| {
            node.stats.iter().for_each(|stat| {
                if stat.name.contains("evasion_rating") && matches!(stat.operand, Operand::Percentage)
                {
                    evasion_nodes.push(node.node_id);
                    total_evasion_percent += stat.value;
                } else if stat.name.contains("evasion") {
                    eprintln!("'evasion' keyword found in {}, maybe we should modify on entry to make nicer..", stat.name);
                }
            });
        });

        println!(
            "Evasion Nodes Count: {}, Total Evasion Percentage: {}",
            evasion_nodes.len(),
            total_evasion_percent
        );

        assert!(
            !evasion_nodes.is_empty(),
            "Expected at least one evasion node"
        );
        assert!(
            total_evasion_percent > 0.0,
            "Total evasion percentage should be greater than zero"
        );
    }
}
