use poe_tree::PassiveTree;

fn main() {
    let tree = quick_tree();

    for n in tree.nodes().iter() {
        let stats = n.stats;
        /*
        - Pull out nodes like this:
            "evasion2": {
            "name": "Evasion",
            "icon": "skillicons/passives/evade",
            "stats": {
                "evasion_rating_+%": 15
            }
            }

        - And sum them.
        */
        if stats.name.contains("evasion_rating") {
            println!("{}", stats);
        }
    }
}
