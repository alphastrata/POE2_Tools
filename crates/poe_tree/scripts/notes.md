# Decoding the stats

we have to handle:

- `+%`
- `+`
- `-`
- `per` i.e `totems_attack_speed_+%_per_active_totem`

```rust

Plus(f32);
PlusPercentage(f32);
Minus(f32);
MinusPercentage(f32);
Percent(f32);

```

Proompts:

````
right now we have this script: """
Generate unique enum variants for all the skills in the tree..
"""

import json
import re

file_path = "data/POE2_Tree.json"
with open(file_path, "r") as f:
    poe_tree = json.load(f)

# Extract unique stat names and categorize them
stat_categories = {
    "addition (+)": set(),
    "subtraction (-)": set(),
    "multiplication (*)": set(),
    "division (/)": set(),
    "percentage (+%)": set(),
    "other": set(),
}

# Regular expression patterns for categorizing stats
percentage_pattern = re.compile(r"\+%")
addition_pattern = re.compile(r"\+$")
multiplication_pattern = re.compile(r"\*$")
division_pattern = re.compile(r"/")
subtraction_pattern = re.compile(r"-")

# Iterate through passive_skills and categorize stats
for skill in poe_tree.get("passive_skills", {}).values():
    if "is_just_icon" in skill:
        continue

    if "stats" in skill:
        for stat_name, value in skill["stats"].items():
            # Determine category based on the stat name
            if percentage_pattern.search(stat_name):
                stat_categories["percentage (+%)"].add(stat_name)
            elif addition_pattern.search(stat_name):
                stat_categories["addition (+)"].add(stat_name)
            elif multiplication_pattern.search(stat_name):
                stat_categories["multiplication (*)"].add(stat_name)
            elif division_pattern.search(stat_name):
                stat_categories["division (/)"].add(stat_name)
            elif subtraction_pattern.search(stat_name):
                stat_categories["subtraction (-)"].add(stat_name)
            else:
                stat_categories["other"].add(stat_name)


# Define Rust enum structure with mathematical operations and corrected syntax
rust_enum_template = """use std::ops::Add;
use std::ops::AddAssign;

#[derive(Debug, Clone, PartialEq)]
pub enum StatType {{
{variants}
}}

impl StatType {{
    pub fn sum(stats: &[Self]) -> Option<Self> {{
        let mut total = 0.0;
        let mut variant = None;

        for stat in stats {{
            match stat {{
{sum_arms}
                _ => return None, // Cannot sum unknown stat types
            }}
        }}

        variant.map(|v| match v {{
{sum_wrapping}
            _ => unreachable!(),
        }})
    }}
}}

impl Add for StatType {{
    type Output = Option<Self>;

    fn add(self, other: Self) -> Option<Self> {{
        match (self, other) {{
{add_arms}
            _ => None, // Cannot add different types
        }}
    }}
}}

impl AddAssign for StatType {{
    fn add_assign(&mut self, other: Self) {{
        if let Some(result) = self.clone() + other {{
            *self = result;
        }}
    }}
}}

// Basic arithmetic implementation for each variant
macro_rules! impl_basic_maths_for {{
    ($enum_name:ident) => {{
        impl Add for $enum_name {{
            type Output = Self;
            fn add(self, other: Self) -> Self {{
                Self(self.0 + other.0)
            }}
        }}

        impl AddAssign for $enum_name {{
            fn add_assign(&mut self, other: Self) {{
                self.0 += other.0;
            }}
        }}
    }};
}}

// Define wrapper types
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plus(pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Minus(pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multiply(pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Divide(pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlusPercentage(pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinusPercentage(pub f32);

// Implement arithmetic for each variant type
impl_basic_maths_for!(Plus);
impl_basic_maths_for!(Minus);
impl_basic_maths_for!(Multiply);
impl_basic_maths_for!(Divide);
impl_basic_maths_for!(PlusPercentage);
impl_basic_maths_for!(MinusPercentage);
"""

# Mapping of categories to Rust enum variants
category_to_variant = {
    "addition (+)": "Plus",
    "subtraction (-)": "Minus",
    "multiplication (*)": "Multiply",
    "division (/)": "Divide",
    "percentage (+%)": "PlusPercentage",
    "other": "Other",
}

# Handle MinusPercentage separately
minus_percentage_pattern = re.compile(r"-\%")

variants = []
sum_arms = []
sum_wrapping = []
add_arms = []

