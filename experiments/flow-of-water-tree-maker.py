import json
from typing import Dict, List

def load_and_filter_tree(file_path: str, target_ids: List[str]) -> Dict:
    """Loads and parses the tree from a JSON file and filters nodes, preserving structure."""
    with open(file_path, 'r') as f:
        data = json.load(f)
    
    
    nodes = data.get('passive_tree',{}).get('nodes', {})
    groups = data.get('passive_tree',{}).get('groups', {})
    filtered_nodes = {}
    for node_id, node_data in nodes.items():
            skill_id = node_data.get("skill_id")
            if node_id in target_ids or str(node_data.get("parent")) in target_ids or skill_id in target_ids:
                filtered_nodes[node_id] = node_data
        
    filtered_groups = {k:v for k,v in groups.items() if int(k) > 500 and int(k) < 700}

    data['passive_tree']['nodes'] = filtered_nodes
    data['passive_tree']['groups'] = filtered_groups

    return data

def save_filtered_tree(data: dict, output_path: str):
    """Saves the filtered tree to a JSON file."""
    with open(output_path, 'w') as f:
        json.dump(data, f, indent=2)

def main():
    target_ids = ["49220", "610", "shadow_monk_notable1", "10429", "44223", "53960", "21336", "36778"]
    filtered_data = load_and_filter_tree('POE2_TREE.json', target_ids)
    save_filtered_tree(filtered_data, "flow-of-water-tree.json")
    print("Filtered tree saved to flow-of-water-tree.json")

if __name__ == "__main__":
    main()