// Helper function to get everything before a character
pub fn str_before_char(input: &str, char: &str) -> String {
    let parts: Vec<&str> = input.split(char).collect();

    parts[0..parts.len() - 1].join(char)
}
