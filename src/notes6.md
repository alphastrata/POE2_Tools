You have access to my rust project files, we have the following work to do:

1. Formalise a more strongly typed UserConfig struct to replace the toml from value we use atm, you can see my existing one and notes in the tree_config.toml

2. We currently have 'a' to 'b' path finding, we need to support arbritrary pathing i.e a user should be able to say, I want to go from 'a' to 'b' and arbritrarily add stopping nodes in between.

3. we currently allow you to select nodes, doing so should add the node_ids to a new Config struct we need to create that is for a User's character:

```rust
struct UserCharacter{
    name: str
    activated_node_ids: []
    date_created:
}
```

We should be able to give them a file->load menu option (top menu)/ or use the `load_character_nodes` config binding.
loading a config should override the reference to said config we keep in the `TreeVis` struct I think.

I imagine the activated nodes has a visual aspect to it of a slightly larger than the existing radius of a circle in our config.green.

I also imagine all nodes are a new dark grey (not as dark as our background though) which you can add.

on startup we should load the previously opened character which we can keep in a .tmp file next to our source and just check the cannonical path -- if it exists and parses correcttly we load it, if not we do a default.

When a user 'selects' nodes in the UI we should be updating the UserCharacter and saving it to disk every 5 seconds, or every update whichever is sooner.

You are to implement all of this in perfect idiomatic rust.

Notes for your implementations:

- use iterators over for-loops, unless a continue/break or if/else statement is required.
- keep docs and comments to a minimum unless instrcuted otherwise
