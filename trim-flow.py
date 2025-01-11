import json
from typing import Dict

def load_filter_and_prune_tree(file_path: str) -> Dict:
    """Loads, filters nodes, and prunes passive skills from a JSON file."""
    with open(file_path, 'r') as f:
        data = json.load(f)
    
    nodes = data.get('passive_tree',{}).get('nodes', {})
    passive_skills = data.get('passive_skills', {})
    
    used_skill_ids = set()
    for node_id, node_data in nodes.items():
        skill_id = node_data.get("skill_id")
        if skill_id:
           used_skill_ids.add(skill_id)
    
    filtered_passive_skills = {}
    for skill_id, skill_data in passive_skills.items():
        if skill_id in used_skill_ids:
          filtered_passive_skills[skill_id] = skill_data
        elif not any(skill_id == node.get('skill_id') for _,node in nodes.items()):
          if 'name' in skill_data and isinstance(skill_data['name'], str):
           if any(skill_data['name'] in node.get('skill_id', '') for _,node in nodes.items()):
            filtered_passive_skills[skill_id] = skill_data

    
    data['passive_skills'] = filtered_passive_skills
    return data

def save_filtered_tree(data: dict, output_path: str):
    """Saves the filtered tree to a JSON file."""
    with open(output_path, 'w') as f:
        json.dump(data, f, indent=2)

def main():
    filtered_data = load_filter_and_prune_tree('flow-of-water-tree.json')
    save_filtered_tree(filtered_data, "flow-of-water-tree.json")
    print("Filtered and pruned tree saved to flow-of-water-tree.json")

if __name__ == "__main__":
    main()