import json
import math
import matplotlib.pyplot as plt

# Load the JSON tree data
with open('./POE2_TREE.json', 'r') as f:
    tree_data = json.load(f)

# Constants
orbit_radii = [0, 82, 162, 335, 493]  # Radii for each orbit
skills_per_orbit = [1, 6, 12, 12, 40]  # Number of skills in each orbit

def calc_node_position(group_x, group_y, orbit, orbit_index):
    """Calculate the position of a node given its group and orbit details."""
    angle = orbit_index * (2 * math.pi / skills_per_orbit[orbit])
    offset_x = math.cos(angle) * orbit_radii[orbit]
    offset_y = math.sin(angle) * orbit_radii[orbit]
    return group_x + offset_x, group_y + offset_y

def plot_nodes(node_ids, ax, title):
    """Plot nodes given a list of node IDs."""
    for node_id in node_ids:
        # Find the node group and details
        for group_id, group in tree_data['passive_tree']['groups'].items():
            if 'n' in group and node_id in group['n']:
                group_x, group_y = group['x'], group['y']
                orbit_index = group['n'].index(node_id)
                orbit = 2  # Assume orbit level for demonstration
                x, y = calc_node_position(group_x, group_y, orbit, orbit_index)
                ax.scatter(x, y, label=f"Node {node_id}")
                ax.text(x, y, node_id, fontsize=8, ha='right')
                break

    ax.set_title(title)
    ax.set_xlabel('X')
    ax.set_ylabel('Y')
    ax.legend()

# Define nodes to plot for each case
nodes_plot1 = ['49220', '53960', '8975', '61196', '56045']
nodes_plot2 = ['49220', '36778', '36479', '12925', '61196', '56045']

# Create the plots
fig, axes = plt.subplots(1, 2, figsize=(16, 8))
plot_nodes(nodes_plot1, axes[0], "Plot 1: Nodes")
plot_nodes(nodes_plot2, axes[1], "Plot 2: Nodes")

plt.tight_layout()
plt.show()

# Markdown documentation
markdown_doc = """
# Passive Skill Tree Node Placement

This document explains the process of placing nodes on a passive skill tree and provides a worked example using predefined nodes.

## Constants and Setup
- **Orbit Radii**: `[0, 82, 162, 335, 493]`
- **Skills Per Orbit**: `[1, 6, 12, 12, 40]`

### Node Position Calculation
Each node's position is determined using:
1. **Group Coordinates**: The base (x, y) position of the group.
2. **Orbit Index**: The node's position within the orbit.
3. **Offset Calculation**:
   - `angle = orbit_index * (2 * π / skills_per_orbit[orbit])`
   - `offset_x = cos(angle) * orbit_radius[orbit]`
   - `offset_y = sin(angle) * orbit_radius[orbit]`
   - `node_x = group_x + offset_x`
   - `node_y = group_y + offset_y`

## Worked Example

### Nodes in Plot 1
- `['49220', '53960', '8975', '61196', '56045']`

### Nodes in Plot 2
- `['49220', '36778', '36479', '12925', '61196', '56045']`

### Results
The plots below show the positions of these nodes.

![Plot 1 and 2](path/to/plots.png)
"""

print(markdown_doc)
