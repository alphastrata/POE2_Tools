import math
import json
import argparse

from bokeh.plotting import figure, show
from bokeh.models import ColumnDataSource, HoverTool
from bokeh.io import output_file

# Argument parser to get input file
def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, help="Path to JSON")
    return parser.parse_args()

# Load JSON data from file
def load_data(path):
    with open(path, "r") as f:
        return json.load(f)

# Enrich tree nodes with skill data
def enrich_tree(tree, skills):
    for node_id, node in tree.get("nodes", {}).items():
        skill_id = node.get("skill_id")
        if skill_id and skill_id in skills:
            node["skill_data"] = skills[skill_id]

# Calculate node position based on orbit and position
def calculate_node_position(group_x, group_y, orbit, position):
    ORBIT_NODE_COUNTS = {0: 1, 1: 6, 2: 16, 3: 16, 4: 40, 5: 72, 6: 72}
    if orbit not in ORBIT_NODE_COUNTS:
        return group_x, group_y  # Default to group center if orbit is invalid

    node_count = ORBIT_NODE_COUNTS[orbit]
    angle_increment = 360 / node_count
    angle_deg = position * angle_increment
    angle_rad = math.radians(angle_deg)
    radius = orbit * 100

    x_offset = radius * math.cos(angle_rad)
    y_offset = radius * math.sin(angle_rad)

    return group_x + x_offset, group_y + y_offset

# Visualize the passive tree
def visualize_tree_with_connections(tree, groups):
    node_x = []
    node_y = []
    node_name = []
    node_id = []
    node_conn = []
    edges = []

    for group_id, group in groups.items():
        group_x, group_y = group.get("x", 0), group.get("y", 0)
        for node_id, node in tree.get("nodes", {}).items():
            if node.get("parent") == int(group_id):
                orbit = node.get("orbit", 0)
                position = node.get("position", 0)
                nx, ny = calculate_node_position(group_x, group_y, orbit, position)
                node_x.append(nx)
                node_y.append(ny)
                node_name.append(node.get("skill_data", {}).get("name", "???"))
                node_conn.append(node.get("connections", []))

                for connection in node.get("connections", []):
                    connection_id = connection if isinstance(connection, str) else connection.get("id")
                    connected_node = tree.get("nodes", {}).get(connection_id, {})
                    connected_orbit = connected_node.get("orbit", 0)
                    connected_position = connected_node.get("position", 0)
                    cx, cy = calculate_node_position(group_x, group_y, connected_orbit, connected_position)
                    edges.append(((nx, ny), (cx, cy)))

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
        "conn": [str(c) for c in node_conn],
    })

    p = figure(width=1200, height=1200, title="Passive Tree Visualization with Connections")

    p.segment(x0="x0", y0="y0", x1="x1", y1="y1", source=edge_source, line_width=1, color="gray")
    p.circle(x="x", y="y", size=10, source=node_source, color="blue", alpha=0.8)

    hover = HoverTool(tooltips=[("Name", "@name"), ("Connections", "@conn")])
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

    enrich_tree(passive_tree, passive_skills)
    visualize_tree_with_connections(passive_tree, passive_tree.get("groups", {}))

if __name__ == "__main__":
    main()
