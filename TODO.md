# **poe_vis**

## **Rendering & Performance**

* [ ] **Cull unseen from render:** We should NOT render/paint nodes outside the camera frustrum.
    1. does this even help? be sure to benchmark it.

* [ ] **Zoom Restriction:** The user should not be able to zoom to a level where parts of the tree are no longer visible. If no nodes are within the camera view, this should be flagged as an issue.
* [ ] **Arcs not lines:** The POE2 tree actually draws beautiful arcs for its `edge`s, but we draw lines. it shouldn't be too hard to say what's the x,y of where we `start` and the x,y of where we `finish` and then work out how to make an arc that makes both nodes positioned on a circle.
    1. use them to do euc dist then use that as the r of the circle? or are do we already know this with the (`calculate_world_position` fn we have on `node`?)

## **Node & Path Handling**

* [ ] **Pruned Node Handling:** When a path is pruned such that downstream nodes become unreachable from the starting node, those unreachable nodes should:
  * Turn red for two seconds.
  * Then be removed from both `PassiveTree.active` and `.highlighted_nodes`.
* [ ] **Virtual Path for Hovered Nodes:** When the `.hovered_node` (on `PassiveTree`) is *not* connected to the starting node, a 'virtual path' should be created and displayed. This path will indicate the route that would be taken if the node were to be selected (similar to maxroll.gg's functionality).

## **User Interface & Interaction**

* [ ] **Configurable Canvas Background:** The canvas background should be configurable via the `#egui` framework and a user configuration file (`user_config.toml`).
* [ ] **Menu Mouse Interaction:** Mouse interactions over menu elements should *not* trigger selection, removal, hover, translation, or zoom functions. This requires attention to the `#egui` implementation.
* [ ] **Camera Home/Reset:** Assign either `'h'` or `Esc` (configurable via settings) to reset/home the camera view.
* [ ] **Fuzzy Search Textline Behaviour:** Pressing `Enter` within the single edit textline of the fuzzy search should close the text line and jump/focus on the topmost search result.
* [ ] **Tab Navigation:** Implement tabbed navigation.

## **Visuals & Configuration**

* [ ] **Icon Integration:** If node icons can be obtained, they should be incorporated into the visual representation.
* [ ] **Expanded Colour Palette:** The colour palette needs expanding. The `user_config.toml` currently only contains approximately eight or nine usable colours.

## **Background Services**

* [ ] **Backend Thread:** The `PassiveTree` should use references to a backend thread that handles all heavy lifting within `background_services.rs`. This should address current performance limitations.
* [ ] **Update Pause:** Implement `'p'` to pause updates, which will temporarily halt the background pathfinding services. This is needed to allow for multi-destination/source breadth-first search (BFS).

---

# **poe_tree**

* [ ] **Stats API:** Create a clean API to access and work with the `.stats`.
  * This should facilitate mathematical operations such as `+`, `-`, `*`, `%`, and `/` on the unconventional data structures in `passive_skills`. Some work has been started on this with the `Operand` concept.
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

* [ ] **Default Starting Stats:** Implement default starting stats for all character classes.
* [ ] **Streaming Pathfinding:** Create streaming versions of the functions in `pathfinding.rs`. This would enable pathfinding calculations to be performed with a delay to facilitate animations.
* [ ] **Manual Character Stats:** In the absence of actual character statistics derived from in-game equipment, allow the user to manually set attributes like `armour`, `evasion`, `energy shield`, etc., perhaps via a `character.toml` file.
