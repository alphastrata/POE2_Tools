// const LVL_CAP: usize = 35;
// // 100 should be possible of 31 lvls..
// const MIN_BONUS_VALUE: f32 = 101.0;

// tree.take_while(start_node, |s| s.as_str().contains(keyword), LVL_CAP),
// tree.par_take_while(start_node, |s| s.as_str().contains(keyword), LVL_CAP),

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poe_tree::{quick_tree, type_wrappings::NodeId};

const LVL_CAP: usize = 10;
// const MIN_BONUS_VALUE: f32 = 100.0;
const STARTING_LOC: NodeId = 3936; //warrior melee damage.
const KEYWORD: &'static str = "melee_damage_+%"; // filter string

fn bench_take_while(c: &mut Criterion) {
    let tree = quick_tree();
    c.bench_function("take_while_contains_str", |b| {
        b.iter(|| {
            let result = tree.take_while(STARTING_LOC, |s| s.as_str().contains(KEYWORD), LVL_CAP);
            assert!(!result.is_empty());
            black_box(&result);
        })
    });
}

fn bench_par_take_while(c: &mut Criterion) {
    let tree = quick_tree();

    c.bench_function("par_take_while__contains_str", |b| {
        b.iter(|| {
            let result =
                tree.par_take_while(STARTING_LOC, |s| s.as_str().contains(KEYWORD), LVL_CAP);
            assert!(!result.is_empty());
            black_box(&result);
        })
    });
}

criterion_group!(benches, bench_take_while, bench_par_take_while);
criterion_main!(benches);

/*
I think it's probably this @ 120 for 10 lvls.

[55473, 46325, 33556, 43164, 58528, 64284, 3936, 19011, 5710, 45363]

*/
