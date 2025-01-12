import json
import argparse
from typing import Set
from bokeh.plotting import figure, show
from bokeh.models import ColumnDataSource, HoverTool
from collections import deque

from poe_tree_types import Tree, Node


def load_tree(file_path: str) -> Tree:
    with open(file_path, 'r') as f:
        data = json.load(f)
    nodes = {
        str(node_id): Node(
            id=str(node_id),
            parent=node.get("parent", 0),
            radius=node.get("radius", 0),
            position=node.get("position", 0),
            skill_id=node.get("skill_id"),
            connections=[str(conn["id"]) for conn in node.get("connections", [])],
            skill_data=None,  # Skill data will be enriched later
        )
        for node_id, node in data["passive_tree"]["nodes"].items()
    }
    groups = data["passive_tree"]["groups"]
    return Tree(nodes=nodes, groups=groups)


def enrich_tree(tree: Tree, skills: dict):
    for node in tree.nodes.values():
        if node.skill_id and node.skill_id in skills:
            skill_data = skills[node.skill_id]
            node.skill_data = {
                "name": skill_data.get("name"),
                "icon": skill_data.get("icon"),
                "stats": skill_data.get("stats", {}),
                "is_notable": skill_data.get("is_notable", False),
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


def visualize_path(tree: Tree, reachable_nodes: Set[str], start_node_id: str):
    node_x, node_y, node_color, node_labels = [], [], [], []
    edge_x0, edge_y0, edge_x1, edge_y1 = [], [], [], []

    for node_id in reachable_nodes:
        node = tree.nodes[node_id]
        group = tree.groups.get(str(node.parent), {"x": 0, "y": 0})
        x, y = group["x"], group["y"]
        node_x.append(x)
        node_y.append(y)
        node_color.append("red" if node.id == start_node_id else "blue")

        # Include skill data and stats in the hover label
        stats = node.skill_data.get("stats", {})
        stats_text = "\n".join(f"{key}: {value}" for key, value in stats.items())
        node_labels.append(
            f"Node ID: {node.id}\n"
            f"Skill: {node.skill_data.get('name', 'None')}\n"
            f"Stats:\n{stats_text}"
        )

        for conn in node.connections:
            if conn in reachable_nodes:
                conn_node = tree.nodes[conn]
                conn_group = tree.groups.get(str(conn_node.parent), {"x": 0, "y": 0})
                cx, cy = conn_group["x"], conn_group["y"]
                edge_x0.append(x)
                edge_y0.append(y)
                edge_x1.append(cx)
                edge_y1.append(cy)

    node_source = ColumnDataSource(data={
        "x": node_x, "y": node_y, "color": node_color, "label": node_labels,
    })
    edge_source = ColumnDataSource(data={
        "x0": edge_x0, "y0": edge_y0, "x1": edge_x1, "y1": edge_y1,
    })

    p = figure(title="Passive Tree Path Visualization", width=800, height=800)
    p.segment(x0="x0", y0="y0", x1="x1", y1="y1", source=edge_source, line_width=1, color="gray")
    p.circle(x="x", y="y", size=10, source=node_source, color="color", alpha=0.8)

    hover = HoverTool(tooltips=[("Details", "@label")])
    p.add_tools(hover)

    show(p)


def main():
    parser = argparse.ArgumentParser(description="Visualize node paths in the passive tree.")
    parser.add_argument("--input", required=True, help="Path to the JSON file.")
    parser.add_argument("--start-node", required=True, help="ID of the start node.")
    parser.add_argument("--steps", type=int, default=3, help="Number of steps to traverse.")
    args = parser.parse_args()

    tree = load_tree(args.input)
    with open(args.input, 'r') as f:
        data = json.load(f)
    enrich_tree(tree, data["passive_skills"])

    reachable_nodes = get_reachable_nodes(tree, args.start_node, args.steps)
    visualize_path(tree, reachable_nodes, args.start_node)


if __name__ == "__main__":
    main()
