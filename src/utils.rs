// Helper function to get everything before a character
pub fn str_before_char(input: &str, char: &str) -> String {
    let parts: Vec<&str> = input.split(char).collect();

    parts[0..parts.len() - 1].join(char)
}

#[test]
fn test_str_before_char() {
    assert_eq!(str_before_char("foo/bar/baz", "/"), "foo/bar");
    assert_eq!(str_before_char("foo/bar/baz", "z"), "foo/bar/ba");
    assert_eq!(str_before_char("foo/bar/baz", "f"), "");
}
