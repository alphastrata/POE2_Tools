//$ src\data\mod.rs
pub mod poe_tree;

pub mod prelude {
    pub use poe_tree::PassiveTree;
}

#[cfg(test)]
mod tests {
    use poe_tree::PassiveTree;

    use super::*;

    #[test]
    fn test_path_between_flow_like_water_and_chaos_inoculation() {
        let (mut tree, _value) = PassiveTree::from_file("data/POE2_TREE.json");
        tree.compute_positions_and_stats();

        // Use fuzzy search to find nodes
        let flow_ids = fuzzy_search_nodes(&tree, "flow like water");
        let chaos_ids = fuzzy_search_nodes(&tree, "chaos inoculation");

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
    }

    #[test]
    fn test_bidirectional_edges() {
        let (tree, _value) = PassiveTree::from_file("data/POE2_TREE.json");

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
    fn test_path_between_avatar_of_fire_and_over_exposure() {
        let (tree, _value) = PassiveTree::from_file("data/POE2_TREE.json");

        // Use fuzzy search to find nodes
        let avatar_ids = fuzzy_search_nodes(&tree, "Avatar of Fire");
        let over_exposure_ids = fuzzy_search_nodes(&tree, "Over Exposure");

        assert!(!avatar_ids.is_empty(), "No node found for 'Avatar of Fire'");
        assert!(
            !over_exposure_ids.is_empty(),
            "No node found for 'Over Exposure'"
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
        assert_eq!(path.len(), 95, "Path length mismatch");
    }

    #[test]
    fn test_collect_life_nodes_from_real_tree() {
        let (tree, _value) = PassiveTree::from_file("data/POE2_TREE.json");

        let mut life_nodes = Vec::new();
        let mut total_life = 0.0;

        for node in tree.nodes.values() {
            for stat in node.stats {
                if stat.name.contains("Maximum Life") && matches!(stat.operand, Operand::Add) {
                    life_nodes.push(node.node_id);
                    total_life += stat.value;
                }
            }
        }

        println!(
            "Life Nodes Count: {}, Total Life Added: {}",
            life_nodes.len(),
            total_life
        );

        assert!(!life_nodes.is_empty(), "Expected at least one life node");
        assert!(total_life > 0.0, "Total life should be greater than zero");
    }

    #[test]
    fn test_collect_evasion_percentage_nodes_from_real_tree() {
        let (tree, _value) = PassiveTree::from_file("data/POE2_TREE.json");

        let mut evasion_nodes = Vec::new();
        let mut total_evasion_percent = 0.0;

        for node in tree.nodes.values() {
            for stat in node.stats {
                if stat.name.contains("Evasion") && matches!(stat.operand, Operand::Percentage) {
                    evasion_nodes.push(node.node_id);
                    total_evasion_percent += stat.value;
                }
            }
        }

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
