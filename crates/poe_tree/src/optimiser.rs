// We're working on an optimiser that is currently populated as you see below:
// this path: [47175, 3936, 46325, 39581, 6839, 5710]
// to this: [47175, 3936, 43164, 5710, 33556, 55473, 46325]
// if 4 swaps allowed.
// for 'melee damage'

#[derive(Resource)]
pub struct Optimiser {
    pub results: Vec<Vec<NodeId>>,
}
