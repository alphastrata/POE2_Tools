# asking questions of the Poe2 Passive Tree

You'll see in our 
```rust
#[derive(Debug, Clone, Default)]
pub struct PassiveTree<'data> {
    pub groups: HashMap<GroupId, coordinates::Group>,
    pub nodes: HashMap<NodeId, PoeNode<'data>>,
    pub edges: HashSet<Edge>, // Using a HashSet for bidirectional uniqueness
    pub passive_skills: HashMap<String, skills::PassiveSkill>,
}
``` passive tree struct taht we're creating a graph.

We want to -- and you can look at the tests in pathfinding.rs for examples on what we already have:
1. supply a list of nodes, starting, {$n number of middles} ending by either .name (fuzzy matching), or node_id and create paths that connect them all.

2. we want to for a given path through the tree (Vec<NodeId>), be able to say what are all the _endings_ of the branches, and we want a method that is called frontier nodes, which will achieve this for us, we'll define the concept of a frontier node as a node that has inactive neighbours.

4 we want a helper that can take a list of these frontier nodes and return all the node_id and .stats fields for them.

I like my iterations to be lazy so adhere to that as a constraint.
These implementations should be ON the PassiveTree type i've defined above.
Use my code, use my types, use my conventions throughout your work.
