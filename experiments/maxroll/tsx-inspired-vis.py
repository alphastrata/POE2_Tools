import json
import numpy as np
import plotly.graph_objects as go

# Load the JSON file
with open('POE2_TREE.json', 'r') as file:
    data = json.load(file)

# Extract data
groups = data['passive_tree']['groups']
nodes = data['passive_tree']['nodes']
passive_skills = data['passive_skills']

# Define orbit radii and slots
orbit_radii = [0, 82, 162, 335, 493, 662, 812, 972, 1133, 1303]
orbit_slots = [1, 6, 12, 12, 40, 60, 60, 60, 60, 60]

# Function to compute node positions based on orbit and radius
def calculate_position(group, radius, position):
    if abs(radius) >= len(orbit_radii):
        raise ValueError(f"Radius {radius} exceeds defined orbit radii.")
    r = orbit_radii[abs(radius)]
    slots = orbit_slots[abs(radius)]
    angle = position * (2 * np.pi / slots)
    x = group['x'] + r * np.cos(angle)
    y = group['y'] + r * np.sin(angle)
    return x, y

# Lists for plotly
node_x, node_y, node_color, node_size, node_labels = [], [], [], [], []
edge_x, edge_y = [], []

# Process nodes
for node_id, node in nodes.items():
    skill_id = node.get("skill_id")
    skill = passive_skills.get(skill_id, {})
    parent_group = groups[str(node["parent"])]

    # Calculate node position
    try:
        x, y = calculate_position(parent_group, node["radius"], node["position"])
    except ValueError as e:
        print(f"Skipping node {node_id}: {e}")
        continue

    # Add node properties
    is_notable = skill.get("is_notable", False)
    color = "gold" if is_notable else "blue"
    size = 20 if is_notable else 10
    label = skill.get("name", "")

    node_x.append(x)
    node_y.append(y)
    node_color.append(color)
    node_size.append(size)
    node_labels.append(label)

    # Process connections
    for conn in node.get("connections", []):
        connected_node = nodes.get(str(conn["id"]))
        if not connected_node:
            continue
        try:
            x2, y2 = calculate_position(
                groups[str(connected_node["parent"])],
                connected_node["radius"],
                connected_node["position"],
            )
            edge_x.extend([x, x2, None])
            edge_y.extend([y, y2, None])
        except ValueError as e:
            print(f"Skipping connection from node {node_id}: {e}")
            continue

# Create plotly traces
edge_trace = go.Scatter(
    x=edge_x,
    y=edge_y,
    line=dict(width=0.5, color="gray"),
    hoverinfo="none",
    mode="lines",
)

node_trace = go.Scatter(
    x=node_x,
    y=node_y,
    mode="markers+text",
    hoverinfo="text",
    text=node_labels,
    marker=dict(
        size=node_size,
        color=node_color,
        line=dict(width=1, color="black"),
    ),
)

# Create the figure
fig = go.Figure(data=[edge_trace, node_trace])
fig.update_layout(
    title="Path of Exile Passive Tree",
    title_font_size=20,
    showlegend=False,
    xaxis=dict(showgrid=False, zeroline=False),
    yaxis=dict(showgrid=False, zeroline=False),
    hovermode="closest",
)

# Show the plot
fig.show()
