

This data is available at ./POE2_TREE.json, the top level schema is:
```json
{
    "passive_tree": {
        "root_passives": {},
        "nodes": {     
             "4": {
            "skill_id": "lightning14",
            "parent": 703,
            "radius": 0,
            "position": 0,
            "connections": [
            {
                "id": 11578,
                "radius": 0
            }
            ]
            // snip... 10000s more
        },
      },
        "groups": {
            "1": {
            "x": -22256.955078125,
            "y": -6513.4951171875
        },
        "2": {
            "x": -21155.17578125,
            "y": 4078.10498046875
        },
            // snip... 10000s more
        }
    },
    "passive_skills": { 
        "attributes1": {
        "name": "Attribute",
        "icon": "skillicons/passives/plusattribute",
        "stats": {
            "display_passive_attribute_text": 2
        }
        },
        "attributes2": {
        "name": "Attribute",
        "icon": "skillicons/passives/plusattribute",
        "stats": {
            "display_passive_attribute_text": 1
        }
        },
         // snip... 10000s more...
    }
}
```

Our goal is to first have a good way of 'linking' all these nodes properly, so that we can traverse a path from any given nodes to any others, for which my first go is:

```py
import json
import argparse
from dataclasses import dataclass
from typing import List, Dict, Any, Tuple
from bokeh.plotting import figure, show
from bokeh.layouts import gridplot
from bokeh.models import ColumnDataSource, HoverTool


@dataclass
class Node:
    id: str
    parent: int
    radius: int
    position: int
    skill_id: str | None
    connections: List[str]
    skill_data: Dict[str, Any] | None = None


@dataclass
class Edge:
    source: str
    target: str


@dataclass
class Tree:
    nodes: Dict[str, Node]
    groups: Dict[str, Dict]


def load_tree(file_path: str) -> Tree:
    with open(file_path, 'r') as f:
        data = json.load(f)

    nodes = {
        node_id: Node(
            id=node_id,
            parent=node.get("parent", 0),
            radius=node.get("radius", 0),
            position=node.get("position", 0),
            skill_id=node.get("skill_id"),
            connections=[str(conn["id"]) for conn in node.get("connections", [])],
            skill_data=None
        )
        for node_id, node in data["passive_tree"]["nodes"].items()
    }
    groups = data["passive_tree"]["groups"]
    return Tree(nodes=nodes, groups=groups)


def enrich_tree(tree: Tree, skills: dict) -> None:
    for node in tree.nodes.values():
        if node.skill_id and node.skill_id in skills:
            node.skill_data = {
                "name": skills[node.skill_id].get("name"),
                "icon": skills[node.skill_id].get("icon"),
                "stats": skills[node.skill_id].get("stats", {}),
                "is_notable": skills[node.skill_id].get("is_notable", False),
            }


def find_all_paths(tree: Tree, start_node: str, end_node: str, max_steps: int = 7) -> List[List[str]]:
    """
    Find all paths between start_node and end_node using DFS, printing each attempted path.
    """
    def dfs(current, target, path, visited, results):
        if len(path) > max_steps:
            return
        print(f"Exploring path: {path}")  # Print the current path
        if current == target:
            results.append(path[:])
            return
        visited.add(current)
        for neighbor in tree.nodes[current].connections:
            if neighbor not in visited:
                path.append(neighbor)
                dfs(neighbor, target, path, visited, results)
                path.pop()
        visited.remove(current)

    results = []
    dfs(start_node, end_node, [start_node], set(), results)
    return results


def get_node_positions(tree: Tree, path: List[str]) -> Tuple[List[float], List[float], List[str]]:
    """
    Compute positions for nodes in a given path.
    """
    x_coords, y_coords, labels = [], [], []
    for node_id in path:
        node = tree.nodes[node_id]
        group = tree.groups.get(str(node.parent), {"x": 0, "y": 0})
        x_coords.append(group["x"])
        y_coords.append(group["y"])

        stats = node.skill_data.get("stats", {})
        stats_text = "\n".join(f"{k}: {v}" for k, v in stats.items())
        labels.append(
            f"Node ID: {node.id}\nSkill: {node.skill_data.get('name', 'None')}\nStats:\n{stats_text}"
        )
        
    return x_coords, y_coords, labels


def plot_paths(tree: Tree, paths: List[List[str]]) -> None:
    """
    Plot all paths separately.
    """
    plots = []
    for path in paths:
        x_coords, y_coords, labels = get_node_positions(tree, path)
        source = ColumnDataSource(data={"x": x_coords, "y": y_coords, "label": labels})

        p = figure(
            title=f"Path: {' -> '.join(path)}",
            width=800,
            height=800,
            match_aspect=True,
            tools="pan,wheel_zoom,box_zoom,reset,save",
        )
        p.line(x="x", y="y", source=source, line_width=2, color="blue", alpha=0.8)
        p.circle(x="x", y="y", size=10, source=source, color="red", alpha=0.8)

        hover = HoverTool(tooltips=[("Details", "@label")])
        p.add_tools(hover)
        plots.append(p)

    grid = gridplot([plots[i:i + 2] for i in range(0, len(plots), 2)])
    show(grid)


def main():
    parser = argparse.ArgumentParser(description="Visualize paths in the passive tree.")
    parser.add_argument("--input", required=True, help="Path to the JSON file.")
    parser.add_argument("--starting-node", required=True, help="ID of the starting node.")
    parser.add_argument("--ending-node", required=True, help="ID of the ending node.")
    args = parser.parse_args()

    tree = load_tree(args.input)

    with open(args.input, 'r') as f:
        data = json.load(f)
    enrich_tree(tree, data["passive_skills"])

    paths = find_all_paths(tree, args.starting_node, args.ending_node)
    if not paths:
        print(f"No paths found between {args.starting_node} and {args.ending_node}.")
        return

    plot_paths(tree, paths)


if __name__ == "__main__":
    main()
```
Which works well enough:
```
 python a2b.py --starting-node 49220 --ending-node 56045 --input POE2_Tree.json | rg '56045'                            mac ✱ ◼
Exploring path: ['49220', '53960', '8975', '61196', '56045']
Exploring path: ['49220', '36778', '36479', '12925', '61196', '56045']
```


