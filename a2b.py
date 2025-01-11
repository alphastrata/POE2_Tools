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
