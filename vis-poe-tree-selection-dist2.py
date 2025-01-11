from collections import deque
import json
import math
from bokeh.plotting import figure, show
from bokeh.models import ColumnDataSource, HoverTool
from typing import Set


def calculate_node_position(node, group, orbit_radii, skills_per_orbit, tree_size, min_x, min_y):
    """Calculate the (x, y) position of a node."""
    orbit = node.get("radius", 0)
    position = node.get("position", 0)
    group_x, group_y = group["x"], group["y"]

    if orbit >= len(orbit_radii) or orbit >= len(skills_per_orbit):
        return group_x, -group_y

    radius = orbit_radii[orbit]
    angle_step = 2 * math.pi / skills_per_orbit[orbit]
    angle = position * angle_step

    # Calculate relative position
    x = group_x + math.cos(angle) * radius
    y = group_y + math.sin(angle) * radius

    # Apply scaling and centering
    x = (x - min_x) / tree_size
    y = -(y - min_y) / tree_size  # Flip y-axis and normalize
    return x, y



def enrich_tree(tree, skills):
    """Attach skill data to each node."""
    for node_id, node in tree["nodes"].items():
        skill_id = node.get("skill_id")
        if skill_id and skill_id in skills:
            node["skill_data"] = {
                "name": skills[skill_id].get("name", "Unknown"),
                "stats": skills[skill_id].get("stats", {}),
                "is_notable": skills[skill_id].get("is_notable", False),
            }
        else:
            node["skill_data"] = {
                "name": "???",
                "stats": {},
                "is_notable": False,
            }


def visualize_tree(tree, reachable_nodes, start_node_id, orbit_radii, skills_per_orbit):
    """Visualize the passive tree using Bokeh."""
    node_x, node_y, node_color, node_labels = [], [], [], []
    edge_x0, edge_y0, edge_x1, edge_y1 = [], [], [], []

    for node_id in reachable_nodes:
        node = tree["nodes"][node_id]
        group = tree["groups"].get(str(node["parent"]), {"x": 0, "y": 0})

        # Calculate node position
        x, y = calculate_node_position(node, group, orbit_radii, skills_per_orbit)
        node_x.append(x)
        node_y.append(y)
        node_color.append("red" if node_id == start_node_id else "blue")

        # Create hover labels
        skill_data = node.get("skill_data", {})
        stats = skill_data.get("stats", {})
        stats_text = "\n".join(f"{k}: {v}" for k, v in stats.items())
        node_labels.append(
            f"Node ID: {node_id}\nSkill: {skill_data.get('name', 'Unknown')}\nStats:\n{stats_text}"
        )

        # Draw edges to connected nodes
        for conn in node.get("connections", []):
            conn_id = str(conn["id"])
            if conn_id in reachable_nodes:
                conn_node = tree["nodes"][conn_id]
                conn_group = tree["groups"].get(str(conn_node["parent"]), {"x": 0, "y": 0})
                conn_x, conn_y = calculate_node_position(conn_node, conn_group, orbit_radii, skills_per_orbit)

                edge_x0.append(x)
                edge_y0.append(y)
                edge_x1.append(conn_x)
                edge_y1.append(conn_y)

    # Create Bokeh data sources
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

    # Plot with Bokeh
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


def get_reachable_nodes(tree: dict, start_node_id: str, steps: int) -> Set[str]:
    """Get all nodes reachable within a certain number of steps."""
    visited = set()
    queue = deque([(start_node_id, 0)])
    reachable = set()

    while queue:
        current_node, depth = queue.popleft()
        if depth > steps or current_node in visited:
            continue
        visited.add(current_node)
        reachable.add(current_node)
        for conn in tree["nodes"][current_node]["connections"]:
            conn_id = str(conn["id"])  # Ensure the connection ID is a string
            if conn_id not in visited:
                queue.append((conn_id, depth + 1))
    return reachable

def main():
    with open("POE2_TREE.json", "r") as f:
        data = json.load(f)

    tree = {
        "nodes": data["passive_tree"]["nodes"],
        "groups": data["passive_tree"]["groups"]
    }
    skills = data.get("passive_skills", {})

    # Enrich tree nodes with skill data
    enrich_tree(tree, skills)

    starting_node = "49220"
    steps = 20

    # Orbit radii and number of skills per orbit
    orbit_radii = [0, 82, 162, 335, 493]
    skills_per_orbit = [1, 6, 12, 12, 40]

    # Calculate tree size and offsets
    max_x = max(group["x"] for group in tree["groups"].values())
    max_y = max(group["y"] for group in tree["groups"].values())
    min_x = min(group["x"] for group in tree["groups"].values())
    min_y = min(group["y"] for group in tree["groups"].values())
    tree_size = min(max_x - min_x, max_y - min_y) * 1.1

    # Find reachable nodes and visualize
    reachable_nodes = get_reachable_nodes(tree, starting_node, steps)
    visualize_tree(tree, reachable_nodes, starting_node, orbit_radii, skills_per_orbit, tree_size, min_x, min_y)
