//$ deprecated\ggez_vis.rs

// src/main.rs
use ggez::{conf, event, ContextBuilder, GameResult};
use crate::data::*;
use crate::visual::*;

mod data;
mod visual;

fn main() -> GameResult {
    // Load data your way...
    let tree_data = TreeData {
        passive_tree: PassiveTree { groups: Default::default(), nodes: Default::default() },
        passive_skills: Default::default(),
    };
    let (ctx, events) = ContextBuilder::new("passive_tree", "ggez")
        .window_setup(conf::WindowSetup::default().title("POE2 Tree"))
        .build()?;
    let vis = TreeVisualization::new(tree_data);
    event::run(ctx, events, vis)
}
