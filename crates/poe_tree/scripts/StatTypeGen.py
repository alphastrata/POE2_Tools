#!/usr/bin/env python3
"""
Generates a large Rust enum `StatType` with a unique variant for each discovered stat.

THIS CODE IS MACHINE-GENERATED. DO NOT EDIT MANUALLY.
"""

import json
import re

file_path = "data/POE2_Tree.json"
with open(file_path, "r") as f:
    poe_tree = json.load(f)

stat_categories = {
    "addition (+)": set(),
    "subtraction (-)": set(),
    "multiplication (*)": set(),
    "division (/)": set(),
    "percentage (+%)": set(),
    "other": set(),
}

percentage_pat = re.compile(r"\+%")
add_pat = re.compile(r"\+$")
mult_pat = re.compile(r"\*$")
div_pat = re.compile(r"/")
sub_pat = re.compile(r"-")

for skill in poe_tree.get("passive_skills", {}).values():
    if skill.get("is_just_icon"):
        continue
    stats_map = skill.get("stats", {})
    for stat_name in stats_map:
        if re.search(r"-%", stat_name):
            stat_categories["percentage (+%)"].add(stat_name)
        elif percentage_pat.search(stat_name):
            stat_categories["percentage (+%)"].add(stat_name)
        elif add_pat.search(stat_name):
            stat_categories["addition (+)"].add(stat_name)
        elif mult_pat.search(stat_name):
            stat_categories["multiplication (*)"].add(stat_name)
        elif div_pat.search(stat_name):
            stat_categories["division (/)"].add(stat_name)
        elif sub_pat.search(stat_name):
            stat_categories["subtraction (-)"].add(stat_name)
        else:
            stat_categories["other"].add(stat_name)

# We map each category to a default wrapper type:
cat_to_wrapper = {
    "addition (+)": "Plus",
    "subtraction (-)": "Minus",
    "multiplication (*)": "Multiply",
    "division (/)": "Divide",
    "percentage (+%)": "PlusPercentage",
    "other": "Other",  # We'll comment these out
}


def transform_name(raw: str) -> str:
    """
    E.g.:
    'evasion_rating_+%_while_surrounded' ->
    'evasion_rating_while_surrounded' for the variant name
    (Then we'll pick the wrapper e.g. PlusPercentage)
    """
    # We'll remove the +, - etc. from the name. Actually we prefer to transform them:
    # because e.g.  'attack_damage_+%' -> 'attack_damage__Percent'
    # but user wants to remove the suffix chars, then handle them in the wrapper type
    # We'll do a simpler approach: just strip out the plus/minus stuff that you
    # parse in the wrapper. For clarity:
    out = raw
    # remove the suffixes if any:
    out = re.sub(r"\+%$", "", out)  # remove trailing +%
    out = re.sub(r"\+$", "", out)  # remove trailing +
    out = re.sub(r"\-$", "", out)  # remove trailing -
    out = re.sub(r"\*$", "", out)  # remove trailing *
    # slash is tricky. Usually we do e.g. out = out.replace('/', '') if it's always in the middle
    # but let's do a simpler approach:
    out = out.replace("%", "")
    out = out.replace("+", "")
    out = out.replace("-", "")
    out = out.replace("*", "")
    out = out.replace("/", "")
    # remove double underscores or leftover underscores from the name
    out = out.replace(" ", "_")
    out = re.sub(r"_+", "_", out)
    return out.strip("_")


# Prepare code segments
variants = []
sum_match_arms = []
sum_wrapping_arms = []
add_arms = []
deserialize_arms = []


# We unify the "suffix detection" into the script:
def detect_wrapper(stat_name: str, default_wrapper: str) -> str:
    if "-%" in stat_name:
        return "MinusPercentage"
    elif "+%" in stat_name:
        return "PlusPercentage"
    elif stat_name.endswith("+"):
        return "Plus"
    elif stat_name.endswith("-"):
        return "Minus"
    elif stat_name.endswith("*"):
        return "Multiply"
    elif "/" in stat_name:
        return "Divide"
    else:
        return default_wrapper


# Keep track to avoid duplicates
already_done = set()

