# poe_vis

## Features

### Build & UI

- [ ] Move potential builds and the paths2chaos-inoculation visualiser examples from poe_tree into poe_vis (poe_tree becomes just data).
- [ ] Remove permanent starting node—add UI button and RPC command to set it to None.

### RPC

- [x] Implement the todos.
- [x] Colour node requests (use all Tailwind colours).
- [x] `fetch_colours` RPC request.

### Rendering & Performance

- [ ] **Arcs not lines:** Use arcs (like POE2) instead of straight lines. Calculate start and finish points to form an arc (maybe use `calculate_world_position` info, and compare with PoB assets to work out arc-length and radius).

### Node & Path Handling

- [ ] **Pruned Node Handling:** When a pruned path disconnects downstream nodes:
  - Turn them red for two seconds.
  - Remove them from both `PassiveTree.active` and `.highlighted_nodes`.
- [ ] **Virtual Path for Hovered Nodes:** If `.hovered_node` isn’t connected to the starting node, display a 'virtual path' indicating the potential route (like maxroll.gg).
- [ ] Improve performance of `walk_n_steps` (it's too slow):
  - Try a CSR `impl`.
  - Consider GPU computation.
- [ ] Implement `take_while_for_n_steps(predicate, num_steps)` to retrieve paths with at least one evasion_rating buff over N steps.

### User Interface & Interaction

- [ ] BUG: typing the move keys' bindings will move the canvas around when we're searching.
- [ ] piggybacking on the SearchState for the hover aggregated stats to show matching nodes is bad, because it encircles nodes that we haven't got inour current active build.
- [ ] **Configurable Canvas Background:** Make it configurable via egui and a `user_config.toml`.
- [ ] **Menu Mouse Interaction:** Ensure mouse actions over menu elements don’t trigger selection, removal, hover, translation, or zoom.
- [ ] **Camera Home/Reset:** Bind `'h'` or `Esc` (configurable) to reset the camera view.
- [ ] **Fuzzy Search Behaviour:** Pressing Enter in the fuzzy search field should close it and focus the top search result.
- [ ] **Tab Navigation:** Implement tabs.

### Visuals & Configuration

- [ ] **Icon Integration:** Incorporate node icons if available.
- [ ] **Expanded Colour Palette:** Expand the colour choices in `user_config.toml` (currently around eight or nine colours).
- [ ] Use arcs with PNGs—choose the right arc once determined.

### Custom Shading

- [ ] Develop a screenspace shader that tints the background circle in six 60° wedges, blending colours from int → dex → str (and intermediate values).

### Background Services

- [ ] **Backend Thread:** Have `PassiveTree` reference a backend thread (in `background_services.rs`) for heavy lifting.
- [ ] **Update Pause:** Bind `'p'` to pause background pathfinding services for multi-destination/source BFS.
- [ ] Expose an RPC interface for external programmatic access.

# poe_tree

## Tasks to Complete

- [ ] Higlight for `t` time, then revert.
- [ ] Replace the `Gizmo` hilighting with Glyphs (actual geomtery)
- [ ] Handle root_node being None -> #DirectionTextPlugin? ThrowWarning?
- [ ] Replace the 'Attribute Text' with some sorta UI letting em spend
- [ ] Export complex POB builds for path testing.
- [ ] 'Keystone' node building UI, place anchors and path between.
- [ ] take_while_with_minimums<P, R>(....
- [ ] new filtering code moves to filters.rs
- [ ] `RPC` Draw a Square, Circle
- [ ] `ClearAll` needs to handle re-root.
- [ ] re-add the ascendencies we normally prune with prune.hidden().
- [ ] implement a draw_node ONLY IF not a spastic length.
- [ ] right-click dialouge.
- [ ] `keystone` or `anchor` nodes may be `PermanentlyAssigned` component.

- [ ] **Stats API:** Create a clean API to work with `.stats` that supports math operations (`+`, `-`, `*`, `%`, `/`) on passive_skill data.

  - Example data:

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
    }
    ```

- [ ] **Default Starting Stats:** Set up default starting stats for all character classes, build them from POB exports.
- [ ] **Streaming Pathfinding:** Create streaming versions of functions in `pathfinding.rs` for animation-friendly pathfinding.
- [ ] **Manual Character Stats:** Allow users to manually set stats (armour, evasion, energy shield, etc.) via a `character.toml` in absence of in-game data.

## Changelog (Completed Tasks)

- [x] implement the todos.
- [x] colour node requests... all tailwind colours are available, just provide those?
- [x] fetch_colours RPC req...
- [x] prune eh non data-carrying 'is_just_icon' passive_skill items.
- [x] POB parser.
- [x] Begin aggregating stats into the UI.
- [x] Top menu for File (Save, Load, Exit, Import, Export, etc.).
