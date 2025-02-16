# poe_vis
- [ ] BUG: sometimes active nodes' connecting edge doesn't highlight.
- [ ] RPC circle and rect are kinda shit, replace both with a quad and let's do a custom shader, the aliasing is fucking abhorent.
- [ ] Remove permanent starting node—add UI button and RPC command to set it to None.
~~- [ ] Arcs not lines: Use arcs (like POE2) instead of straight lines. Calculate start and finish points to form an arc (maybe use `calculate_world_position` info, and compare with PoB assets to work out arc-length and radius).
~~
- [ ] Pruned Node Handling: When a pruned path disconnects downstream nodes:
  - Turn them red for two seconds.
  - Remove them from both `PassiveTree.active` and `.highlighted_nodes`.
- [ ] Camera Home/Reset: Bind `'h'` or `Esc` (configurable) to reset the camera view.
- [ ] Fuzzy Search Behaviour: Pressing Enter in the fuzzy search field should close it and focus the top search result.
- [ ] Tab Navigation: Implement tabs.
- [ ] Icon Integration: Incorporate node icons if available.
- [ ] Use arcs with PNGs—choose the right arc once determined.
- [ ] Develop a screenspace shader that tints the background circle in six 60° wedges, blending colours from int → dex → str (and intermediate values).

## [ ] Handle root node by forcing it to == the ONLY real start node from 0 based on class
- [ ] `ClearAll` needs to handle re-root.

- [ ] Replace the 'Attribute Text' with some sorta UI letting em spend

- [ ] Export complex POB builds for path testing.
- [ ] 'Keystone' node building UI, place anchors and path between.
- [ ] right-click dialouge.
- [ ] `keystone` or `anchor` nodes may be `PermanentlyAssigned` component. using 'Pinned'
- [ ] Default Starting Stats: Set up default starting stats for all character classes, build them from POB exports.
- [ ] Streaming Pathfinding: Create streaming versions of functions in `pathfinding.rs` for animation-friendly pathfinding.

## Changelog (Completed Tasks)

- [x] Expanded Colour Palette: Expand the colour choices in `user_config.toml` (currently around eight or nine colours).
- [x] Virtual Path for Hovered Nodes: If `.hovered_node` isn’t connected to the starting node, display a 'virtual path' indicating the potential route (like maxroll.gg).
- [x] Improve performance of `walk_n_steps` (it's too slow):
- [x] Try a CSR `impl`.
  - Consider GPU computation. NO!
- [x] implement a draw_node ONLY IF not a spastic length.
- [x] Higlight for `t` time, then revert.
- [x] Replace the `Gizmo` hilighting with Glyphs (actual geomtery)
- [x] take_while_with_minimums<P, R>(....
- [x] new filtering code moves to filters.rs
- [x] `RPC` Draw a Square, Circle
- [x] Stats API: Create a clean API to work with `.stats` that supports math operations (`+`, `-`, `*`, `%`, `/`) on passive_skill data.
- [x] colour node requests... all tailwind colours are available, just provide those?
- [x] fetch_colours RPC req...
- [x] prune eh non data-carrying 'is_just_icon' passive_skill items.
- [x] POB parser.
- [x] Begin aggregating stats into the UI.
- [x] Top menu for File (Save, Load, Exit, Import, Export, etc.).
- [x] Backend Thread: Have `PassiveTree` reference a backend thread (in `background_services.rs`) for heavy lifting.
      multi-destination/source BFS.
- [x] Expose an RPC interface for external programmatic access.
- [x] Implement the todos.
- [x] Colour node requests (use all Tailwind colours).
- [x] `fetch_colours` RPC request.
- [x] re-add the ascendencies we normally prune with prune.hidden().
- [x] RPC draw square
- [x] RPC draw circle/other shapes
- [x] Move potential builds and the paths2chaos-inoculation visualiser examples from poe_tree into poe_vis (poe_tree becomes just data).
- [x] Implement `take_while_for_n_steps(predicate, num_steps)` to retrieve paths with at least one evasion_rating buff over N steps.
- [x] BUG: typing the move keys' bindings will move the canvas around when we're searching.
- [x] piggybacking on the SearchState for the hover aggregated stats to show matching nodes is bad, because it encircles nodes that we haven't got in our current active build.
- [x] Menu Mouse Interaction: Ensure mouse actions over menu elements don’t trigger selection, removal, hover, translation, or zoom.


