use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poe_tree::PassiveTree;
use poe_vis::{components::EdgeMarker, resources::VirtualPath};

fn bench_sort_virtual_path(tree: &PassiveTree) {
    let mut vp = VirtualPath {
        nodes: tree.nodes.keys().cloned().collect(),
        edges: tree
            .get_edges()
            .into_iter()
            .map(|(start, end)| EdgeMarker(start, end))
            .collect(),
    };

    vp.sort()
}

fn criterion_benchmark(c: &mut Criterion) {
    let tree = poe_tree::quick_tree();
    c.bench_function("virtual path sorter", |b| {
        b.iter(|| bench_sort_virtual_path(black_box(&tree)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
