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
