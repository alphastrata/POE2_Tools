//$ deprecated\tree_utils.rs
use serde_json::Value;
use std::collections::HashSet;

fn print_keys_recursive(value: &Value, indent: usize, seen: &mut HashSet<String>) {
    if let Some(obj) = value.as_object() {
        for (k, v) in obj.iter() {
            let stripped_key = k.replace(|c: char| c.is_ascii_digit(), "");
            if seen.insert(stripped_key.clone()) {
                println!("{:indent$}{}", "", stripped_key, indent = indent);
                print_keys_recursive(v, indent + 2, seen);
            }
        }
    } else if let Some(arr) = value.as_array() {
        for elem in arr.iter() {
            print_keys_recursive(elem, indent + 2, seen);
        }
    }
}

// Usage
fn main() {
    let data = std::fs::read_to_string("./data/POE2_TREE.json").unwrap();
    let json: Value = serde_json::from_str(&data).unwrap();
    let mut seen = HashSet::new();
    print_keys_recursive(&json, 0, &mut seen);
}