for category, rust_type in category_to_variant.items():
    for stat_name in stat_categories[category]:
        formatted_name = (
            stat_name.replace("%", "Percent")
            .replace("+", "Plus")
            .replace("-", "Minus")
            .replace("*", "Multiply")
            .replace("/", "Divide")
            .replace(" ", "_")
        )

        # Check for MinusPercentage specifically
        if minus_percentage_pattern.search(stat_name):
            rust_type = "MinusPercentage"

        # Comment out "Other" types
        if category == "other":
            rust_variant = f"    // {formatted_name}({rust_type}),"
        else:
            rust_variant = f"    {formatted_name}({rust_type}),"

        variants.append(rust_variant)

        # Generate match arms for sum function
        if category != "other":
            sum_arms.append(
                f'                Self::{formatted_name}({rust_type}(v)) => {{ total += v; variant = Some("{formatted_name}"); }},'
            )

        # Generate match arms for wrapping sum
        if category != "other":
            sum_wrapping.append(
                f'            "{formatted_name}" => Self::{formatted_name}({rust_type}(total)),'
            )

        # Generate match arms for Add trait
        if category != "other":
            add_arms.append(
                f"            (Self::{formatted_name}({rust_type}(a)), Self::{formatted_name}({rust_type}(b))) => Some(Self::{formatted_name}({rust_type}(a + b))),"
            )

# Generate full Rust enum
rust_enum = rust_enum_template.format(
    variants="\n".join(variants),
    sum_arms="\n".join(sum_arms),
    sum_wrapping="\n".join(sum_wrapping),
    add_arms="\n".join(add_arms),
)

# Output Rust enum to a file
rust_file_path = "stats_enum.rs"
with open(rust_file_path, "w") as f:
    f.write(rust_enum)

# Provide the file to the user
rust_file_path making this... ```rust use std::ops::Add;
use std::ops::AddAssign;

#[derive(Debug, Clone, PartialEq)]
pub enum StatType {
    all_skill_gem_level_Plus(Plus),
    map_pinnacle_boss_difficulty_Plus(Plus),
    map_expedition_chest_marker_count_Plus(Plus),
    maps_with_bosses_additional_shrine_Plus(Plus),
    quarterstaff_critical_strike_multiplier_Plus(Plus),
    support_gem_limit_Plus(Plus), /* snippping */
impl StatType {
    pub fn sum(stats: &[Self]) -> Option<Self> {
        let mut total = 0.0;
        let mut variant = None;

        for stat in stats {
            match stat {
                Self::all_skill_gem_level_Plus(Plus(v)) => { total += v; variant = Some("all_skill_gem_level_Plus"); },
                Self::map_pinnacle_boss_difficulty_Plus(Plus(v)) => { total += v; variant = Some("map_pinnacle_boss_difficulty_Plus"); },
                Self::map_expedition_chest_marker_count_Plus(Plus(v)) => { total += v; variant = Some("map_expedition_chest_marker_count_Plus"); },
                Self::maps_with_bosses_additional_shrine_Plus(Plus(v)) => { total += v; variant = Some("maps_with_bosses_additional_shrine_Plus"); },
                Self::quarterstaff_critical_strike_multiplier_Plus(Plus(v)) => { total += v; variant = Some("quarterstaff_critical_strike_multiplier_Plus"); },
                Self::support_gem_limit_Plus(Plus(v)) => { total += v; variant = Some("support_gem_limit_Plus"); },
                Self::map_simulacrum_difficulty_Plus(Plus(v)) => { total += v; variant = Some("map_simulacrum_difficulty_Plus"); },
                Self::base_spell_critical_strike_multiplier_Plus(Plus(v)) => { total += v; variant = Some("base_spell_critical_strike_multiplier_Plus"); },
                Self::map_voodoo_king_difficulty_Plus(Plus(v)) => { total += v; variant = Some("map_voodoo_king_difficulty_Plus"); },
                Self::maps_with_bosses_additional_strongbox_Plus(Plus(v)) => { total += v; variant = Some("maps_with_bosses_additional_strongbox_Plus"); },
                Self::melee_range_Plus(Plus(v)) => { total += v; variant = Some("melee_range_Plus"); },
             /* snipping again */
impl Add for StatType {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Self::all_skill_gem_level_Plus(Plus(a)), Self::all_skill_gem_level_Plus(Plus(b))) => Some(Self::all_skill_gem_level_Plus(Plus(a + b))),
            (Self::map_pinnacle_boss_difficulty_Plus(Plus(a)), Self::map_pinnacle_boss_difficulty_Plus(Plus(b))) => Some(Self::map_pinnacle_boss_difficulty_Plus(Plus(a + b))),
            (Self::map_expedition_chest_marker_count_Plus(Plus(a)), Self::map_expedition_chest_marker_count_Plus(Plus(b))) => Some(Self::map_expedition_chest_marker_count_Plus(Plus(a + b))),
            (Self::maps_with_bosses_additional_shrine_Plus(Plus(a)), Self::maps_with_bosses_additional_shrine_Plus(Plus(b))) => Some(Self::maps_with_bosses_additional_shrine_Plus(Plus(a + b))),
            (Self::quarterstaff_critical_strike_multiplier_Plus(Plus(a)), Self::quarterstaff_critical_strike_multiplier_Plus(Plus(b))) => Some(Self::quarterstaff_critical_strike_multiplier_Plus(Plus(a + b))),
           ... and so on. I want to remove the Plus from the end of our enum names.. i.e quarterstaff_critical_strike_multiplier_Plus should just be quarterstaff_critical_strike_multiplier where can I do that?```
````
