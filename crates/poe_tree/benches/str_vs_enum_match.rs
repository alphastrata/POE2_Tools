use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poe_tree::{quick_tree, stats::Stat, type_wrappings::NodeId};

const KEYWORD: &str = "lightning_damage_+%";

fn bench_string_match(c: &mut Criterion) {
    let tree = quick_tree();
    c.bench_function("string match filter", |b| {
        b.iter(|| {
            let potential: Vec<NodeId> = tree
                .nodes
                .iter()
                .filter(|(_, pnode)| pnode.contains_stat_with_keyword(&tree, KEYWORD))
                .map(|(nid, _)| *nid)
                .collect();
            black_box(potential.len())
        })
    });
}

fn bench_type_match(c: &mut Criterion) {
    let tree = quick_tree();
    c.bench_function("type match filter", |b| {
        b.iter(|| {
            let potential: Vec<NodeId> = tree
                .nodes
                .iter()
                .map(|(nid, pnode)| (nid, pnode.as_passive_skill(&tree)))
                .filter(|(_, passive)| {
                    passive.stats().iter().any(|s| {
                        matches!(
                            s,
                            // See this is the one you'd want:
                            Stat::LightningDamage(_) // // but the 'keyword' strategy can also get you hits on these:
                                                     //     | Stat::LightningDamageWhileAffectedByHeraldOfThunder(_)
                                                     //     | Stat::LightningExposureEffect(_)
                                                     //     | Stat::ExtraDamageRollsWithLightningDamageOnNonCriticalHits(_)
                                                     //     | Stat::BaseMaximumLightningDamageResistance(_)
                                                     //     | Stat::MinionLightningDamageResistance(_)
                                                     //     | Stat::NonSkillBaseLightningDamageToGainAsCold(_)
                                                     //     | Stat::WitchPassiveMaximumLightningDamageFinal(_)
                        )
                    })
                })
                .map(|(nid, _)| *nid)
                .collect();
            black_box(potential.len())
        })
    });
}

criterion_group!(benches, bench_string_match, bench_type_match);
criterion_main!(benches);
