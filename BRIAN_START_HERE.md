This is a bit of a sprawl, but my main question should be answerable (trivially) by looking at these two things, assuming you've a rust toolchain installed.

1. `cargo run -r -p poe_tree --bin vis` will run the visualiser.
2. `cargo run -r --example -p poe_tree --example possible_builds -- --visualiser`

the `--visualiser` arg will plot the nodes from the paths in realtime in the visualiser, running this one without the `--visualiser` has some, if you have `RUST_LOG=debug` set in your shell's env will help us drive down the times.

My goal is very simple, currently I cannot really (in reasonable time) compute the `n` of greater than 50. The upper limit on steps walkable is `123` so we don't have to go far.

I believe the solution will be some way to compress the longest runs of NodeIds that are common between paths we're evaluating, perhaps CSR is the way... I dunno..

To get possible start locations:

```rust
let nodes: Vec<(&'static str, &[u32; 2])> = get_level_one_nodes()
        .iter()
        .map(|(name, ids)| (*name, ids))
        .collect();
```

Why collect? because the static won't play nicely with rayon.

There's lots of this, around:

```rust
 // Less garbled output to stdout in the serial case.
    nodes.iter().for_each(|(character, node_ids)| {
        // nodes.par_iter().for_each(|(character, node_ids)| {
```

because for debugging it's usually easier to see the correct order of things in the serial case.

At the moment my naive implementation to collect **all** possible paths of length `n` (i.e that many steps) is:

```rust
let paths = tree.walk_n_steps(start_node, steps);
```

The impl is in `poe_tree::pathfinding`

There's a bunch of other pathfinding shit in there, the BFS is good but the dijkstras area WIPs.

the `NodeId` alias is deliberately `u16` because that's conspiciously the highest value assigned as an ID for the nodes in the raw data. (i'll take 4 for the price of a `usize` anyday!)

you may need to purge the contents of `.cargo/config.toml` as you probs don't have the same home build server setup I do with `sccachce` etc.

sorry about the mess [not really WIP is allowed to be messy IMO]
