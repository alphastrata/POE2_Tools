// const LVL_CAP: usize = 35;
// // 100 should be possible of 31 lvls..
// const MIN_BONUS_VALUE: f32 = 101.0;

// tree.take_while(start_node, |s| s.as_str().contains(keyword), LVL_CAP),
// tree.par_take_while(start_node, |s| s.as_str().contains(keyword), LVL_CAP),

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poe_tree::{quick_tree, stats::Stat, type_wrappings::NodeId};

const LVL_CAP: usize = 15;
// const MIN_BONUS_VALUE: f32 = 100.0;
const STARTING_LOC: NodeId = 3936; //warrior melee damage.

fn bench_take_while(c: &mut Criterion) {
    let tree = quick_tree();
    let selector = |s: &Stat| matches!(s, Stat::MeleeDamage(_));

    c.bench_function("take_while_contains_MeleeDamage", |b| {
        b.iter(|| {
            let result = tree.take_while(STARTING_LOC, selector, LVL_CAP);
            assert!(!result.is_empty());
            black_box(&result);
        })
    });
}

fn bench_par_take_while(c: &mut Criterion) {
    let tree = quick_tree();
    let selector = |s: &Stat| matches!(s, Stat::MeleeDamage(_));

    c.bench_function("par_take_while_contains_MeleeDamage", |b| {
        b.iter(|| {
            let result = tree.par_take_while(STARTING_LOC, selector, LVL_CAP);
            assert!(!result.is_empty());
            black_box(&result);
        })
    });
}

fn bench_take_while_many_selection(c: &mut Criterion) {
    let tree = quick_tree();
    let selector = |s: &Stat| {
        matches!(
            s,
            Stat::MeleeDamage(_)
                | Stat::PhysicalDamage(_)
                | Stat::AttackDamage(_)
                | Stat::MeleeDamageAtCloseRange(_)
        )
    };

    c.bench_function("take_while_many_selections", |b| {
        b.iter(|| {
            let result = tree.take_while(STARTING_LOC, selector, LVL_CAP);
            assert!(!result.is_empty());
            black_box(&result);
        })
    });
}

fn bench_par_take_while_many_selection(c: &mut Criterion) {
    let tree = quick_tree();
    let selector = |s: &Stat| {
        matches!(
            s,
            Stat::MeleeDamage(_)
                | Stat::PhysicalDamage(_)
                | Stat::AttackDamage(_)
                | Stat::MeleeDamageAtCloseRange(_)
        )
    };

    c.bench_function("par_take_while_many_selections", |b| {
        b.iter(|| {
            let result = tree.par_take_while(STARTING_LOC, selector, LVL_CAP);
            assert!(!result.is_empty());
            black_box(&result);
        })
    });
}

criterion_group!(
    benches,
    bench_take_while,
    bench_par_take_while,
    bench_par_take_while_many_selection,
    bench_take_while_many_selection
);
criterion_main!(benches);

/*
I think it's probably this @ 110 for 11 lvls.

[55473, 46325, 33556, 43164, 58528, 64284, 3936, 19011, 5710, 45363]

*/
