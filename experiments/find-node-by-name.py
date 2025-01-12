import argparse
from typing import Dict, Any, List, Tuple
from termcolor import colored

from common import load_data


def find_nodes_by_keyword(tree_data: Dict[str, Any], keyword: str) -> List[Tuple[str, str]]:
    """
    Finds and returns a list of tuples containing the node ID and its name
    if the node's name contains the given keyword (case-insensitive).
    Returns an empty list if no nodes are found.
    """
    nodes = tree_data.get('passive_tree', {}).get('nodes', {})
    passive_skills = tree_data.get('passive_skills', {})
    matching_nodes: List[Tuple[str, str]] = []

    for node_id, node_data in nodes.items():
        skill_id = node_data.get("skill_id")
        if skill_id and passive_skills.get(skill_id):
            skill_name = passive_skills[skill_id].get("name")
            if skill_name and keyword.lower() in skill_name.lower():
                matching_nodes.append((node_id, skill_name))
    return matching_nodes


def highlight_keyword(text: str, keyword: str) -> str:
    """Highlights the keyword in the given text using termcolor."""
    parts = text.lower().split(keyword.lower())
    colored_parts = []
    for i, part in enumerate(parts):
        colored_parts.append(part)
        if i < len(parts) - 1:
           colored_parts.append(colored(keyword, 'red'))
    return "".join(colored_parts)


def main():
    """Main function to handle CLI arguments and find the node ID(s)."""
    parser = argparse.ArgumentParser(description="Find node IDs by keyword in name.")
    parser.add_argument("--input", required=True, help="Path to the JSON input file.")
    parser.add_argument("--keyword", required=True, help="Keyword to search for in node names (case-insensitive).")
    args = parser.parse_args()

    data = load_data(args.input)
    if not data:
        print("Error: Could not load JSON data.")
        return

    matching_nodes = find_nodes_by_keyword(data, args.keyword)

    if matching_nodes:
        print(f"Nodes matching '{args.keyword}':")
        for node_id, skill_name in matching_nodes:
            highlighted_name = highlight_keyword(skill_name, args.keyword)
            print(f"  {node_id}: {highlighted_name}")
    else:
        print(f"No nodes found with keyword '{args.keyword}'.")


if __name__ == "__main__":
    main()