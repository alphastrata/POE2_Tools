This is a bit of a sprawl, but my main question should be answerable (trivially) by looking at these two things, assuming you've a rust toolchain installed.

1. `cargo run -r -p poe_tree --bin vis` will run the visualiser.
2. `➜  poo-tools2 git:(main) `cargo run -r --example -p poe_tree --example possible_builds -- --visualiser`

the `--visualiser` arg will plot the nodes from the paths in realtime in the visualiser, running this one without the `--visualiser` has some, if you have `RUST_LOG=debug` set in your shell's env will help us drive down the times.

My goal is very simple, currently I cannot really (in reasonable time) compute the `n` of greater than 50. The upper limit on steps takeable is `123` so we don't have to go far.

To get possible start locations:

````rust
let nodes: Vec<(&'static str, &[u32; 2])> = get_level_one_nodes()
        .iter()
        .map(|(name, ids)| (*name, ids))
        .collect();
``` Why collect? because the static won't play nicely with rayon.


````

At the moment my naive implementation to collect **all** possible paths of length `n` (i.e that many steps) is:

```rust
let paths = tree.walk_n_steps(start_node, steps);
```

The impl is in `poe_tree::pathfinding`

There's a bunch of other pathfinding shit in there, the BFS is good but the dijkstras area WIPs.

the `NodeId` alias is deliberately `u16` because that's conspiciously the highest value assigned as an ID for the nodes in the raw data. (i'll take 4 for the price of a `usize` anyday!)
