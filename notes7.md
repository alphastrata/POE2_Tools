We're working on a path-of-exile2 passive tree data tool.

you have access to my rust files.

let's do some refactoring:

lets remove the .connections field from our Node type, as we're going to store the 'edges' which is what they are in a separate collection.

so we're going to take the passive tree from :

```rust


#[derive(Debug, Clone, Default)]
pub struct PassiveTree {
    pub groups: HashMap<usize, coordinates::Group>,
    pub nodes: HashMap<usize, Node>,
    pub passive_skills: HashMap<String, skills::PassiveSkill>,
}


```

to

```rust
type GroupId=usize;
type NodeId = usize;

#[derive(Debug, Clone, Default)]
pub struct PassiveTree {
    pub groups: HashMap<GroupID, coordinates::Group>,
    pub nodes: HashMap<NodeID, Node>,
    pub edges: // some sort of bidirectional map where we could call a .get on either of the two usizes(representing NodeIDs) to get to the other... do you know of such a structure?
    pub passive_skills: HashMap<String, skills::PassiveSkill>,
}
```

Create an edge type:
```rust
struct Edge(from:NodeId, to:NodeId);

when constructing edges, we'll look at the euclidian absolute distance from 0,0 around which the tree is built, and we'll take the closer one as the `from` (always).
impl Edge{
    // you decide if we need some methods here...
}
```

So Node will go from:

```rust
#[derive(Debug, Clone, Default)]
pub struct PoeNode<'stat> { // note rename from Node
    pub node_id: NodeID,
    pub skill_id: Option<String>,
    pub parent: GroupID,
    pub radius: u8, // see consts.rs we can get away with this I think?
    pub position: usize,
    // pub connections: Vec<usize>, //remove
    // Derived data
    pub name: String,
    pub is_notable: bool,
    pub stats: &'stat [Stat],
    pub wx: f64,
    pub wy: f64,
    pub active: bool,
}
```
I'd be kinda cool if we could think of a way to optimize the Strings too ey...

Refactor the entirety of all code required after we make the changes to the data holding structs above.

Once refactoring is done we'll also be adding these methods:

```rust
impl PoeNode{
   fn path_to_target(&self, &target: NodeId) -> impl Iterator<Item=NodeId>;

   fn distance_to(&self, other: Self) -> usize;





}

impl PassiveTree{
    // we already have find shortest path.
    rename load_tree to -> from_file, have it return the Self and the raw-data.
}

```


