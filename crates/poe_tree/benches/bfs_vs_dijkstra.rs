use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poe_tree::{quick_tree, type_wrappings::NodeId};

const DESTINATION: NodeId = 62439; // enraged reaver
const STARTING_LOC: NodeId = 4739; // lhs side starting sorc node
const EXPECTED_LENGTH: usize = 31;

fn bench_bfs(c: &mut Criterion) {
    let tree = quick_tree();
    c.bench_function("BFS shortest path", |b| {
        b.iter(|| {
            let path = tree.bfs(STARTING_LOC, DESTINATION);
            assert_eq!(path.len(), EXPECTED_LENGTH);
            assert!(path.contains(&DESTINATION) && path.contains(&STARTING_LOC));
            black_box(&path);
        });
    });
}

fn bench_dijkstra(c: &mut Criterion) {
    let tree = quick_tree();
    c.bench_function("Dijkstra shortest path", |b| {
        b.iter(|| {
            let path = tree.dijkstra(STARTING_LOC, DESTINATION);
            assert_eq!(path.len(), EXPECTED_LENGTH);
            assert!(path.contains(&DESTINATION) && path.contains(&STARTING_LOC));
            black_box(&path);
        });
    });
}

fn bench_bfs_reversed(c: &mut Criterion) {
    let tree = quick_tree();
    c.bench_function("BFS shortest path reversed", |b| {
        b.iter(|| {
            let path = tree.bfs(DESTINATION, STARTING_LOC);
            assert_eq!(path.len(), EXPECTED_LENGTH);
            assert!(path.contains(&DESTINATION) && path.contains(&STARTING_LOC));
            black_box(&path);
        });
    });
}

fn bench_dijkstra_reversed(c: &mut Criterion) {
    let tree = quick_tree();
    c.bench_function("Dijkstra shortest path reversed", |b| {
        b.iter(|| {
            let path = tree.dijkstra(DESTINATION, STARTING_LOC);
            assert_eq!(path.len(), EXPECTED_LENGTH);
            assert!(path.contains(&DESTINATION) && path.contains(&STARTING_LOC));
            black_box(&path);
        });
    });
}

criterion_group!(
    benches,
    bench_bfs,
    bench_dijkstra,
    bench_bfs_reversed,
    bench_dijkstra_reversed
);
criterion_main!(benches);
