We are working in the poe_vis side of things, see the `projuct_structure.txt` for details.

we have two issues to start with:

1. the edges are being rendered incorrectly. [crates\poe_vis\src\nodes.rs]
2. the node size should scale with the zoom level (i.e the camera position's .z) however it's not working. [crates\poe_vis\src\nodes.rs and crates\poe_vis\src\camera.rs]

let's make these fixes.
