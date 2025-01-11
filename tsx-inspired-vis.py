import json
import matplotlib.pyplot as plt
import numpy as np

# Load the JSON file
with open('POE2_TREE.json', 'r') as file:
    data = json.load(file)

# Extract data
groups = data['passive_tree']['groups']
nodes = data['passive_tree']['nodes']
passive_skills = data['passive_skills']

# Function to compute node positions based on orbit and radius
def calculate_position(group, radius, position):
    orbit_radii = [0, 64, 128, 192, 256]  # Example orbit radii
    angle = position * (2 * np.pi / 40)  # Assume 40 slots in orbit
    r = orbit_radii[abs(radius)] if abs(radius) < len(orbit_radii) else 0
    x = group['x'] + r * np.cos(angle)
    y = group['y'] + r * np.sin(angle)
    return x, y

# Plot nodes and connections
plt.figure(figsize=(14, 14))

# Plot nodes
for node_id, node in nodes.items():
    skill_id = node.get("skill_id")
    skill = passive_skills.get(skill_id, {})
    parent_group = groups[str(node["parent"])]
    
    # Calculate node position
    x, y = calculate_position(parent_group, node["radius"], node["position"])
    
    # Determine style based on node properties
    is_notable = skill.get("is_notable", False)
    color = "gold" if is_notable else "blue"
    size = 30 if is_notable else 10
    
    plt.scatter(x, y, color=color, s=size)
    
    # Add skill name as a label (optional, remove for cleaner plot)
    if is_notable:
        plt.text(x, y, skill.get("name", ""), fontsize=8, ha='center', va='center')

# Plot connections
for node_id, node in nodes.items():
    x1, y1 = calculate_position(groups[str(node["parent"])], node["radius"], node["position"])
    for conn in node.get("connections", []):
        connected_node = nodes.get(str(conn["id"]))
        if not connected_node:
            continue
        x2, y2 = calculate_position(groups[str(connected_node["parent"])], connected_node["radius"], connected_node["position"])
        plt.plot([x1, x2], [y1, y2], color="gray", linewidth=0.5)

# Finalize the plot
plt.title("Path of Exile Passive Tree")
plt.axis("equal")
plt.show()