but what I would love to do is say, something like 
- start at node A and end at node B
- optimise for $keyword in .stats and produce for me paths that're plus/minus a $delta that have the same start and end but collect different stats printing me a list of the nodes' names, and the stats i'd aquire traversing them.

A few things to know about this tree (it's a graph).

All nodes are connected, however in principle from any given starting node the maximum number of steps is 123.

Here follows a more intricate breakdown of the 'components' of our schema and how we should refer to them, note the things with a $name because that's how we'll want do be referring to it in code



# Passive Skills:
From .passive_skills
```json
{
  "shadow_monk_notable1": {  // This is the skill_id
      "name": "Flow Like Water", // $name
      "icon": "skillicons/passives/harrier", //$art
      "stats": { //$stats, note there could be any number of these. The ones with numbers and +% are of particular interest we'll need to disambiguate what stats are +% and what are just plain old +
        "attack_and_cast_speed_+%": 8,
        "base_dexterity_and_intelligence": 5
      },
      "is_notable": true // This _will_ mean that when visualising we'll draw this Node larger than others.
    },
    "shadow_monk_notable2": {
      "name": "Step Like Mist",
      "icon": "skillicons/passives/minemanareservationnotable",
      "stats": {
        "mana_regeneration_rate_+%": 15,
        "base_movement_velocity_+%": 4,
        "base_dexterity_and_intelligence": 5
      },
      "is_notable": true // Note this field is optional
    },
    "spell_criticals1": {
      "name": "Spell Damage",
      "icon": "skillicons/passives/damagespells",
      "stats": {
        "spell_damage_+%": 8
      }
    },
    "spell_criticals2__": {
      "name": "Spell Damage",
      "icon": "skillicons/passives/damagespells",
      "stats": {
        "spell_damage_+%": 8
      }
    },
    "shadow_monk_notable1": {
        "name": "Flow Like Water",
        "icon": "skillicons/passives/harrier",
        "stats": {
          "attack_and_cast_speed_+%": 8,
          "base_dexterity_and_intelligence": 5
        },
        "is_notable": true
      },
    ...
}
```
the key, for example `shadow_monk_notable1` is how you get here from a Node

.nodes
```json
...
"49220": { //<<<<< THIS NUMBER IS A NodeID>>>>>
    "skill_id": "shadow_monk_notable1", //<<<<< THIS skill_id IS NOT ALWAYS UNIQUE, we would want to jump from here (the .nodes to the passive_skills though, not the other way around)>>>>>
    "parent": 610, // <<<< This is the GroupID, .nodes.groups[] is an array shown later>>>>
    "radius": 4, // This is somehow related to the 'orbit_index'
    "position": 12, // This is related to a Node's x,y `position` on the Orbit.
    "connections": [ // This tree avoids parent/child terminology as there's too many cycles.
      {
        "id": 10429,
        "radius": 0
      },
      {
        "id": 44223,
        "radius": -3
      },
      {
        "id": 53960,
        "radius": -6
      },
      {
        "id": 21336,
        "radius": 5
      },
      {
        "id": 36778,
        "radius": 6
      }
    ]
  },
  "49231": {
    "skill_id": "attack_speed9",
    "parent": 389,
    "radius": 7,
    "position": 20,
    "connections": [
      {
        "id": 43183,
        "radius": 0
      }
    ]
  },
  ...
  ```

  there are more Node(s) than anything else, which is why the Nodes reference the data in the .passive_skills as there are many duplicates.

  the `parent`, I believe refers to the GroupId:

  ```json
  "607": {
    "x": 1760.625,
    "y": -17480.375
  },
  "610": {
    "x": 1821.35498046875,
    "y": -1202.9449462890625
  },
  "611": {
    "x": 1834.8050537109375,
    "y": -7995.134765625
  },
  ```


My issue with plotting these has been that -- I do not know the units, and have been unable to ascertain them and the only information I **do** have about 'angles' etc and 'orbits' is this, [from GGG's publicly available Tree data from the previous game.](https://github.com/grindinggear/skilltree-export/blob/master/README.md)
---
> _The amount of nodes per orbit has changed to give us more angles to work with._

| Orbit | Nodes (Old) | Nodes (New) |
| --- | --- | --- |
| 0 | 1 | 1 |
| 1 | 6 | 6 |
| 2 | 12 | **16** |
| 3 | 12 | **16** |
| 4 | 40 | 40 |
| 5 | 72 | 72 |
| 6 | 72 | 72 |

The angle for positions in these orbits (2 & 3) are now:

| Orbit Index | Angle |
| --- | --- |
| 0 | 0 |
| 1 | 30 |
| 2 | **45** |
| 3 | 60 |
| 4 | 90 |
| 5 | 120 |
| 6 | **135** |
| 7 | 150 |
| 8 | 180 |
| 9 | 210 |
| 10 | **225** |
| 11 | 240 |
| 12 | 270 |
| 13 | 300 |
| 14 | **315** |
| 15 | 330 |
---

Which fits with our earlier calculation.

## foreign code
As the POE2_TREE.json data we have, shown earlier is from a website, here is what I was able to get of _their_ code which plots it well indeed!:

i believe it comes from renderPassives.tsx
```js
for (const { id: id2, radius } of node.connections) {
      if (id2 <= id) continue
      const node2 = data.node(id2)
      if (!node2) continue
      const skill2 = node2.skill
      if (skill.ascendancy !== skill2.ascendancy) continue
      if (skill.starting_node || skill2.starting_node) continue
      if (skill.is_just_icon || skill2.is_just_icon) continue

      const textures = skill.ascendancy ? ascendancyLineTextures : lineTextures

      const state = lineState(
        node.state | (node.weaponSet << NodeState.WeaponSetShift),
        node2.state | (node2.weaponSet << NodeState.WeaponSetShift)
      )
      if (radius && orbitRadii[Math.abs(radius)]) {
        const r = orbitRadii[Math.abs(radius)]
        const dx = node2.x - node.x
        const dy = node2.y - node.y
        const dist = Math.sqrt(dx * dx + dy * dy)
        if (dist < r * 2) {
          const perp = Math.sqrt(r * r - (dist * dist) / 4) * Math.sign(radius)
          const cx = node.x + dx / 2 + perp * (dy / dist)
          const cy = node.y + dy / 2 - perp * (dx / dist)
          items.push({
            layer: Layer.LINE,
            image: state ? textures.active : textures.normal,
            color: frameColor(state),
            type: 'arc',
            x: cx,
            y: cy,
            r: r,
            a1: Math.atan2(node.y - cy, node.x - cx),
            a2: Math.atan2(node2.y - cy, node2.x - cx)
          })
        }
      } else if (
        node.group === node2.group &&
        node.radius === node2.radius &&
        !radius
      ) {
        items.push({
          layer: Layer.LINE,
          image: state ? textures.active : textures.normal,
          color: frameColor(state),
          type: 'arc',
          x: node.group.x,
          y: node.group.y,
          r: node.radius,
          a1: node.arc,
          a2: node2.arc
        })
      } else {
        items.push({
          layer: Layer.LINE,
          image: state ? textures.active : textures.normal,
          color: frameColor(state),
          type: 'line',
          x1: node.x,
          y1: node.y,
          x2: node2.x,
          y2: node2.y
        })
      }
    }
  }
```

```tsx
const treeDimensions = {
  left: -12000,
  top: -12000,
  right: 12000,
  bottom: 12000
}

const atlasTreeDimensions = {
  left: -4500,
  top: -6500,
  right: 4500,
  bottom: 2500
}
```
