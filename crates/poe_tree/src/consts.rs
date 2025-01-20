//$ crates/poe_tree/src/consts.rs
pub const ORBIT_RADII: [f64; 10] = [
    0.0, 82.0, 162.0, 335.0, 493.0, 662.0, 846.0, 251.0, 1080.0, 1322.0,
];
pub const ORBIT_SLOTS: [usize; 10] = [1, 12, 24, 24, 72, 72, 72, 24, 72, 144];

pub const CHAR_START_NODES: [usize; 6] = [
    50459, // Ranger 4 o'clock
    47175, // Warrior 8'oclock
    50986, // Mercenary 6'oclock
    61525, // 10 oclock ?? mystery character...
    54447, // Witch top  12'oclock, Sorceress too..
    44683, // Monk 2'clock
];
