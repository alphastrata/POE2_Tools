pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f32 / GB as f32)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f32 / MB as f32)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f32 / KB as f32)
    } else {
        format!("{bytes} bytes")
    }
}
