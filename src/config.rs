pub fn parse_color(col_str: &str) -> egui::Color32 {
    // Parse color from hex string (e.g., "#FF0000")
    if col_str.starts_with('#') && col_str.len() == 7 {
        let r = u8::from_str_radix(&col_str[1..3], 16).unwrap_or(255);
        let g = u8::from_str_radix(&col_str[3..5], 16).unwrap_or(255);
        let b = u8::from_str_radix(&col_str[5..7], 16).unwrap_or(255);
        egui::Color32::from_rgb(r, g, b)
    } else {
        egui::Color32::GRAY // Fallback if parsing fails
    }
}
