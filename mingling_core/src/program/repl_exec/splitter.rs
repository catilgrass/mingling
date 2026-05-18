/// Wraps `split_input` to work with owned `String` inputs.
pub(crate) fn split_input_string(input: String) -> Vec<String> {
    split_input(&input)
}

/// Splits a string input into arguments, respecting single quotes, double quotes,
/// and backslash escaping.
pub(crate) fn split_input(input: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // Take the next character literally (if any) and add it to current.
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                // If there's no next character, the backslash is just ignored/lost.
            }
            '"' | '\'' => {
                // Start of a quoted segment.
                let quote_char = ch;
                let mut escaped = false;
                loop {
                    match chars.next() {
                        None => break,
                        Some(c) => {
                            if escaped {
                                current.push(c);
                                escaped = false;
                            } else if c == '\\' {
                                escaped = true;
                            } else if c == quote_char {
                                break;
                            } else {
                                current.push(c);
                            }
                        }
                    }
                }
            }
            ' ' => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

#[cfg(test)]
mod splitter_tests {
    use crate::program::repl_exec::splitter::split_input;

    #[test]
    fn test_split_with_double_quotes() {
        let input = r#"a "b c" d"#;
        let result = split_input(input);
        assert_eq!(result, vec!["a", "b c", "d"]);
    }

    #[test]
    fn test_split_with_single_quotes() {
        let input = "a 'b c' d";
        let result = split_input(input);
        assert_eq!(result, vec!["a", "b c", "d"]);
    }

    #[test]
    fn test_empty_input() {
        assert!(split_input("").is_empty());
    }

    #[test]
    fn test_no_quotes() {
        let result = split_input("hello world");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_double_quotes_at_edges() {
        let result = split_input(r#""hello world" foo"#);
        assert_eq!(result, vec!["hello world", "foo"]);
    }

    #[test]
    fn test_single_quotes_at_edges() {
        let result = split_input("'hello world' foo");
        assert_eq!(result, vec!["hello world", "foo"]);
    }

    #[test]
    fn test_multiple_double_quoted_parts() {
        let result = split_input(r#"a "b c" d "e f g""#);
        assert_eq!(result, vec!["a", "b c", "d", "e f g"]);
    }

    #[test]
    fn test_multiple_single_quoted_parts() {
        let result = split_input("a 'b c' d 'e f g'");
        assert_eq!(result, vec!["a", "b c", "d", "e f g"]);
    }

    #[test]
    fn test_backslash_escaped_space() {
        let result = split_input("a b\\ c d");
        assert_eq!(result, vec!["a", "b c", "d"]);
    }

    #[test]
    fn test_backslash_escaped_double_quote() {
        let result = split_input(r#"a b\"c d"#);
        assert_eq!(result, vec!["a", r#"b"c"#, "d"]);
    }

    #[test]
    fn test_backslash_escaped_single_quote() {
        let result = split_input("a b\\'c d");
        assert_eq!(result, vec!["a", "b'c", "d"]);
    }
}
