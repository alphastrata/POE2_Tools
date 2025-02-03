# **poe_vis**

- [ ] Move the potential builds and the paths2chaos-inoculation visualiser examples out to poe_vis, from poe_tree. poe_tree should JUST be data.

- [ ] starting node is perma-pegged, new button to set it to None in UI, RPC cmd too.

## RPC

- [x] implement the todos.
- [x] colour node requests... all tailwind colours are available, just provide those?
- [x] fetch_colours RPC req...

## **Rendering & Performance**

- [ ] **Arcs not lines:** The POE2 tree actually draws beautiful arcs for its `edge`s, but we draw lines. it shouldn't be too hard to say what's the x,y of where we `start` and the x,y of where we `finish` and then work out how to make an arc that makes both nodes positioned on a circle.
  1. use them to do euc dist then use that as the r of the circle? or are do we already know this with the (`calculate_world_position` fn we have on `node`?)
     I think we should be able, looking at the PathOfBuilding assets' .pngs of all those arcs work out the relationship between said arc's arc-length and radius etc...

## **Node & Path Handling**

- [ ] **Pruned Node Handling:** When a path is pruned such that downstream nodes become unreachable from the starting node, those unreachable nodes should:
  - Turn red for two seconds.
  - Then be removed from both `PassiveTree.active` and `.highlighted_nodes`.
- [ ] **Virtual Path for Hovered Nodes:** When the `.hovered_node` (on `PassiveTree`) is _not_ connected to the starting node, a 'virtual path' should be created and displayed. This path will indicate the route that would be taken if the node were to be selected (similar to maxroll.gg's functionality).

- [ ] Performance of the `walk_n_steps` is frikkn garbage, I cannot compute 40 on my big rig ><, at least on wangblows...

  - Try a CSR `impl`
  - Maybe you can compute it on ze gpu?

- [ ] implement a `take_while_for_n_steps(p: $predicate, n: $num_steps)` so we can do something like take_while 'evasion_rating' for 20, and get back all the paths with at least 1 evasion_rating mentioning buff.

## **User Interface & Interaction**

- [ ] **Configurable Canvas Background:** The canvas background should be configurable via the `#egui` framework and a user configuration file (`user_config.toml`).
- [ ] **Menu Mouse Interaction:** Mouse interactions over menu elements should _not_ trigger selection, removal, hover, translation, or zoom functions. This requires attention to the `#egui` implementation.
- [ ] **Camera Home/Reset:** Assign either `'h'` or `Esc` (configurable via settings) to reset/home the camera view.
- [ ] **Fuzzy Search Textline Behaviour:** Pressing `Enter` within the single edit textline of the fuzzy search should close the text line and jump/focus on the topmost search result.
- [ ] **Tab Navigation:** Implement tabbed navigation.

## **Visuals & Configuration**

- [ ] **Icon Integration:** If node icons can be obtained, they should be incorporated into the visual representation.
- [ ] **Expanded Colour Palette:** The colour palette needs expanding. The `user_config.toml` currently only contains approximately eight or nine usable colours.
- [ ] We have the Arcs so... do the png thing? (After we work out WHICH arc to use...)

## Custom shading:

- [ ] How do we set the bg? --> make a screenspace shader that slightly tints the circle in 6 wedges of 60deg each, colouring a blend of int -> dex -> str with all in-betweens.

## **Background Services**

- [ ] **Backend Thread:** The `PassiveTree` should use references to a backend thread that handles all heavy lifting within `background_services.rs`. This should address current performance limitations.
- [ ] **Update Pause:** Implement `'p'` to pause updates, which will temporarily halt the background pathfinding services. This is needed to allow for multi-destination/source breadth-first search (BFS).
- [ ] RPC interface for programmatic access from outside `poe_vis`

---

# **poe_tree**

- [x] prune eh non data-carrying 'is_just_icon' passive_skill items.
- [ ] **Stats API:** Create a clean API to access and work with the `.stats`.

  - This should facilitate mathematical operations such as `+`, `-`, `*`, `%`, and `/` on the unconventional data structures in `passive_skills`. Some work has been started on this with the `Operand` concept.
    some examples of the data:

    ```json
     "stun_threshold_if_no_recent_stun1": {
      "name": "Stun Threshold while on Full Life",
      "icon": "skillicons/passives/life1",
      "stats": {
        "stun_threshold_+%_when_not_stunned_recently": 20
      }
    },
      "dexterity26": {
      "name": "Attribute",
      "icon": "skillicons/passives/plusattribute",
      "stats": {
        "display_passive_attribute_text": 1
      }
    },
      "strength104": {
      "name": "Strength",
      "icon": "skillicons/passives/plusstrength",
      "stats": {
        "base_strength": 8
      }
    },
     "flail3": {
      "name": "Flail Damage",
      "icon": "skillicons/passives/macedmg",
      "stats": {
        "flail_damage_+%": 10
      }
    },
      "melee39": {
      "name": "Deadly Flourish",
      "icon": "skillicons/passives/meleeaoenode",
      "stats": {
        "melee_critical_strike_chance_+%": 20,
        "melee_critical_strike_chance_+%_when_on_full_life": 20
      },
      "is_notable": true
    },
    .... and so on...
    ```

    There will be a lot of tedious one-off handling here.

- [ ] **Default Starting Stats:** Implement default starting stats for all character classes.
- [ ] **Streaming Pathfinding:** Create streaming versions of the functions in `pathfinding.rs`. This would enable pathfinding calculations to be performed with a delay to facilitate animations.
- [ ] **Manual Character Stats:** In the absence of actual character statistics derived from in-game equipment, allow the user to manually set attributes like `armour`, `evasion`, `energy shield`, etc., perhaps via a `character.toml` file.
