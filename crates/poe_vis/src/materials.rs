use bevy::prelude::*;

use crate::resources::UserConfig;

pub struct PoeVisMaterials;

impl Plugin for PoeVisMaterials {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_materials);

        log::debug!("PoeVisMaterials plugin enabled");
    }
}

#[derive(Resource)]
pub struct GameMaterials {
    // Node colors
    pub node_base: Handle<ColorMaterial>,
    pub node_attack: Handle<ColorMaterial>,
    pub node_mana: Handle<ColorMaterial>,
    pub node_dexterity: Handle<ColorMaterial>,
    pub node_intelligence: Handle<ColorMaterial>,
    pub node_strength: Handle<ColorMaterial>,
    pub node_activated: Handle<ColorMaterial>,

    // Edge colors
    pub edge_base: Handle<ColorMaterial>,
    pub edge_activated: Handle<ColorMaterial>,

    // UI colors
    pub background: Handle<ColorMaterial>,
    pub foreground: Handle<ColorMaterial>,
    pub red: Handle<ColorMaterial>,
    pub orange: Handle<ColorMaterial>,
    pub yellow: Handle<ColorMaterial>,
    pub green: Handle<ColorMaterial>,
    pub blue: Handle<ColorMaterial>,
    pub purple: Handle<ColorMaterial>,
    pub cyan: Handle<ColorMaterial>,
}
fn init_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<UserConfig>,
) {
    commands.insert_resource(GameMaterials {
        // Node materials
        node_base: materials.add(parse_hex_color(&config.colors["all_nodes"])),
        node_attack: materials.add(parse_hex_color(&config.colors["attack"])),
        node_mana: materials.add(parse_hex_color(&config.colors["mana"])),
        node_dexterity: materials.add(parse_hex_color(&config.colors["dexterity"])),
        node_intelligence: materials.add(parse_hex_color(&config.colors["intelligence"])),
        node_strength: materials.add(parse_hex_color(&config.colors["strength"])),
        node_activated: materials.add(parse_hex_color(&config.colors["activated_nodes"])),

        // Edge materials
        edge_base: materials.add(parse_hex_color(&config.colors["all_nodes"])),
        edge_activated: materials.add(parse_hex_color(&config.colors["activated_edges"])),

        // UI materials
        background: materials.add(parse_hex_color(&config.colors["background"])),
        foreground: materials.add(parse_hex_color(&config.colors["foreground"])),
        red: materials.add(parse_hex_color(&config.colors["red"])),
        orange: materials.add(parse_hex_color(&config.colors["orange"])),
        yellow: materials.add(parse_hex_color(&config.colors["yellow"])),
        green: materials.add(parse_hex_color(&config.colors["green"])),
        blue: materials.add(parse_hex_color(&config.colors["blue"])),
        purple: materials.add(parse_hex_color(&config.colors["purple"])),
        cyan: materials.add(parse_hex_color(&config.colors["cyan"])),
    });
}



 fn parse_hex_color(col_str: &str) -> Color {
    if col_str.starts_with('#') && col_str.len() == 7 {
        let hex = u32::from_str_radix(&col_str[1..7], 16).unwrap_or(0x808080);
        Color::srgb_u8(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    } else {
        Color::srgb(0.48, 0.48, 0.48) // Fallback color (gray)
    }
}
