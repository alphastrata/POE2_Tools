/// If you want a default fade time, you can store it here or in NodeScaling
pub const DEFAULT_HOVER_FADE_TIME: f32 = 0.760;

// Min values that we use for NodeScaling
pub const NODE_MIN_SCALE: f32 = 1.0;
pub const NODE_MAX_SCALE: f32 = 6.0;
pub const NODE_BASE_RADIUS: f32 = 7.2;
pub const NODE_HOVER_MULTIPLIER: f32 = 1.06;
pub const NODE_HOVER_FADE_TIME: f32 = 0.120;

/// In the game you can reach level 99 + 24 skillpoints from quests.
pub const MAXIMUM_ACTIVATEABLE_NODES: u8 = 123;

pub const NODE_PLACEMENT_Z_IDX: f32 = 0.0;

pub const EDGE_PLACEMENT_Z_IDX: f32 = -10.0;

/// The length a query MUST be in the searchbox before we allow the searching to begin...
pub const SEARCH_THRESHOLD: usize = 5;

pub const DEFAULT_SEARCH_HIGHLIGHT_DURATION: f32 = 60.0; //seconds

/// Default path to which we'll save a character.
pub const DEFAULT_SAVE_PATH: &str = "data/character.toml";

pub const TAILWIND_COLOURS_AS_STR: [&str; 242] = [
    "amber-50",
    "amber-100",
    "amber-200",
    "amber-300",
    "amber-400",
    "amber-500",
    "amber-600",
    "amber-700",
    "amber-800",
    "amber-900",
    "amber-950",
    "blue-50",
    "blue-100",
    "blue-200",
    "blue-300",
    "blue-400",
    "blue-500",
    "blue-600",
    "blue-700",
    "blue-800",
    "blue-900",
    "blue-950",
    "cyan-50",
    "cyan-100",
    "cyan-200",
    "cyan-300",
    "cyan-400",
    "cyan-500",
    "cyan-600",
    "cyan-700",
    "cyan-800",
    "cyan-900",
    "cyan-950",
    "emerald-50",
    "emerald-100",
    "emerald-200",
    "emerald-300",
    "emerald-400",
    "emerald-500",
    "emerald-600",
    "emerald-700",
    "emerald-800",
    "emerald-900",
    "emerald-950",
    "fuchsia-50",
    "fuchsia-100",
    "fuchsia-200",
    "fuchsia-300",
    "fuchsia-400",
    "fuchsia-500",
    "fuchsia-600",
    "fuchsia-700",
    "fuchsia-800",
    "fuchsia-900",
    "fuchsia-950",
    "gray-50",
    "gray-100",
    "gray-200",
    "gray-300",
    "gray-400",
    "gray-500",
    "gray-600",
    "gray-700",
    "gray-800",
    "gray-900",
    "gray-950",
    "green-50",
    "green-100",
    "green-200",
    "green-300",
    "green-400",
    "green-500",
    "green-600",
    "green-700",
    "green-800",
    "green-900",
    "green-950",
    "indigo-50",
    "indigo-100",
    "indigo-200",
    "indigo-300",
    "indigo-400",
    "indigo-500",
    "indigo-600",
    "indigo-700",
    "indigo-800",
    "indigo-900",
    "indigo-950",
    "lime-50",
    "lime-100",
    "lime-200",
    "lime-300",
    "lime-400",
    "lime-500",
    "lime-600",
    "lime-700",
    "lime-800",
    "lime-900",
    "lime-950",
    "neutral-50",
    "neutral-100",
    "neutral-200",
    "neutral-300",
    "neutral-400",
    "neutral-500",
    "neutral-600",
    "neutral-700",
    "neutral-800",
    "neutral-900",
    "neutral-950",
    "orange-50",
    "orange-100",
    "orange-200",
    "orange-300",
    "orange-400",
    "orange-500",
    "orange-600",
    "orange-700",
    "orange-800",
    "orange-900",
    "orange-950",
    "pink-50",
    "pink-100",
    "pink-200",
    "pink-300",
    "pink-400",
    "pink-500",
    "pink-600",
    "pink-700",
    "pink-800",
    "pink-900",
    "pink-950",
    "purple-50",
    "purple-100",
    "purple-200",
    "purple-300",
    "purple-400",
    "purple-500",
    "purple-600",
    "purple-700",
    "purple-800",
    "purple-900",
    "purple-950",
    "red-50",
    "red-100",
    "red-200",
    "red-300",
    "red-400",
    "red-500",
    "red-600",
    "red-700",
    "red-800",
    "red-900",
    "red-950",
    "rose-50",
    "rose-100",
    "rose-200",
    "rose-300",
    "rose-400",
    "rose-500",
    "rose-600",
    "rose-700",
    "rose-800",
    "rose-900",
    "rose-950",
    "sky-50",
    "sky-100",
    "sky-200",
    "sky-300",
    "sky-400",
    "sky-500",
    "sky-600",
    "sky-700",
    "sky-800",
    "sky-900",
    "sky-950",
    "slate-50",
    "slate-100",
    "slate-200",
    "slate-300",
    "slate-400",
    "slate-500",
    "slate-600",
    "slate-700",
    "slate-800",
    "slate-900",
    "slate-950",
    "stone-50",
    "stone-100",
    "stone-200",
    "stone-300",
    "stone-400",
    "stone-500",
    "stone-600",
    "stone-700",
    "stone-800",
    "stone-900",
    "stone-950",
    "teal-50",
    "teal-100",
    "teal-200",
    "teal-300",
    "teal-400",
    "teal-500",
    "teal-600",
    "teal-700",
    "teal-800",
    "teal-900",
    "teal-950",
    "violet-50",
    "violet-100",
    "violet-200",
    "violet-300",
    "violet-400",
    "violet-500",
    "violet-600",
    "violet-700",
    "violet-800",
    "violet-900",
    "violet-950",
    "yellow-50",
    "yellow-100",
    "yellow-200",
    "yellow-300",
    "yellow-400",
    "yellow-500",
    "yellow-600",
    "yellow-700",
    "yellow-800",
    "yellow-900",
    "yellow-950",
    "zinc-50",
    "zinc-100",
    "zinc-200",
    "zinc-300",
    "zinc-400",
    "zinc-500",
    "zinc-600",
    "zinc-700",
    "zinc-800",
    "zinc-900",
    "zinc-950",
];
