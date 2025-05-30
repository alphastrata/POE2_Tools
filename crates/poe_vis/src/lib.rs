#![feature(box_into_inner)]
#![allow(dead_code, unused_imports)]
#![allow(clippy::type_complexity)]
use bevy::{
    color::palettes::tailwind,
    prelude::{App, *},
};

use background_services::BGServicesPlugin;
use camera::PoeVisCameraPlugin;
use characters::CharacterPlugin;
use config::UserConfigPlugin;
use hotkeys::HotkeysPlugin;
use init_tree::TreeCanvasPlugin;
use materials::PoeVisMaterials;
use mouse::MouseControlsPlugin;
use overlays_n_popups::OverlaysAndPopupsPlugin;
use remote::RPCPlugin;
use search::SearchToolsPlugin;
use ui::UIPlugin;

mod background_services;
mod camera;
mod characters;
pub mod components; // Pub because used in benchmarks
mod config;
mod consts;
mod events;
mod hotkeys;
mod init_tree;
mod materials;
mod mouse;
mod overlays_n_popups;
mod remote;
pub mod resources;
mod search;
mod shaders;
mod ui;

pub struct PoeVis;

impl Plugin for PoeVis {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            //TODO: CFG FLAG
            RPCPlugin,
            // ALWAYS
            BGServicesPlugin,
            PoeVisCameraPlugin,
            TreeCanvasPlugin,
            CharacterPlugin,
            PoeVisMaterials,
            MouseControlsPlugin,
            UserConfigPlugin,
            SearchToolsPlugin,
            OverlaysAndPopupsPlugin,
            // ShadersPlugin
            //  If making RPC Videos suggest disabling these:
            HotkeysPlugin,
            UIPlugin,
        ));
    }
}

#[derive(Resource)]
struct PassiveTreeWrapper {
    tree: poe_tree::PassiveTree,
}
impl std::ops::Deref for PassiveTreeWrapper {
    type Target = poe_tree::PassiveTree;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}
impl std::ops::DerefMut for PassiveTreeWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

