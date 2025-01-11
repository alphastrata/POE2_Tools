import json
import argparse
from dataclasses import dataclass
from typing import List, Dict, Any, Set
from bokeh.plotting import figure, show
from bokeh.models import ColumnDataSource, HoverTool
from collections import deque


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
    edges: List[Edge] = None
    groups: Dict[str, Dict] = None


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


def get_reachable_nodes(tree: Tree, start_node_id: str, steps: int) -> Set[str]:
    visited = set()
    queue = deque([(start_node_id, 0)])
    reachable = set()

    while queue:
        current_node, depth = queue.popleft()
        if depth > steps or current_node in visited:
            continue
        visited.add(current_node)
        reachable.add(current_node)
        for conn in tree.nodes[current_node].connections:
            if conn not in visited:
                queue.append((conn, depth + 1))
    return reachable


def visualize_tree(tree: Tree, reachable_nodes: Set[str], start_node_id: str):
    node_x, node_y, node_color, node_labels = [], [], [], []
    edge_x0, edge_y0, edge_x1, edge_y1 = [], [], [], []

    for node_id in reachable_nodes:
        node = tree.nodes[node_id]
        group = tree.groups.get(str(node.parent), {"x": 0, "y": 0})
        x, y = group["x"], group["y"]
        node_x.append(x)
        node_y.append(y)
        node_color.append("red" if node.id == start_node_id else "blue")

        stats = node.skill_data.get("stats", {})
        stats_text = "\n".join(f"{k}: {v}" for k, v in stats.items())
        node_labels.append(
            f"Node ID: {node.id}\nSkill: {node.skill_data.get('name', 'None')}\nStats:\n{stats_text}"
        )

        for conn in node.connections:
            if conn in reachable_nodes:
                conn_node = tree.nodes[conn]
                conn_group = tree.groups.get(str(conn_node.parent), {"x": 0, "y": 0})
                edge_x0.append(x)
                edge_y0.append(y)
                edge_x1.append(conn_group["x"])
                edge_y1.append(conn_group["y"])

    node_source = ColumnDataSource(data={
        "x": node_x,
        "y": node_y,
        "color": node_color,
        "label": node_labels,
    })

    edge_source = ColumnDataSource(data={
        "x0": edge_x0,
        "y0": edge_y0,
        "x1": edge_x1,
        "y1": edge_y1,
    })

    p = figure(
        title="Passive Tree Path Visualization",
        width=1200,
        height=1200,
        match_aspect=True,
        tools="pan,wheel_zoom,box_zoom,reset,save",
    )
    p.segment(x0="x0", y0="y0", x1="x1", y1="y1", source=edge_source, line_width=1, color="gray")
    p.circle(x="x", y="y", size=10, source=node_source, color="color", alpha=0.8)

    hover = HoverTool(tooltips=[("Details", "@label")])
    p.add_tools(hover)

    show(p)


def main():
    parser = argparse.ArgumentParser(description="Visualize node paths in the passive tree.")
    parser.add_argument("--input", required=True, help="Path to the JSON file.")
    parser.add_argument("--starting-node", required=True, help="ID of the starting node.")
    parser.add_argument("--steps", type=int, default=3, help="Number of steps to traverse.")
    args = parser.parse_args()

    tree = load_tree(args.input)

    with open(args.input, 'r') as f:
        data = json.load(f)
    enrich_tree(tree, data["passive_skills"])

    reachable_nodes = get_reachable_nodes(tree, args.starting_node, args.steps)
    visualize_tree(tree, reachable_nodes, args.starting_node)


if __name__ == "__main__":
    main()
