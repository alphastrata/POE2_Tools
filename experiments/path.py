import json
import argparse
from typing import Dict, Any, List, Tuple
from collections import deque
import math

from bokeh.plotting import figure, show
from bokeh.models import ColumnDataSource, HoverTool
from bokeh.io import output_file
from dataclasses import dataclass

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
    edges: List[Edge]


def load_data(filepath: str) -> Dict[str, Any]:
    """Loads and returns the JSON data from the given file path."""
    with open(filepath, 'r') as f:
        return json.load(f)

def parse_tree_data(data: Dict[str, Any]) -> Tree:
    """Parses the JSON data into a Tree object."""
    nodes_data = data.get("passive_tree", {}).get("nodes", {})
    passive_skills = data.get("passive_skills", {})
    
    nodes: Dict[str, Node] = {}
    edges: List[Edge] = []

    for node_id, node_data in nodes_data.items():
      skill_id = node_data.get("skill_id")
      skill_data = passive_skills.get(skill_id) if skill_id else None
      nodes[node_id] = Node(
          id=node_id,
          parent=node_data["parent"],
          radius=node_data["radius"],
          position=node_data["position"],
          skill_id=skill_id,
          connections=[
              conn if isinstance(conn, str) else conn.get("id")
              for conn in node_data.get("connections", [])
          ],
          skill_data=skill_data
      )
      for connection in node_data.get("connections",[]):
        connection_id = connection if isinstance(connection, str) else connection.get("id")
        edges.append(Edge(source=node_id,target=connection_id))
    return Tree(nodes=nodes, edges=edges)


def calculate_node_positions(tree: Tree, groups: Dict[str, Any]) -> Dict[str, Tuple[float, float]]:
    """Calculates the x, y coordinates for each node."""
    ORBIT_NODE_COUNTS = {0: 1, 1: 6, 2: 16, 3: 16, 4: 40, 5: 72, 6: 72}
    node_positions: Dict[str, Tuple[float, float]] = {}

    for group_id, group in groups.items():
        group_x, group_y = group.get("x", 0), group.get("y", 0)
        for node_id, node in tree.nodes.items():
            if node.parent == int(group_id):
                orbit = node.radius
                position = node.position

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

def find_path_bfs(tree: Tree, start_node_id: str, end_node_id: str) -> List[str] | None:
    """
    Finds a path between two nodes using Breadth-First Search.
    Returns the list of node IDs representing the path or None if no path is found.
    """
    if start_node_id not in tree.nodes or end_node_id not in tree.nodes:
        return None

    queue = deque([(start_node_id, [start_node_id])])  # Node ID and path
    visited = {start_node_id}

    while queue:
        current_node_id, path = queue.popleft()

        if current_node_id == end_node_id:
            return path

        current_node = tree.nodes[current_node_id]
        for neighbor_id in current_node.connections:
            if neighbor_id not in visited and neighbor_id in tree.nodes:
                visited.add(neighbor_id)
                queue.append((neighbor_id, path + [neighbor_id]))
    return None

def visualize_path(
    tree: Tree,
    path: List[str],
    groups: Dict[str, Any],
    node_positions: Dict[str, Tuple[float,float]],
):
    """Visualizes the path on the passive skill tree using Bokeh."""
    node_x: List[float] = []
    node_y: List[float] = []
    node_name: List[str] = []
    node_ids: List[str] = []
    node_stats: List[str] = []

    edges: List[Tuple[Tuple[float, float], Tuple[float, float]]] = []

    path_edges: List[Tuple[Tuple[float, float], Tuple[float, float]]] = []


    for node_id, node in tree.nodes.items():
        x,y = node_positions.get(node_id, (0,0))
        node_x.append(x)
        node_y.append(-y) # Flip Y axis
        node_ids.append(node_id)
        node_name.append(node.skill_data.get("name", "???") if node.skill_data else "???")
        node_stats.append(str(node.skill_data.get("stats", {})) if node.skill_data else "")
        
        for connection in node.connections:
          connected_pos = node_positions.get(connection)
          if connected_pos:
              edges.append(((x, -y), (connected_pos[0], -connected_pos[1])))
    
    if path:
        for i in range(len(path) - 1):
            node1_id = path[i]
            node2_id = path[i+1]
            pos1 = node_positions.get(node1_id)
            pos2 = node_positions.get(node2_id)
            if pos1 and pos2:
                path_edges.append(( (pos1[0],-pos1[1]), (pos2[0],-pos2[1])))
    
    edge_source = ColumnDataSource(data={
        "x0": [edge[0][0] for edge in edges],
        "y0": [edge[0][1] for edge in edges],
        "x1": [edge[1][0] for edge in edges],
        "y1": [edge[1][1] for edge in edges],
    })
    
    path_edge_source = ColumnDataSource(data={
      "x0": [edge[0][0] for edge in path_edges],
      "y0": [edge[0][1] for edge in path_edges],
      "x1": [edge[1][0] for edge in path_edges],
      "y1": [edge[1][1] for edge in path_edges],
    })
    
    node_source = ColumnDataSource(data={
        "x": node_x,
        "y": node_y,
        "name": node_name,
        "id": node_ids,
        "stats": node_stats,
    })
    
    p = figure(width=1200, height=1200, title="Passive Tree with Path")
    p.segment(x0="x0", y0="y0", x1="x1", y1="y1", source=edge_source, line_width=1, color="gray")
    p.segment(x0="x0", y0="y0", x1="x1", y1="y1", source=path_edge_source, line_width=3, color="red")
    p.circle(x="x", y="y", size=10, source=node_source, color="blue", alpha=0.8)
    
    hover = HoverTool(tooltips=[
        ("Name", "@name"),
        ("ID", "@id"),
        ("Stats", "@stats"),
    ])
    p.add_tools(hover)

    output_file("passive_tree_with_path.html")
    show(p)

def main():
    """Main function to handle CLI arguments and find/plot the path."""
    parser = argparse.ArgumentParser(description="Plot a path between two nodes on the passive tree.")
    parser.add_argument("--input", required=True, help="Path to the JSON input file.")
    parser.add_argument("--start-node", required=True, help="ID of the starting node.")
    parser.add_argument("--end-node", required=True, help="ID of the ending node.")
    args = parser.parse_args()

    data = load_data(args.input)
    if not data:
      print("Error: could not load json data")
      return

    tree = parse_tree_data(data)
    groups = data.get('passive_tree', {}).get('groups',{})
    node_positions = calculate_node_positions(tree, groups)
    path = find_path_bfs(tree, args.start_node, args.end_node)

    if path:
        print(f"Path found: {path}")
        visualize_path(tree, path, groups, node_positions)
    else:
        print("No path found between the specified nodes.")

if __name__ == "__main__":
    main()