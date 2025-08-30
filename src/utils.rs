pub fn strip_ansi(input: &str) -> String {
    let bytes = strip_ansi_escapes::strip(input);
    String::from_utf8(bytes).unwrap_or_else(|_| input.to_string())
}
