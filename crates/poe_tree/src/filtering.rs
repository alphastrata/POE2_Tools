use crate::{stats::Stat, type_wrappings::NodeId, PassiveTree};

pub fn filter_winners<F>(
    ser_res: Vec<Vec<NodeId>>,
    tree: &PassiveTree,
    predicate: F,
) -> Vec<Vec<NodeId>>
where
    F: Fn(&Stat) -> bool,
{
    let mut winners = vec![];

    for potential in ser_res {
        let mut total_bonus = 0.0;

        for n in &potential {
            let pnode = tree.nodes.get(n).unwrap();
            let pskill = tree.passive_for_node(pnode);
            let stats = pskill.stats();

            for s in stats {
                if predicate(s) {
                    total_bonus += s.value();
                }
            }
        }

        // if total_bonus >= MIN_BONUS_VALUE {
        //     winners.push(potential);
        // }
    }

    winners
}
