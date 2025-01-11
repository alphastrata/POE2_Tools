import math
import json
import argparse

from bokeh.plotting import figure, show
from bokeh.models import ColumnDataSource, HoverTool
from bokeh.io import output_file
from typing import Dict, List, Tuple, Any

# Argument parser to get input file
def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, help="Path to JSON")
    return parser.parse_args()

# Load JSON data from file
def load_data(path: str) -> Dict[str, Any]:
    with open(path, "r") as f:
        return json.load(f)

# Enrich tree nodes with skill data
def enrich_tree(tree: Dict[str, Any], skills: Dict[str, Any]) -> Dict[str, Any]:
    for node_id, node in tree.get("nodes", {}).items():
        skill_id = node.get("skill_id")
        if skill_id and skill_id in skills:
            node["skill_data"] = skills[skill_id]
        else:
            node["skill_data"] = {"name":"???"} # default value to avoid KeyErrors
    return tree
    
# Calculate node position based on orbit and position
def calculate_node_positions(tree: Dict[str, Any], groups: Dict[str, Any]) -> Dict[str, Tuple[float, float]]:
    ORBIT_NODE_COUNTS = {0: 1, 1: 6, 2: 16, 3: 16, 4: 40, 5: 72, 6: 72}
    node_positions: Dict[str, Tuple[float, float]] = {}

    for group_id, group in groups.items():
        group_x, group_y = group.get("x", 0), group.get("y", 0)
        for node_id, node in tree.get("nodes", {}).items():
            if node.get("parent") == int(group_id):
                orbit = node.get("orbit", 0)
                position = node.get("position", 0)

                if orbit not in ORBIT_NODE_COUNTS:
                    node_positions[node_id] = (group_x, group_y)  # Default to group center
                    continue

                node_count = ORBIT_NODE_COUNTS[orbit]
                angle_increment = 360 / node_count
                angle_deg = position * angle_increment
                angle_rad = math.radians(angle_deg)
                radius = orbit * 100

                x_offset = radius * math.cos(angle_rad)
                y_offset = radius * math.sin(angle_rad)
                node_positions[node_id] = (group_x + x_offset, group_y + y_offset)

    return node_positions


# Visualize the passive tree
def visualize_tree_with_connections(tree: Dict[str, Any], groups: Dict[str, Any], node_positions: Dict[str, Tuple[float, float]]):
    node_x: List[float] = []
    node_y: List[float] = []
    node_name: List[str] = []
    node_parent: List[int] = []
    node_radius: List[int] = []
    node_stats: List[str] = []
    edges: List[Tuple[Tuple[float, float], Tuple[float, float]]] = []

    for node_id, node in tree.get("nodes", {}).items():
        x, y = node_positions.get(node_id, (0, 0))  # default to origin if not in node_positions
        node_x.append(x)
        node_y.append(-y)  # Flip the y-axis
        node_name.append(node.get("skill_data", {}).get("name", "???"))
        node_parent.append(node.get("parent", None))
        node_radius.append(node.get("radius", None))
        
        skill_data = node.get("skill_data", {})
        stats = skill_data.get("stats", {})
        node_stats.append(str(stats))

        for connection in node.get("connections", []):
            connection_id = connection if isinstance(connection, str) else connection.get("id")
            connected_pos = node_positions.get(connection_id, None)
            if connected_pos:
              edges.append(((x, -y), (connected_pos[0], -connected_pos[1])))


    edge_source = ColumnDataSource(data={
        "x0": [edge[0][0] for edge in edges],
        "y0": [edge[0][1] for edge in edges],
        "x1": [edge[1][0] for edge in edges],
        "y1": [edge[1][1] for edge in edges],
    })
    
    node_source = ColumnDataSource(data={
        "x": node_x,
        "y": node_y,
        "name": node_name,
        "parent": node_parent,
        "radius": node_radius,
        "stats": node_stats,
    })

    p = figure(width=1200, height=1200, title="Passive Tree Visualization with Connections")
    p.segment(x0="x0", y0="y0", x1="x1", y1="y1", source=edge_source, line_width=1, color="gray")
    p.circle(x="x", y="y", size=10, source=node_source, color="blue", alpha=0.8)

    hover = HoverTool(tooltips=[
        ("Name", "@name"),
        ("Parent", "@parent"),
        ("Radius", "@radius"),
        ("Stats", "@stats")
    ])
    p.add_tools(hover)

    output_file("passive_tree.html")
    show(p)


# Main function
def main():
    args = parse_args()
    data = load_data(args.input)

    if not data or "passive_tree" not in data or "passive_skills" not in data:
        print("Error: Invalid or missing data in the input file.")
        return

    passive_tree = data["passive_tree"]
    passive_skills = data["passive_skills"]

    enriched_tree = enrich_tree(passive_tree, passive_skills)
    node_positions = calculate_node_positions(enriched_tree, enriched_tree.get("groups", {}))
    visualize_tree_with_connections(enriched_tree, enriched_tree.get("groups", {}), node_positions)

if __name__ == "__main__":
    main()