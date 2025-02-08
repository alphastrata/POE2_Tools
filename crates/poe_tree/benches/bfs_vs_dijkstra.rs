use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poe_tree::{quick_tree, type_wrappings::NodeId};

const DESTINATION: NodeId = 62439; // enraged reaver
const STARTING_LOC: NodeId = 4739; // lhs side starting sorc node
const EXPECTED_LENGTH: usize = 31;

fn bench_bfs(c: &mut Criterion) {
    let tree = quick_tree();
    let path_checks: Vec<(NodeId, NodeId, usize)> = {
        let flow_ids = tree.fuzzy_search_nodes("flow like water");
        let chaos_ids = tree.fuzzy_search_nodes("chaos inoculation");
        vec![(flow_ids[0], chaos_ids[0], EXPECTED_LENGTH)]
    };

    c.bench_function("BFS shortest path", |b| {
        b.iter(|| {
            for &(start, target, expected) in &path_checks {
                let path = tree.find_shortest_path(start, target);
                assert_eq!(path.len(), expected);
                black_box(&path);
            }
        });
    });
}

fn bench_dijkstra(c: &mut Criterion) {
    let tree = quick_tree();
    let path_checks: Vec<(NodeId, NodeId, usize)> = {
        let flow_ids = tree.fuzzy_search_nodes("flow like water");
        let chaos_ids = tree.fuzzy_search_nodes("chaos inoculation");
        vec![(flow_ids[0], chaos_ids[0], EXPECTED_LENGTH)]
    };

    c.bench_function("Dijkstra shortest path", |b| {
        b.iter(|| {
            for &(start, target, expected) in &path_checks {
                let path = tree.dijkstra(start, target);
                assert_eq!(path.len(), expected);
                black_box(&path);
            }
        });
    });
}

criterion_group!(benches, bench_bfs, bench_dijkstra);
criterion_main!(benches);
