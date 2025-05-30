# THIS IS UNMAINTAINED

# About

I created this because I couldn't quite achieve what I wanted on maxroll.gg or with PathOfBuilding.

There's two crates in here `poe_vis` and `poe_tree`, both with plenty of examples. see the `xyz/examples/*.rs` files.

The tooling in here was really to give _me_ a playground for seeing what algorithmic approaches could be deployed to optimise/scrutinise paths for the (non ascendency) passive tree in POE2. 

**It is far from complete, and so on -- but I no longer have the bandwidth to work on this as I have too much freelance work atm.**

## Features:
- Basic pathing and cumulative stats information about possible paths
- Advice for reducing the _frontier_ of a path by `n` moves to respec in another direction and optimize for criteria like `evasion_rating` or `energy_shield`. By taking an existing path, we see if it can be altered for the same number of levels, even considering a few `fixed` `node_id`s, thereby changing stats and showing users the deltas.
- Loading and saving characters (which include name, class, starting_node, and a bunch of activated nodes)

Character schema is:

```toml
character_class = "Monk"
name = "jengablox"
activated_node_ids = [30555, 49046, 53960, 10364, 55342, 17248]
date_created = "1970-01-01T00:00:00Z"
level = 6
quest_passive_skills = 0
starting_node = 10364
```

However importing the saves from Path Of Building is partially supported.

## Visualizations

The `poe_vis` binary runs a server that you can connect to see later in the README for example `curl`s etc.

- Basic visualization efforts
- Visualization characteristics like:

```text
nodes
edges
highlighting
activating/deactivating nodes
camera zoom
camera sensitivity
default temporary file paths
default zoom
key bindings
mouse bindings
```

These should be configurable through a user-friendly `user_config.toml` whose schema is:

```toml
#$ data/user_config.toml
[colors]
# Node Colors
attack = "#E95678"          # Red
mana = "#26BBD9"            # Blue
dexterity = "#29D398"       # Green
intelligence = "#26BBD9"    # Blue
strength = "#E95678"        # Red
all_nodes = "#3C3C3C"       # Dark grey
activated_nodes = "#29D398" # Green
activated_edges = "#22A882" # Green

# UI Colors
background = "#1C1E26"
foreground = "#C7C9CB"
red = "#E95678"
orange = "#FAB795"
yellow = "#FABD2F"
green = "#29D398"
blue = "#26BBD9"
purple = "#EE64AC"
cyan = "#59E3E3"

[controls]

move_left = ["h", "left_arrow"]
move_right = ["i", "right_arrow"]
move_up = ["e", "up_arrow"]
move_down = ["n", "down_arrow"]
camera_reset_home = ["esc"]

search_for_node_by_name = ["f"]
exit = ["q"]
load_character_nodes = ["l"]

#TODO: base_node_size
#TODO: default camera_zoom
#TODO: zoom sensitivity
#TODO: translate_sensitivity
#TODO: lock_camera_when_typing
#TODO: screen dimensions (save last camera position and window size)

```

- Loading a character in the visualizer should plot your path correctly.

- Orphaned nodes should be removed from the path and revert to default highlighting (i.e., none).

- Edges should light up when nodes are activated.

- Active nodes' `.stats` should accumulate into a stat sheet for your character, computed on-the-fly.

- The stat sheet is a left-hand side menu (toggleable with the same default key as the game uses).

# Out of Scope

- Gear (unless GGG wants to give me an API key?)

# Contributing

See the Rust contribution guidelines, but contributions are generally welcome (especially from Rust beginners! I will help you land PRs here time permitting!).

# Windows getting started

```sh
git clone $this_repo
cd $this_repo
$env:RUST_LOG = "off,poe_tools=error,poe_vis=debug"; cargo run -p poe_vis --bin vis --release
```

# Linux getting started

```sh

```

## Examples

1. How many nodes are there? what about just passives?

```sh
cargo run --example -p poe_tree --example node_count
```

2. How many `+%15 to Evasion Rating` nodes are there?

### RPC:

```sh
#!/usr/bin/env bash

URL="http://0.0.0.0:90210"
PATH1=(10364 42857 20024 44223 49220 36778 36479 12925 61196 58329)
PATH2=(10364 42857 20024 44223 49220 14725 34233 32545 61196 58329)
PATH3=(10364 55342 17248 53960 8975 61196 58329)

activate() {
  curl -s -X POST -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"activate_node\",\"params\":[$1],\"id\":1}" \
    "$URL"
  sleep 0.25
}

for node in "${PATH1[@]}"; do activate "$node"; done
for node in "${PATH2[@]}"; do activate "$node"; done
for node in "${PATH3[@]}"; do activate "$node"; done
```


The Tree, the `.json` representation thereof and the `./assets/*` from the game are subject to GGG's copyright etc.

Code is MIT.