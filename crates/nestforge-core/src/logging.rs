pub fn framework_log(message: impl AsRef<str>) {
    println!("[nestforge] {}", message.as_ref());
}
