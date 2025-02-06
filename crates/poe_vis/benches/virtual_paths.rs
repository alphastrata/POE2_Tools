use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poe_tree::{type_wrappings::NodeId, PassiveTree};
use poe_vis::{components::EdgeMarker, resources::VirtualPath};

const SEARCH_FOR_NODES: ([NodeId; 10], [NodeId; 10], [NodeId; 9], [NodeId; 7]) = (
    [
        10364, 42857, 20024, 44223, 49220, 36778, 36479, 12925, 61196, 58329,
    ],
    [
        10364, 42857, 20024, 44223, 49220, 14725, 34233, 32545, 61196, 58329,
    ],
    [10364, 42857, 20024, 44223, 49220, 53960, 8975, 61196, 58329],
    [10364, 55342, 17248, 53960, 8975, 61196, 58329],
);

const SEARCH_FOR_EDGES: (
    [(NodeId, NodeId); 10],
    [(NodeId, NodeId); 10],
    [(NodeId, NodeId); 9],
    [(NodeId, NodeId); 7],
) = (
    [
        (10364, 42857),
        (20024, 44223),
        (49220, 36778),
        (36479, 12925),
        (61196, 58329),
        (49220, 10364),
        (20024, 58329),
        (61196, 36479),
        (36778, 49220),
        (58329, 12925),
    ],
    [
        (10364, 42857),
        (20024, 44223),
        (49220, 14725),
        (34233, 32545),
        (61196, 58329),
        (49220, 10364),
        (14725, 58329),
        (61196, 34233),
        (20024, 49220),
        (32545, 12925),
    ],
    [
        (10364, 42857),
        (20024, 44223),
        (49220, 53960),
        (8975, 61196),
        (58329, 49220),
        (10364, 53960),
        (20024, 58329),
        (61196, 8975),
        (49220, 20024),
    ],
    [
        (10364, 55342),
        (17248, 53960),
        (8975, 61196),
        (58329, 49220),
        (10364, 53960),
        (20024, 58329),
        (61196, 55342),
    ],
);

fn bench_virtual_path(c: &mut Criterion, label: &str, sorted: bool) {
    let tree = poe_tree::quick_tree();
    let mut vp = VirtualPath {
        nodes: tree.nodes.keys().cloned().collect(),
        edges: tree
            .get_edges()
            .into_iter()
            .map(|(start, end)| EdgeMarker(start, end))
            .collect(),
    };

    if sorted {
        vp.sort();
    }

    c.bench_function(&format!("virtual path contains_node ({})", label), |b| {
        b.iter(|| {
            for &search_node in SEARCH_FOR_NODES.0.iter() {
                black_box(vp.contains_node(search_node));
            }
        });
    });

    c.bench_function(&format!("virtual path contains_edge ({})", label), |b| {
        b.iter(|| {
            SEARCH_FOR_EDGES.0.iter().for_each(|&search_edge| {
                let edg = EdgeMarker(search_edge.0, search_edge.1);
                black_box(vp.contains_edge(&edg));
            });
        });
    });
}

fn virtual_path_search(c: &mut Criterion) {
    bench_virtual_path(c, "unsorted", false);
    bench_virtual_path(c, "sorted", true);
}

criterion_group!(benches, virtual_path_search);
criterion_main!(benches);