for cat, default_wrap in cat_to_wrapper.items():
    for raw_name in sorted(stat_categories[cat]):
        # e.g. "evasion_rating_+%_while_surrounded"
        # we want => variant name "evasion_rating_while_surrounded"
        var_id = transform_name(raw_name)
        if not var_id:
            # skip empty
            continue
        # if 'other', comment out
        if cat == "other":
            variants.append(f"    // {var_id}() // commented out.")
            continue
        # detect the actual wrapper for e.g. +% => PlusPercentage
        real_wrapper = detect_wrapper(raw_name, default_wrap)
        # e.g. "PlusPercentage"
        # Now we form the final variant: e.g.
        # evasion_rating_while_surrounded(PlusPercentage),
        # in the sum arms, etc.

        # Avoid duplicates if the same name appears more than once
        if var_id in already_done:
            continue
        already_done.add(var_id)

        # variants
        variants.append(f"    {var_id}({real_wrapper}),")

        # sum arms
        # e.g. Self::evasion_rating_while_surrounded(PlusPercentage(v)) => { ... },
        sum_match_arms.append(
            f'                Self::{var_id}({real_wrapper}(v)) => {{ total += v; variant = Some("{var_id}"); }},'
        )

        # sum_wrapping
        sum_wrapping_arms.append(
            f'            "{var_id}" => Self::{var_id}({real_wrapper}(total)),'
        )

        # add arms
        # e.g. (Self::...(...(a)), Self::...(...(b))) => Some(Self::...( ...(a + b))),
        add_arms.append(
            f"            (Self::{var_id}({real_wrapper}(a)), Self::{var_id}({real_wrapper}(b))) => Some(Self::{var_id}({real_wrapper}(a + b))),"
        )

        # For Deserialization arms, we assume we store "evasion_rating_while_surrounded" in JSON's "name".
        deserialize_arms.append(
            f'            "{var_id}" => Ok(StatType::{var_id}({real_wrapper}(helper.value))),'
        )

# Now let's create the final Rust code:
rust_template = r"""//! THIS CODE IS MACHINE-GENERATED. DO NOT EDIT MANUALLY.

use std::ops::{Add, AddAssign};
use serde::{Deserialize, de};
use serde::de::{Deserializer, Error as DeError};

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

/// If you absolutely can't categorize a stat, store it in `Other`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Other(pub f32);

macro_rules! impl_basic_maths_for {
    ($ty:ident) => {
        impl Add for $ty {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }
        impl AddAssign for $ty {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
    };
}

impl_basic_maths_for!(Plus);
impl_basic_maths_for!(Minus);
impl_basic_maths_for!(Multiply);
impl_basic_maths_for!(Divide);
impl_basic_maths_for!(PlusPercentage);
impl_basic_maths_for!(MinusPercentage);
impl_basic_maths_for!(Other);

#[allow(non_camel_Case)]
#[derive(Debug, Clone, PartialEq)]
pub enum StatType {
{VARIANTS}
}

impl StatType {
    pub fn sum(stats: &[Self]) -> Option<Self> {
        let mut total = 0.0;
        let mut variant = None;
        for stat in stats {
            match stat {
{SUM_ARMS}
            }
        }
        variant.map(|v| match v {
{SUM_WRAP}
            _ => unreachable!(),
        })
    }
}

impl Add for StatType {
    type Output = Option<Self>;
    fn add(self, other: Self) -> Option<Self> {
        match (self, other) {
{ADD_ARMS}
            _ => None,
        }
    }
}

impl AddAssign for StatType {
    fn add_assign(&mut self, other: Self) {
        if let Some(res) = self.clone() + other {
            *self = res;
        }
    }
}

/// Helper struct for deserializing from:
/// { \"name\": \"evasion_rating_while_surrounded\", \"value\": 30 }
#[derive(Debug, Deserialize)]
struct StatDeHelper {
    name: String,
    value: f32,
}

impl<'de> Deserialize<'de> for StatType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        let helper = StatDeHelper::deserialize(deserializer)?;
        match helper.name.as_str() {
{DESER_ARMS}
            _ => Err(DeError::custom(format!("Unknown stat: {}", helper.name))),
        }
    }
}
"""


final_rust = (
    rust_template.replace("{VARIANTS}", "\n".join(variants))
    .replace("{SUM_ARMS}", "\n".join(sum_match_arms))
    .replace("{SUM_WRAP}", "\n".join(sum_wrapping_arms))
    .replace("{ADD_ARMS}", "\n".join(add_arms))
    .replace("{DESER_ARMS}", "\n".join(deserialize_arms))
)

output_path = "crates/poe_tree/src/stats/stats_enum.rs"
with open(output_path, "w") as f:
    f.write(final_rust)

print(f"Done. Generated {output_path}")
