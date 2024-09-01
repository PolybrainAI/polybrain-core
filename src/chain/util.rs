pub fn trim_assistant_prefix(s: &str) -> &str {
    let prefix = "Assistant:";

    if let Some(stripped) = s.strip_prefix(prefix) {
        stripped.trim_start()
    } else {
        s
    }
}
