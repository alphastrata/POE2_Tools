# Performance

All projects using graphs are ambitiously performant.

This is a place to keep notes about the 'why' of things, without performance data, the 'why' is bullshit.

## Why encourage the strongly typed match on the exact enum you want for searching the graph for skills?

> `crates\poe_tree\benches\str_vs_enum_match.rs`

> `$ cargo bench -p poe_tree`

```shell
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

```shell
type match filter       time:   [110.56 µs 110.64 µs 110.73 µs]
                        change: [-9.5283% -9.2841% -9.0593%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**and** you have the `Stat` actual as opposed to the other method where we're looking it up (to get the actual numbers in there).

## Why sometimes BFS, why sometimes Dijkstra?

> `crates\poe_tree\benches\bfs_vs_dijkstra.rs`

```shell
BFS shortest path       time:   [7.5281 ms 7.5433 ms 7.5605 ms]
Found 11 outliers among 100 measurements (11.00%)
  11 (11.00%) high severe

Dijkstra shortest path  time:   [9.0483 ms 9.0559 ms 9.0637 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

BFS shortest path reversed
                        time:   [6.1527 ms 6.1574 ms 6.1626 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

Dijkstra shortest path reversed
                        time:   [6.9833 ms 6.9891 ms 6.9950 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
```

because depending on which direction we're travelling (toward the center, or outward from the center) of the tree I've observed paths' errors and compute times fluctuate a lot -- it's not my area of expertise.
For now we use BFS everywhere, because the above test data says it is better.
