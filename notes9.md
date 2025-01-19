- we need to never draw the edges to the root nodes that chars spawn on:

```json
  "passive_tree": {
    "root_passives": [
      50459,
      47175,
      50986,
      61525,
      54447,
      44683
    ],

```

- we need to tint the nodes for things that make sense: green/cyan/red

- the nodes' screen scaling needs work. we need to set a max angular res.

- we need a nicer TreeVis::surround_with_circle() method for the active etc.

- fix the path solver so it finds the shortest point to an ALREADY active node NOT origin.

- camera needs to spawn at the center of the tree

- mark the center of the tree's x,y

- UI for class select

- save class in user_config.toml
