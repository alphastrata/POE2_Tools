import json
from typing import Dict, Tuple, Set

def load_tree(file_path: str) -> Tuple[Dict[str, dict], Dict[str, dict]]:
    """Loads and parses the tree from a JSONL file."""
    nodes = {}
    groups = {}
    with open(file_path, 'r') as f:
        for line in f:
            data = json.loads(line)
            nodes = data.get('nodes',{})
            groups = data.get('groups',{})
    return nodes,groups


def expand_tree(nodes: Dict[str, dict], groups: Dict[str, dict]) -> Dict[str, Set[str]]:
    """Expands the tree to include all directly and indirectly connected nodes."""
    expanded: Dict[str, Set[str]] = {node_id: set() for node_id in nodes}

    for node_id, node_data in nodes.items():
        node_data.get("connections", [])
        
        
        visited_nodes = {node_id}
        nodes_to_visit =  [node_id]

        while nodes_to_visit:
            current_node_id = nodes_to_visit.pop(0)
            
            current_node = nodes.get(current_node_id)

            if current_node:
              for connection_data in current_node.get("connections",[]):
                 
                  connected_node_id = str(connection_data["id"])
                  if connected_node_id not in visited_nodes:
                      visited_nodes.add(connected_node_id)
                      nodes_to_visit.append(connected_node_id)
                      expanded[node_id].add(connected_node_id)



    return expanded

def main():
    nodes, groups = load_tree('small_tree.jsonl')
    expanded = expand_tree(nodes, groups)

    for node_id, connected_ids in expanded.items():
        print(f"Node {node_id} is connected to: {connected_ids}")

if __name__ == "__main__":
    main()