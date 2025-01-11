{
    echo "# Examine the structure inside passive_tree";
    jq '.passive_tree | keys' POE2_Tree.json;
    echo "\n# Examine a sample root_passive (First 2)";
    jq '.passive_tree.root_passives[:2]' POE2_Tree.json;

    echo "\n# Get the structure of groups (First 2 keys and their values)";
    jq '.passive_tree.groups | to_entries[:2] | map("\(.key): \(.value)")' POE2_Tree.json;

     echo "\n# Get the keys of a single node and value (First 2 keys and their values)";
    jq '.passive_tree.nodes | to_entries[:2] | map("\(.key): \(.value | keys)")' POE2_Tree.json;

    echo "\n# Look at the node with connections data (First 2)";
    jq '.passive_tree.nodes | to_entries[] | select(.value.connections != null) | .key'  POE2_Tree.json | head -n 2 | xargs jq '.passive_tree.nodes."\""'
    echo "\n# Examine one passive skill (First 2 keys and values)";
    jq '.passive_skills | to_entries[:2] | map("\(.key): \(.value | keys)")' POE2_Tree.json;

} > schema.txt