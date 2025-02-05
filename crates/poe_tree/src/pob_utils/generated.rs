//! MACHINE GENERATED DO NOT EDIT BY HAND

use serde::{Deserialize, Serialize};

use crate::character::CharacterClass;

/// This is a type to represent the PathOfBuilding's build schema, sourced here:
/// https://github.com/PathOfBuildingCommunity/PathOfBuilding/blob/85099b03f0a79b19b3a859ff15bea19c5bcda6af/spec/TestBuilds/3.13/OccVortex.xml
/// The rust type etc is machine generated from that && an exported empty character, see ./data/empty-build.xml
//TODO: add the script to the repo.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "PathOfBuilding2")]
pub struct POBCharacter {
    #[serde(rename = "Build")]
    pub build: Build,
    #[serde(rename = "Import")]
    pub import: Import,
    #[serde(rename = "Party")]
    pub party: Party,
    #[serde(rename = "Tree")]
    pub tree: Tree,
    #[serde(rename = "Notes")]
    pub notes: Notes,
    #[serde(rename = "Skills")]
    pub skills: Skills,
    #[serde(rename = "Calcs")]
    pub calcs: Calcs,
    #[serde(rename = "TreeView")]
    pub tree_view: TreeView,
    #[serde(rename = "Items")]
    pub items: Items,
    #[serde(rename = "Config")]
    pub config: Config,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    #[serde(rename = "@level")]
    pub level: String,
    // Note: we use OUR type here as it's trivially deserialiseable from our own methods etc.
    #[serde(rename = "@className")]
    pub class_name: CharacterClass,
    #[serde(rename = "@ascendClassName")]
    pub ascend_class_name: String,
    #[serde(rename = "@targetVersion")]
    pub target_version: String,
    #[serde(rename = "@characterLevelAutoMode")]
    pub character_level_auto_mode: String,
    #[serde(rename = "@mainSocketGroup")]
    pub main_socket_group: String,
    #[serde(rename = "@viewMode")]
    pub view_mode: String,
    #[serde(rename = "PlayerStat", default)]
    pub player_stats: Vec<PlayerStat>,
    #[serde(rename = "TimelessData")]
    pub timeless_data: TimelessData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerStat {
    #[serde(rename = "@stat")]
    pub stat: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimelessData {
    #[serde(rename = "@devotionVariant2")]
    pub devotion_variant2: String,
    #[serde(rename = "@searchListFallback")]
    pub search_list_fallback: String,
    #[serde(rename = "@searchList")]
    pub search_list: String,
    #[serde(rename = "@devotionVariant1")]
    pub devotion_variant1: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Import {
    #[serde(rename = "@exportParty")]
    pub export_party: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
    #[serde(rename = "@destination")]
    pub destination: String,
    #[serde(rename = "@ShowAdvanceTools")]
    pub show_advance_tools: String,
    #[serde(rename = "@append")]
    pub append: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    #[serde(rename = "@activeSpec")]
    pub active_spec: String,
    #[serde(rename = "Spec")]
    pub spec: Spec,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    #[serde(rename = "@masteryEffects")]
    pub mastery_effects: String,
    #[serde(rename = "@ascendClassId")]
    pub ascend_class_id: String,
    #[serde(rename = "@nodes")]
    pub nodes: String,
    #[serde(rename = "@secondaryAscendClassId")]
    pub secondary_ascend_class_id: String,
    #[serde(rename = "@treeVersion")]
    pub tree_version: String,
    #[serde(rename = "@classId")]
    pub class_id: String,
    #[serde(rename = "URL")]
    pub url: URL,
    #[serde(rename = "Sockets")]
    pub sockets: Option<String>,
    #[serde(rename = "Overrides")]
    pub overrides: Overrides,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct URL {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Overrides {
    #[serde(rename = "AttributeOverride")]
    pub attribute_override: AttributeOverride,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttributeOverride {
    #[serde(rename = "@dexNodes")]
    pub dex_nodes: String,
    #[serde(rename = "@intNodes")]
    pub int_nodes: String,
    #[serde(rename = "@strNodes")]
    pub str_nodes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notes {
    #[serde(rename = "$value")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skills {
    #[serde(rename = "@sortGemsByDPSField")]
    pub sort_gems_by_dps_field: String,
    #[serde(rename = "@activeSkillSet")]
    pub active_skill_set: String,
    #[serde(rename = "@sortGemsByDPS")]
    pub sort_gems_by_dps: String,
    #[serde(rename = "@defaultGemQuality")]
    pub default_gem_quality: String,
    #[serde(rename = "@defaultGemLevel")]
    pub default_gem_level: String,
    #[serde(rename = "@showSupportGemTypes")]
    pub show_support_gem_types: String,
    #[serde(rename = "SkillSet")]
    pub skill_set: SkillSet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillSet {
    #[serde(rename = "@id")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Calcs {
    #[serde(rename = "Input", default)]
    pub inputs: Vec<Input>,
    #[serde(rename = "Section", default)]
    pub sections: Vec<Section>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@string")]
    pub string: Option<String>,
    #[serde(rename = "@number")]
    pub number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    #[serde(rename = "@subsection")]
    pub subsection: String,
    #[serde(rename = "@collapsed")]
    pub collapsed: String,
    #[serde(rename = "@id")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeView {
    #[serde(rename = "@searchStr")]
    pub search_str: String,
    #[serde(rename = "@zoomY")]
    pub zoom_y: String,
    #[serde(rename = "@zoomLevel")]
    pub zoom_level: String,
    #[serde(rename = "@showStatDifferences")]
    pub show_stat_differences: String,
    #[serde(rename = "@zoomX")]
    pub zoom_x: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Items {
    #[serde(rename = "@activeItemSet")]
    pub active_item_set: String,
    #[serde(rename = "@showStatDifferences")]
    pub show_stat_differences: String,
    #[serde(rename = "@useSecondWeaponSet")]
    pub use_second_weapon_set: Option<String>,
    #[serde(rename = "ItemSet")]
    pub item_set: ItemSet,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ItemSlot {
    Slot(Slot),
    SocketIdURL(SocketIdURL),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketIdURL {
    #[serde(rename = "@nodeId")]
    pub node_id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@itemPbURL")]
    pub item_pb_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemSet {
    #[serde(rename = "@useSecondWeaponSet")]
    pub use_second_weapon_set: Option<String>,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "$value", default)]
    pub slots: Vec<ItemSlot>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    #[serde(rename = "@itemPbURL")]
    pub item_pb_url: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@itemId")]
    pub item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "@activeConfigSet")]
    pub active_config_set: String,
    #[serde(rename = "ConfigSet")]
    pub config_set: ConfigSet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSet {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "Placeholder", default)]
    pub placeholders: Vec<Placeholder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Placeholder {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@number")]
    pub number: String,
}
