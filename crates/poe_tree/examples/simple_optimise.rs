// We're working on an optimiser that is currently populated as you see below:
// this path: [47175, 3936, 46325, 39581, 6839, 5710]
// to this: [47175, 3936, 43164, 5710, 33556, 55473, 46325]
// if 4 swaps allowed.
// for 'melee damage'

// The optimiser is very simple atm... just this.
// fn populate_optimiser(
//     mut optimiser: ResMut<Optimiser>,
//     tree: Res<PassiveTreeWrapper>,
//     active_character: Res<ActiveCharacter>,
//     mut req: EventReader<OptimiseReq>,
// ) {
//     log::trace!("Optimise requested");
//     req.read().for_each(|req| {
//         if optimiser.is_available() {
//             optimiser.set_busy();
//             optimiser.results = tree
//                 .branches(&active_character.activated_node_ids)
//                 .iter()
//                 .flat_map(|opt| tree.take_while(*opt, &req.selector, req.delta))
//                 .collect();
//         }
//         optimiser.set_available();
//     })
// }

#[derive(Resource)]
pub struct Optimiser {
    pub results: Vec<Vec<NodeId>>,
    pub starting_path: Vec<NodeId>,
    pub allowable_swaps: u8,
}

use poe_tree::{stats::Stat, type_wrappings::NodeId};
mod common;

fn main() {
    let mut tree = common::quick_tree();

    tree.remove_hidden();

    let start_warrior: NodeId = 47175;

    let starting_path = [47175, 3936, 46325, 39581, 6839, 5710];
    let allowable_delta = 4;
    let best_path = [47175, 3936, 43164, 5710, 33556, 55473, 46325];

    let s = |s: &Stat| {
        matches!(
            s,
            Stat::MeleeDamage(_)
                | Stat::PhysicalDamage(_)
                | Stat::AttackDamage(_)
                | Stat::MeleeDamageAtCloseRange(_)
        )
    };
    let branchpoints = tree.branches(starting_path);
    let searchspace = {
        branchpoints
            .iter()
            .flat_map(|b| tree.take_while(b, s, allowable_delta))
    };
}
