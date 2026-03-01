pub fn framework_log(message: impl AsRef<str>) {
    println!("[nestforge] {}", message.as_ref());
}

pub fn framework_log_event(event: &str, fields: &[(&str, String)]) {
    if fields.is_empty() {
        println!("[nestforge] event={event}");
        return;
    }

    let pairs = fields
        .iter()
        .map(|(key, value)| format!("{key}=\"{}\"", escape_log_value(value)))
        .collect::<Vec<_>>()
        .join(" ");

    println!("[nestforge] event={event} {pairs}");
}

fn escape_log_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