pub fn parse_tailwind_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        // AMBER
        "amber-50" => tailwind::AMBER_50.into(),
        "amber-100" => tailwind::AMBER_100.into(),
        "amber-200" => tailwind::AMBER_200.into(),
        "amber-300" => tailwind::AMBER_300.into(),
        "amber-400" => tailwind::AMBER_400.into(),
        "amber-500" => tailwind::AMBER_500.into(),
        "amber-600" => tailwind::AMBER_600.into(),
        "amber-700" => tailwind::AMBER_700.into(),
        "amber-800" => tailwind::AMBER_800.into(),
        "amber-900" => tailwind::AMBER_900.into(),
        "amber-950" => tailwind::AMBER_950.into(),
        // BLUE
        "blue-50" => tailwind::BLUE_50.into(),
        "blue-100" => tailwind::BLUE_100.into(),
        "blue-200" => tailwind::BLUE_200.into(),
        "blue-300" => tailwind::BLUE_300.into(),
        "blue-400" => tailwind::BLUE_400.into(),
        "blue-500" => tailwind::BLUE_500.into(),
        "blue-600" => tailwind::BLUE_600.into(),
        "blue-700" => tailwind::BLUE_700.into(),
        "blue-800" => tailwind::BLUE_800.into(),
        "blue-900" => tailwind::BLUE_900.into(),
        "blue-950" => tailwind::BLUE_950.into(),
        // CYAN
        "cyan-50" => tailwind::CYAN_50.into(),
        "cyan-100" => tailwind::CYAN_100.into(),
        "cyan-200" => tailwind::CYAN_200.into(),
        "cyan-300" => tailwind::CYAN_300.into(),
        "cyan-400" => tailwind::CYAN_400.into(),
        "cyan-500" => tailwind::CYAN_500.into(),
        "cyan-600" => tailwind::CYAN_600.into(),
        "cyan-700" => tailwind::CYAN_700.into(),
        "cyan-800" => tailwind::CYAN_800.into(),
        "cyan-900" => tailwind::CYAN_900.into(),
        "cyan-950" => tailwind::CYAN_950.into(),
        // EMERALD
        "emerald-50" => tailwind::EMERALD_50.into(),
        "emerald-100" => tailwind::EMERALD_100.into(),
        "emerald-200" => tailwind::EMERALD_200.into(),
        "emerald-300" => tailwind::EMERALD_300.into(),
        "emerald-400" => tailwind::EMERALD_400.into(),
        "emerald-500" => tailwind::EMERALD_500.into(),
        "emerald-600" => tailwind::EMERALD_600.into(),
        "emerald-700" => tailwind::EMERALD_700.into(),
        "emerald-800" => tailwind::EMERALD_800.into(),
        "emerald-900" => tailwind::EMERALD_900.into(),
        "emerald-950" => tailwind::EMERALD_950.into(),
        // FUCHSIA
        "fuchsia-50" => tailwind::FUCHSIA_50.into(),
        "fuchsia-100" => tailwind::FUCHSIA_100.into(),
        "fuchsia-200" => tailwind::FUCHSIA_200.into(),
        "fuchsia-300" => tailwind::FUCHSIA_300.into(),
        "fuchsia-400" => tailwind::FUCHSIA_400.into(),
        "fuchsia-500" => tailwind::FUCHSIA_500.into(),
        "fuchsia-600" => tailwind::FUCHSIA_600.into(),
        "fuchsia-700" => tailwind::FUCHSIA_700.into(),
        "fuchsia-800" => tailwind::FUCHSIA_800.into(),
        "fuchsia-900" => tailwind::FUCHSIA_900.into(),
        "fuchsia-950" => tailwind::FUCHSIA_950.into(),
        // GRAY
        "gray-50" => tailwind::GRAY_50.into(),
        "gray-100" => tailwind::GRAY_100.into(),
        "gray-200" => tailwind::GRAY_200.into(),
        "gray-300" => tailwind::GRAY_300.into(),
        "gray-400" => tailwind::GRAY_400.into(),
        "gray-500" => tailwind::GRAY_500.into(),
        "gray-600" => tailwind::GRAY_600.into(),
        "gray-700" => tailwind::GRAY_700.into(),
        "gray-800" => tailwind::GRAY_800.into(),
        "gray-900" => tailwind::GRAY_900.into(),
        "gray-950" => tailwind::GRAY_950.into(),
        // GREEN
        "green-50" => tailwind::GREEN_50.into(),
        "green-100" => tailwind::GREEN_100.into(),
        "green-200" => tailwind::GREEN_200.into(),
        "green-300" => tailwind::GREEN_300.into(),
        "green-400" => tailwind::GREEN_400.into(),
        "green-500" => tailwind::GREEN_500.into(),
        "green-600" => tailwind::GREEN_600.into(),
        "green-700" => tailwind::GREEN_700.into(),
        "green-800" => tailwind::GREEN_800.into(),
        "green-900" => tailwind::GREEN_900.into(),
        "green-950" => tailwind::GREEN_950.into(),
        // INDIGO
        "indigo-50" => tailwind::INDIGO_50.into(),
        "indigo-100" => tailwind::INDIGO_100.into(),
        "indigo-200" => tailwind::INDIGO_200.into(),
        "indigo-300" => tailwind::INDIGO_300.into(),
        "indigo-400" => tailwind::INDIGO_400.into(),
        "indigo-500" => tailwind::INDIGO_500.into(),
        "indigo-600" => tailwind::INDIGO_600.into(),
        "indigo-700" => tailwind::INDIGO_700.into(),
        "indigo-800" => tailwind::INDIGO_800.into(),
        "indigo-900" => tailwind::INDIGO_900.into(),
        "indigo-950" => tailwind::INDIGO_950.into(),
        // LIME
        "lime-50" => tailwind::LIME_50.into(),
        "lime-100" => tailwind::LIME_100.into(),
        "lime-200" => tailwind::LIME_200.into(),
        "lime-300" => tailwind::LIME_300.into(),
        "lime-400" => tailwind::LIME_400.into(),
        "lime-500" => tailwind::LIME_500.into(),
        "lime-600" => tailwind::LIME_600.into(),
        "lime-700" => tailwind::LIME_700.into(),
        "lime-800" => tailwind::LIME_800.into(),
        "lime-900" => tailwind::LIME_900.into(),
        "lime-950" => tailwind::LIME_950.into(),
        // NEUTRAL
        "neutral-50" => tailwind::NEUTRAL_50.into(),
        "neutral-100" => tailwind::NEUTRAL_100.into(),
        "neutral-200" => tailwind::NEUTRAL_200.into(),
        "neutral-300" => tailwind::NEUTRAL_300.into(),
        "neutral-400" => tailwind::NEUTRAL_400.into(),
        "neutral-500" => tailwind::NEUTRAL_500.into(),
        "neutral-600" => tailwind::NEUTRAL_600.into(),
        "neutral-700" => tailwind::NEUTRAL_700.into(),
        "neutral-800" => tailwind::NEUTRAL_800.into(),
        "neutral-900" => tailwind::NEUTRAL_900.into(),
        "neutral-950" => tailwind::NEUTRAL_950.into(),
        // ORANGE
        "orange-50" => tailwind::ORANGE_50.into(),
        "orange-100" => tailwind::ORANGE_100.into(),
        "orange-200" => tailwind::ORANGE_200.into(),
        "orange-300" => tailwind::ORANGE_300.into(),
        "orange-400" => tailwind::ORANGE_400.into(),
        "orange-500" => tailwind::ORANGE_500.into(),
        "orange-600" => tailwind::ORANGE_600.into(),
        "orange-700" => tailwind::ORANGE_700.into(),
        "orange-800" => tailwind::ORANGE_800.into(),
        "orange-900" => tailwind::ORANGE_900.into(),
        "orange-950" => tailwind::ORANGE_950.into(),
        // PINK
        "pink-50" => tailwind::PINK_50.into(),
        "pink-100" => tailwind::PINK_100.into(),
        "pink-200" => tailwind::PINK_200.into(),
        "pink-300" => tailwind::PINK_300.into(),
        "pink-400" => tailwind::PINK_400.into(),
        "pink-500" => tailwind::PINK_500.into(),
        "pink-600" => tailwind::PINK_600.into(),
        "pink-700" => tailwind::PINK_700.into(),
        "pink-800" => tailwind::PINK_800.into(),
        "pink-900" => tailwind::PINK_900.into(),
        "pink-950" => tailwind::PINK_950.into(),
        // PURPLE
        "purple-50" => tailwind::PURPLE_50.into(),
        "purple-100" => tailwind::PURPLE_100.into(),
        "purple-200" => tailwind::PURPLE_200.into(),
        "purple-300" => tailwind::PURPLE_300.into(),
        "purple-400" => tailwind::PURPLE_400.into(),
        "purple-500" => tailwind::PURPLE_500.into(),
        "purple-600" => tailwind::PURPLE_600.into(),
        "purple-700" => tailwind::PURPLE_700.into(),
        "purple-800" => tailwind::PURPLE_800.into(),
        "purple-900" => tailwind::PURPLE_900.into(),
        "purple-950" => tailwind::PURPLE_950.into(),
        // RED
        "red-50" => tailwind::RED_50.into(),
        "red-100" => tailwind::RED_100.into(),
        "red-200" => tailwind::RED_200.into(),
        "red-300" => tailwind::RED_300.into(),
        "red-400" => tailwind::RED_400.into(),
        "red-500" => tailwind::RED_500.into(),
        "red-600" => tailwind::RED_600.into(),
        "red-700" => tailwind::RED_700.into(),
        "red-800" => tailwind::RED_800.into(),
        "red-900" => tailwind::RED_900.into(),
        "red-950" => tailwind::RED_950.into(),
        // ROSE
        "rose-50" => tailwind::ROSE_50.into(),
        "rose-100" => tailwind::ROSE_100.into(),
        "rose-200" => tailwind::ROSE_200.into(),
        "rose-300" => tailwind::ROSE_300.into(),
        "rose-400" => tailwind::ROSE_400.into(),
        "rose-500" => tailwind::ROSE_500.into(),
        "rose-600" => tailwind::ROSE_600.into(),
        "rose-700" => tailwind::ROSE_700.into(),
        "rose-800" => tailwind::ROSE_800.into(),
        "rose-900" => tailwind::ROSE_900.into(),
        "rose-950" => tailwind::ROSE_950.into(),
        // SKY
        "sky-50" => tailwind::SKY_50.into(),
        "sky-100" => tailwind::SKY_100.into(),
        "sky-200" => tailwind::SKY_200.into(),
        "sky-300" => tailwind::SKY_300.into(),
        "sky-400" => tailwind::SKY_400.into(),
        "sky-500" => tailwind::SKY_500.into(),
        "sky-600" => tailwind::SKY_600.into(),
        "sky-700" => tailwind::SKY_700.into(),
        "sky-800" => tailwind::SKY_800.into(),
        "sky-900" => tailwind::SKY_900.into(),
        "sky-950" => tailwind::SKY_950.into(),
        // SLATE
        "slate-50" => tailwind::SLATE_50.into(),
        "slate-100" => tailwind::SLATE_100.into(),
        "slate-200" => tailwind::SLATE_200.into(),
        "slate-300" => tailwind::SLATE_300.into(),
        "slate-400" => tailwind::SLATE_400.into(),
        "slate-500" => tailwind::SLATE_500.into(),
        "slate-600" => tailwind::SLATE_600.into(),
        "slate-700" => tailwind::SLATE_700.into(),
        "slate-800" => tailwind::SLATE_800.into(),
        "slate-900" => tailwind::SLATE_900.into(),
        "slate-950" => tailwind::SLATE_950.into(),
        // STONE
        "stone-50" => tailwind::STONE_50.into(),
        "stone-100" => tailwind::STONE_100.into(),
        "stone-200" => tailwind::STONE_200.into(),
        "stone-300" => tailwind::STONE_300.into(),
        "stone-400" => tailwind::STONE_400.into(),
        "stone-500" => tailwind::STONE_500.into(),
        "stone-600" => tailwind::STONE_600.into(),
        "stone-700" => tailwind::STONE_700.into(),
        "stone-800" => tailwind::STONE_800.into(),
        "stone-900" => tailwind::STONE_900.into(),
        "stone-950" => tailwind::STONE_950.into(),
        // TEAL
        "teal-50" => tailwind::TEAL_50.into(),
        "teal-100" => tailwind::TEAL_100.into(),
        "teal-200" => tailwind::TEAL_200.into(),
        "teal-300" => tailwind::TEAL_300.into(),
        "teal-400" => tailwind::TEAL_400.into(),
        "teal-500" => tailwind::TEAL_500.into(),
        "teal-600" => tailwind::TEAL_600.into(),
        "teal-700" => tailwind::TEAL_700.into(),
        "teal-800" => tailwind::TEAL_800.into(),
        "teal-900" => tailwind::TEAL_900.into(),
        "teal-950" => tailwind::TEAL_950.into(),
        // VIOLET
        "violet-50" => tailwind::VIOLET_50.into(),
        "violet-100" => tailwind::VIOLET_100.into(),
        "violet-200" => tailwind::VIOLET_200.into(),
        "violet-300" => tailwind::VIOLET_300.into(),
        "violet-400" => tailwind::VIOLET_400.into(),
        "violet-500" => tailwind::VIOLET_500.into(),
        "violet-600" => tailwind::VIOLET_600.into(),
        "violet-700" => tailwind::VIOLET_700.into(),
        "violet-800" => tailwind::VIOLET_800.into(),
        "violet-900" => tailwind::VIOLET_900.into(),
        "violet-950" => tailwind::VIOLET_950.into(),
        // YELLOW
        "yellow-50" => tailwind::YELLOW_50.into(),
        "yellow-100" => tailwind::YELLOW_100.into(),
        "yellow-200" => tailwind::YELLOW_200.into(),
        "yellow-300" => tailwind::YELLOW_300.into(),
        "yellow-400" => tailwind::YELLOW_400.into(),
        "yellow-500" => tailwind::YELLOW_500.into(),
        "yellow-600" => tailwind::YELLOW_600.into(),
        "yellow-700" => tailwind::YELLOW_700.into(),
        "yellow-800" => tailwind::YELLOW_800.into(),
        "yellow-900" => tailwind::YELLOW_900.into(),
        "yellow-950" => tailwind::YELLOW_950.into(),
        // ZINC
        "zinc-50" => tailwind::ZINC_50.into(),
        "zinc-100" => tailwind::ZINC_100.into(),
        "zinc-200" => tailwind::ZINC_200.into(),
        "zinc-300" => tailwind::ZINC_300.into(),
        "zinc-400" => tailwind::ZINC_400.into(),
        "zinc-500" => tailwind::ZINC_500.into(),
        "zinc-600" => tailwind::ZINC_600.into(),
        "zinc-700" => tailwind::ZINC_700.into(),
        "zinc-800" => tailwind::ZINC_800.into(),
        "zinc-900" => tailwind::ZINC_900.into(),
        "zinc-950" => tailwind::ZINC_950.into(),
        _ => Color::WHITE,
    }
}
