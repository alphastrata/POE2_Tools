use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poe_tree::{consts::get_level_one_nodes, quick_tree, type_wrappings::NodeId, PassiveTree};

// WARNING DO NOT GO ABOVE 30 UNLESS YOU HAVE THE RAM FOR IT!
const UPPER_LIMIT: usize = 15;

fn bench_walk_n_steps(
    c: &mut Criterion,
    steps: usize,
    tree: &PassiveTree,
    nodes: &[(&'static str, &[NodeId; 2])],
) {
    for (name, ids) in nodes {
        for &start_node in ids.iter() {
            let bench_name = format!("walk {} steps for {} starting {}", steps, name, start_node);
            c.bench_function(&bench_name, |b| {
                b.iter(|| {
                    //TODO: this will be bad.. we should maybe do a bunch of ranges and then go off that?
                    // let paths = tree.walk_n_steps::<UPPER_LIMIT>(start_node, black_box(steps));
                    let paths = tree.walk_n_steps(start_node, black_box(steps));
                    black_box(&paths);
                });
            });
        }
    }
}

fn bench_all_walks(c: &mut Criterion) {
    let mut tree = quick_tree();
    tree.remove_hidden();

    let nodes: Vec<(&'static str, &[NodeId; 2])> = get_level_one_nodes()
        .iter()
        .map(|(name, ids)| (*name, ids))
        .collect();

    // for steps in (5..=UPPER_LIMIT).step_by(5) {
    bench_walk_n_steps(c, UPPER_LIMIT, &tree, &nodes);
    // }

    bench_csr_walks(c, UPPER_LIMIT, &tree, &nodes)
}

fn bench_csr_walks(
    c: &mut Criterion,
    steps: usize,
    tree: &PassiveTree,
    nodes: &[(&'static str, &[NodeId; 2])],
) {
    for (name, ids) in nodes {
        for &start_node in ids.iter() {
            let bench_name = format!(
                "CSR walk {} steps for {} starting {}",
                steps, name, start_node
            );
            c.bench_function(&bench_name, |b| {
                b.iter(|| {
                    //TODO: this will be bad.. we should maybe do a bunch of ranges and then go off that?
                    // let paths = tree.walk_n_steps::<UPPER_LIMIT>(start_node, black_box(steps));
                    let paths = tree.walk_n_steps_csr::<UPPER_LIMIT>(start_node, black_box(steps));

                    black_box(&paths);
                });
            });
        }
    }
}

criterion_group!(benches, bench_all_walks);
criterion_main!(benches);
