
pub fn trim_assistant_prefix(s: &str) -> &str {
    let prefix = "Assistant:";
    if s.starts_with(prefix) {
        &s[prefix.len()..].trim_start()
    } else {
        s
    }
}