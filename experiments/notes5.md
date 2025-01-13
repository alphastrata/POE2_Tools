

# Review:
I think we have a good start in main.rs, but we need a few things.

1. We shouldn't allow a user to scroll 'off' the page i.e if the furthest node (from any direction) if positioned at the midpoint of our screen (i.e the center of what the camera can see) we should not allow further translating with wads.

2. We are doing a lot of casting, just use `f64` throughout in all our structs etc so that we never need to convert `f32`s into `f64`s and so on.

3. we need the nodes to be rendered as a % of screen space so that when we zoom in/out they don't appear to change in size.

4. rather than moving to 'Flow Like Water' with `h` we need to instead have a `f` opens a window we can fuzzy-find then select a node from, jumping to that node.

5. when we click a node, it needs to be Active=true.

6. Active nodes should have some sort of highlighting.

7. We should break our code up soon, so put things into `pub mod xyz{}`

8. all the `NodeId` and `GroupId` and such are always positive integers, so we can actually use a `usize` for all of them.

9. the number of stats in a `PassiveSkill` is never more than 3 so, storing a `HashMap` for this is silly, just use a `Vec<(String, f32)>`
```rust
#[derive(Debug, Clone)]
struct PassiveSkill {
    name: Option<String>,
    is_notable: bool,
    stats: HashMap<String, f32>, // Should be  Vec<(String, f32)>
}
```

10. Suggest storing the updated nodes here:
```rust

struct TreeVisualization {
    data: TreeData,
    camera: Camera,
    hovered_node: Option<i64>,
    active_nodes: Vec<&Node>, // or similar.
}
```

11. Re 'Active' nodes:
```rust
#[derive(Debug, Clone)]
struct Node {
    skill_id: Option<String>, // Should be a pointer to the PassiveSkill's .name
    parent: i64,    // group ID
    radius: i64,    // orbit index
    position: i64,  // orbit slot
    connections: Vec<i64>,

    // Derived fields for rendering:
    name: String,
    is_notable: bool,
    stats: Vec<(String, f32)>, // Should be a POINTER to the PassiveSkill's .stats we took it from
    // Computed world coords
    wx: f32,
    wy: f32,

    active: bool // we should skip this in Deserializing as it's NOT in the json data.
}
```


12. We should employ a colour scheme, I like the Horizon colour pallette https://github.com/alphastrata/themer/blob/dev/example_output.png 

13. Our hover text should display the .name .stats information.

