use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::OnceLock;

pub const ORBIT_RADII: [f32; 10] = [
    0.0, 82.0, 162.0, 335.0, 493.0, 662.0, 846.0, 251.0, 1080.0, 1322.0,
];
pub const ORBIT_SLOTS: [u32; 10] = [1, 12, 24, 24, 72, 72, 72, 24, 72, 144];

/// Starting nodes for each character class in the passive tree.
///
/// | Node ID | Name      | Class     | Location       |
/// |---------|-----------|-----------|----------------|
/// | 50459   | RANGER    | Ranger    | 4 o'clock      |
/// | 47175   | WARRIOR   | Warrior   | 8 o'clock      |
/// | 50986   | DUELIST   | Mercenary | 6 o'clock      |
/// | 61525   | TEMPLAR   | Unknown   | 10 o'clock     |
/// | 54447   | WITCH     | Witch     | 12 o'clock     |
/// | 44683   | SIX       | Monk      | 2 o'clock      |
pub const CHAR_START_NODES: [u32; 6] = [
    50459, // RANGER (Ranger, 4 o'clock)
    47175, // WARRIOR (Warrior, 8 o'clock)
    50986, // DUELIST (Mercenary, 6 o'clock)
    61525, // TEMPLAR (Unknown, 10 o'clock)
    54447, // WITCH (Witch, 12 o'clock)
    44683, // SIX (Monk, 2 o'clock)
];

/// Level one nodes grouped by character class, excluding `is_just_icon: true`.
///
/// | Node ID | Name                      | Class          |
/// |---------|---------------------------|----------------|
/// | 10364   | Skill Speed               | Monk           |
/// | 52980   | Evasion and Energy Shield | Monk           |
/// | 56651   | Projectile Damage         | Ranger         |
/// | 59915   | Projectile Damage         | Mercenary      |
/// | 59779   | Armour and Evasion        | Mercenary      |
/// | 38646   | Armour                    | Warrior        |
/// | 3936    | Melee Damage              | Warrior        |
/// | 50084   | Damage                    | Unknown        |
/// | 13855   | Armour and Energy Shield  | Unknown        |
/// | 4739    | Spell Damage              | Sorcerer/Witch |
/// | 44871   | Energy Shield             | Sorcerer/Witch |
pub const LEVEL_ONE_NODES: [u32; 12] = [
    10364, 52980, // Monk: Skill Speed, Evasion and Energy Shield
    56651, 13828, // Ranger: Projectile Damage
    59915, 59779, // Mercenary: Projectile Damage, Armour and Evasion
    38646, 3936, // Warrior: Armour, Melee Damage
    50084, 13855, // Unknown: Damage, Armour and Energy Shield
    4739, 44871, // Sorcerer/Witch: Spell Damage, Energy Shield
];

static LEVEL_ONE_NODES_MAP: LazyLock<HashMap<&'static str, [u32; 2]>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("Monk", [10364, 52980]);
    m.insert("Ranger", [56651, 13828]);
    m.insert("Mercenary", [59915, 59779]);
    m.insert("Warrior", [38646, 3936]);
    m.insert("Unknown", [50084, 13855]);
    m.insert("Sorcerer/Witch", [4739, 44871]);
    m
});

pub fn get_level_one_nodes() -> &'static HashMap<&'static str, [u32; 2]> {
    &*LEVEL_ONE_NODES_MAP
}
