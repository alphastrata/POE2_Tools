# Performance

All projects using graphs are ambitiously performant.

This is a place to keep notes about the 'why' of things, without performance data, the 'why' is bullshit.

## Why encourage the strongly typed match on the exact enum you want for searching the graph for skills?

> `crates\poe_tree\benches\str_vs_enum_match.rs`

> `$ cargo bench -p poe_tree`

```txt
    string match filter     time:   [229.78 µs 230.85 µs 232.12 µs]
    type match filter       time:   [120.88 µs 121.17 µs 121.46 µs]
```

and if you get as specific as the keyword search's _intention_ is by you can do slightly better

```rust
Stat::LightningDamage(_) 
//     | Stat::LightningDamageWhileAffectedByHeraldOfThunder(_)
//     | Stat::LightningExposureEffect(_)
//     | Stat::ExtraDamageRollsWithLightningDamageOnNonCriticalHits(_)
//     | Stat::BaseMaximumLightningDamageResistance(_)
//     | Stat::MinionLightningDamageResistance(_)
//     | Stat::NonSkillBaseLightningDamageToGainAsCold(_)
//     | Stat::WitchPassiveMaximumLightningDamageFinal(_)

```

```txt
type match filter       time:   [110.56 µs 110.64 µs 110.73 µs]
                        change: [-9.5283% -9.2841% -9.0593%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**and** you have the `Stat` actual as opposed to the other method where we're looking it up (to get the actual numbers in there).
