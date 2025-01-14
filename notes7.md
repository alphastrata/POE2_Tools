
You can see from the project_structure.txt our app is making progress but we have many problems:


## Bugs
1. when we activate the fuzzy searcher the textbox isn't automatically focused.
2. when we NO hover effects, our on-hover code is broken, hovering should highlight the nodes and the edges leaving them.
3. we cannot 'select' a node, selecting a node should be adding its node_id to the Character we have open, and noting that it's selected we should be styling the node differenty (green circle around perimiter, it will need a larger radius to suit...)

Can you explain any of these bugs?

re Bug1's attempt:
```shell
 poo-log  cargo check
    Checking poo-tools v0.1.0 (D:\poo-log)
error[E0061]: this method takes 1 argument but 0 arguments were supplied
   --> src\visualiser.rs:187:27
    |
187 |                     if ui.memory().request_focus(ui.id()) {
    |                           ^^^^^^-- argument #1 is missing
    |
note: method defined here
   --> C:\Users\jer\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\egui-0.30.0\src\ui.rs:785:12
    |
785 |     pub fn memory<R>(&self, reader: impl FnOnce(&Memory) -> R) -> R {
    |            ^^^^^^
help: provide the argument
    |
187 |                     if ui.memory(/* reader */).request_focus(ui.id()) {
    |                                 ~~~~~~~~~~~~~~

error[E0061]: this method takes 1 argument but 0 arguments were supplied
   --> src\visualiser.rs:188:28
    |
188 |                         ui.memory().request_focus(ui.id()); // Ensure focus is requested
    |                            ^^^^^^-- argument #1 is missing
    |
note: method defined here
   --> C:\Users\jer\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\egui-0.30.0\src\ui.rs:785:12
    |
785 |     pub fn memory<R>(&self, reader: impl FnOnce(&Memory) -> R) -> R {
    |            ^^^^^^
help: provide the argument
    |
188 |                         ui.memory(/* reader */).request_focus(ui.id()); // Ensure focus is requested
    |                               

```
> you should check the docs to make sure you are up to date on eframe and egui 0.30.0

re Bug2:
maybe working but it's very very slow...


re Bug3:
```rust
if ui.input(|i| i.pointer.primary_clicked()) {
    if let Some(id) = self.hovered_node {
        if let Some(node) = self.passive_tree.nodes.get_mut(&id) {
            if let Some(character) = &mut self.current_character {
                if character.activated_node_ids.contains(&id) {
                    character.activated_node_ids.retain(|&nid| nid != id);
                } else {
                    character.activated_node_ids.push(id);
                }
                node.active = !node.active;
            }
        }
    }
}

```
you'll need to give me more specific advise about where things go! 


New bugs: 
my user input (keyboard) to move around is not working... hopefully you can tell me why.

I've updated the visualiser.rs code for you to look at the changes i WAS able to make.

It's time we broke up the fn update() call to have some smaller function calls in it to break out functionality because... it's getting