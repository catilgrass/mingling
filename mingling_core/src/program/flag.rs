use crate::{Program, ProgramCollect};

/// A wrapper for a collection of static string slices representing command-line flags or arguments.
///
/// `Flag` is used to store one or more static string slices (e.g., `["-h", "--help"]`) that
/// represent command-line flags or arguments. It provides conversions from various input types
/// (like a single `&'static str`, a slice, or an array) and dereferences to a slice of strings
/// for easy iteration and access.
///
/// # Examples
///
/// ```
/// use mingling_core::Flag;
///
/// // Create a Flag from a single string slice
/// let flag1 = Flag::from("-h");
/// assert_eq!(flag1.as_ref(), &["-h"]);
///
/// // Create a Flag from a slice of string slices
/// let flag2 = Flag::from(&["-h", "--help"][..]);
/// assert_eq!(flag2.as_ref(), &["-h", "--help"]);
///
/// // Create a Flag from an array
/// let flag3 = Flag::from(["-v", "--verbose"]);
/// assert_eq!(flag3.as_ref(), &["-v", "--verbose"]);
///
/// // Create a Flag from a reference to an array
/// let arr = &["-f", "--file"];
/// let flag4 = Flag::from(arr);
/// assert_eq!(flag4.as_ref(), &["-f", "--file"]);
///
/// // Create an empty Flag from unit type
/// let flag5 = Flag::from(());
/// assert_eq!(flag5.as_ref(), &[] as &[&str]);
///
/// // Dereference to slice for iteration
/// let flag = Flag::from(["-a", "-b"]);
/// for arg in flag.iter() {
///     println!("Flag: {}", arg);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Flag {
    vec: Vec<&'static str>,
}

impl From<&Flag> for Flag {
    fn from(value: &Flag) -> Self {
        value.clone()
    }
}

impl From<()> for Flag {
    fn from(_: ()) -> Self {
        Flag { vec: vec![] }
    }
}

impl From<&'static str> for Flag {
    fn from(s: &'static str) -> Self {
        Flag { vec: vec![s] }
    }
}

impl From<&'static [&'static str]> for Flag {
    fn from(slice: &'static [&'static str]) -> Self {
        Flag {
            vec: slice.to_vec(),
        }
    }
}

impl<const N: usize> From<[&'static str; N]> for Flag {
    fn from(slice: [&'static str; N]) -> Self {
        Flag {
            vec: slice.to_vec(),
        }
    }
}

impl<const N: usize> From<&'static [&'static str; N]> for Flag {
    fn from(slice: &'static [&'static str; N]) -> Self {
        Flag {
            vec: slice.to_vec(),
        }
    }
}

impl AsRef<[&'static str]> for Flag {
    fn as_ref(&self) -> &[&'static str] {
        &self.vec
    }
}

impl std::ops::Deref for Flag {
    type Target = [&'static str];

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! special_flag {
    ($args:expr, $flag:expr) => {{
        let flag = $flag;
        let found = $args.iter().any(|arg| arg == flag);
        $args.retain(|arg| arg != flag);
        found
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! special_argument {
    ($args:expr, $flag:expr) => {{
        let flag = $flag;
        let mut value: Option<String> = None;
        let mut i = 0;
        while i < $args.len() {
            if &$args[i] == flag {
                if i + 1 < $args.len() {
                    value = Some($args[i + 1].clone());
                    $args.remove(i + 1);
                    $args.remove(i);
                } else {
                    value = None;
                    $args.remove(i);
                }
                break;
            }
            i += 1;
        }
        value
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! special_arguments {
    ($args:expr, $flag:expr) => {{
        let flag = $flag;
        let mut values: Vec<String> = Vec::new();
        let mut i = 0;
        while i < $args.len() {
            if &$args[i] == flag {
                $args.remove(i);
                while i < $args.len() && (flag.is_empty() || !$args[i].starts_with('-')) {
                    values.push($args[i].clone());
                    $args.remove(i);
                }
                break;
            }
            i += 1;
        }
        if flag.is_empty() {
            while !$args.is_empty() && !$args[0].starts_with('-') {
                values.push($args.remove(0));
            }
        }
        values
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_special_flag() {
        // Test flag found and removed
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--help".to_string(),
            "c".to_string(),
        ];
        let result = special_flag!(args, "--help");
        assert!(result);
        assert_eq!(args, vec!["a", "b", "c"]);

        // Test flag found at beginning
        let mut args = vec![
            "--help".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ];
        let result = special_flag!(args, "--help");
        assert!(result);
        assert_eq!(args, vec!["a", "b", "c"]);

        // Test flag found at end
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "--help".to_string(),
        ];
        let result = special_flag!(args, "--help");
        assert!(result);
        assert_eq!(args, vec!["a", "b", "c"]);

        // Test flag not found
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--other".to_string(),
            "c".to_string(),
        ];
        let result = special_flag!(args, "--help");
        assert!(!result);
        assert_eq!(args, vec!["a", "b", "--other", "c"]);

        // Test multiple same flags all removed
        let mut args = vec![
            "--help".to_string(),
            "a".to_string(),
            "--help".to_string(),
            "b".to_string(),
            "--help".to_string(),
        ];
        let result = special_flag!(args, "--help");
        assert!(result);
        assert_eq!(args, vec!["a", "b"]);

        // Test empty args
        let mut args: Vec<String> = Vec::new();
        let result = special_flag!(args, "--help");
        assert!(!result);
        assert_eq!(args, Vec::<String>::new());

        // Test flag with empty string
        let mut args = vec!["a".to_string(), "".to_string(), "b".to_string()];
        let result = special_flag!(args, "");
        assert!(result);
        assert_eq!(args, vec!["a", "b"]);

        // Test flag with dash in middle
        let mut args = vec!["a".to_string(), "test-flag".to_string(), "b".to_string()];
        let result = special_flag!(args, "test-flag");
        assert!(result);
        assert_eq!(args, vec!["a", "b"]);

        // Test flag that's a substring of another flag (should not match)
        let mut args = vec!["a".to_string(), "--helpful".to_string(), "b".to_string()];
        let result = special_flag!(args, "--help");
        assert!(!result);
        assert_eq!(args, vec!["a", "--helpful", "b"]);
    }

    #[test]
    fn test_special_argument() {
        // Test extracting value after flag
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--file".to_string(),
            "test.txt".to_string(),
            "c".to_string(),
        ];
        let result = special_argument!(args, "--file");
        assert_eq!(result, Some("test.txt".to_string()));
        assert_eq!(args, vec!["a", "b", "c"]);

        // Test extracting value when flag is at beginning
        let mut args = vec![
            "--file".to_string(),
            "test.txt".to_string(),
            "a".to_string(),
            "b".to_string(),
        ];
        let result = special_argument!(args, "--file");
        assert_eq!(result, Some("test.txt".to_string()));
        assert_eq!(args, vec!["a", "b"]);

        // Test extracting value when flag is at end
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--file".to_string(),
            "test.txt".to_string(),
        ];
        let result = special_argument!(args, "--file");
        assert_eq!(result, Some("test.txt".to_string()));
        assert_eq!(args, vec!["a", "b"]);

        // Test flag without value (at end)
        let mut args = vec!["a".to_string(), "b".to_string(), "--file".to_string()];
        let result = special_argument!(args, "--file");
        assert_eq!(result, None);
        assert_eq!(args, vec!["a", "b"]);

        // Test flag without value (not at end)
        let mut args = vec!["a".to_string(), "--file".to_string(), "b".to_string()];
        let result = special_argument!(args, "--file");
        assert_eq!(result, Some("b".to_string()));
        assert_eq!(args, vec!["a"]);

        // Test flag not found
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--other".to_string(),
            "value".to_string(),
        ];
        let result = special_argument!(args, "--file");
        assert_eq!(result, None);
        assert_eq!(args, vec!["a", "b", "--other", "value"]);

        // Test empty args
        let mut args: Vec<String> = Vec::new();
        let result = special_argument!(args, "--file");
        assert_eq!(result, None);
        assert_eq!(args, Vec::<String>::new());

        // Test multiple same flags (should only extract first)
        let mut args = vec![
            "--file".to_string(),
            "first.txt".to_string(),
            "--file".to_string(),
            "second.txt".to_string(),
        ];
        let result = special_argument!(args, "--file");
        assert_eq!(result, Some("first.txt".to_string()));
        assert_eq!(args, vec!["--file", "second.txt"]);

        // Test flag with empty string value
        let mut args = vec![
            "a".to_string(),
            "--file".to_string(),
            "".to_string(),
            "b".to_string(),
        ];
        let result = special_argument!(args, "--file");
        assert_eq!(result, Some("".to_string()));
        assert_eq!(args, vec!["a", "b"]);

        // Test flag with value starting with dash
        let mut args = vec![
            "a".to_string(),
            "--file".to_string(),
            "-value".to_string(),
            "b".to_string(),
        ];
        let result = special_argument!(args, "--file");
        assert_eq!(result, Some("-value".to_string()));
        assert_eq!(args, vec!["a", "b"]);
    }

    #[test]
    fn test_special_arguments() {
        // Test extracting multiple values after flag
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--list".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "--next".to_string(),
            "1".to_string(),
        ];
        let result = special_arguments!(args, "--list");
        assert_eq!(result, vec!["a", "b", "c", "d"]);
        assert_eq!(args, vec!["a", "b", "--next", "1"]);

        // Test extracting single value
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--next".to_string(),
            "1".to_string(),
        ];
        let result = special_arguments!(args, "--next");
        assert_eq!(result, vec!["1"]);
        assert_eq!(args, vec!["a", "b"]);

        // Test extracting from beginning
        let mut args = vec![
            "--list".to_string(),
            "a".to_string(),
            "b".to_string(),
            "--next".to_string(),
            "1".to_string(),
        ];
        let result = special_arguments!(args, "--list");
        assert_eq!(result, vec!["a", "b"]);
        assert_eq!(args, vec!["--next", "1"]);

        // Test extracting when no values after flag
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--list".to_string(),
            "--next".to_string(),
            "1".to_string(),
        ];
        let result = special_arguments!(args, "--list");
        assert_eq!(result, Vec::<String>::new());
        assert_eq!(args, vec!["a", "b", "--next", "1"]);

        // Test extracting when flag not found
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--list".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let result = special_arguments!(args, "--none");
        assert_eq!(result, Vec::<String>::new());
        assert_eq!(args, vec!["a", "b", "--list", "c", "d"]);

        // Test extracting empty args
        let mut args: Vec<String> = Vec::new();
        let result = special_arguments!(args, "--list");
        assert_eq!(result, Vec::<String>::new());
        assert_eq!(args, Vec::<String>::new());

        // Test extracting with only flag at end
        let mut args = vec!["--list".to_string()];
        let result = special_arguments!(args, "--list");
        assert_eq!(result, Vec::<String>::new());
        assert_eq!(args, Vec::<String>::new());

        // Test extracting multiple values until end of args
        let mut args = vec![
            "--list".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ];
        let result = special_arguments!(args, "--list");
        assert_eq!(result, vec!["a", "b", "c"]);
        assert_eq!(args, Vec::<String>::new());

        // Test extracting with mixed non-dash values
        let mut args = vec![
            "--list".to_string(),
            "value1".to_string(),
            "value2".to_string(),
            "-next".to_string(),
            "value3".to_string(),
        ];
        let result = special_arguments!(args, "--list");
        assert_eq!(result, vec!["value1", "value2"]);
        assert_eq!(args, vec!["-next", "value3"]);

        // Test extracting with single dash values
        let mut args = vec![
            "--list".to_string(),
            "-a".to_string(),
            "-b".to_string(),
            "--next".to_string(),
            "1".to_string(),
        ];
        let result = special_arguments!(args, "--list");
        assert_eq!(result, Vec::<String>::new());
        assert_eq!(args, vec!["-a", "-b", "--next", "1"]);

        // Test extracting with empty flag
        let mut args = vec![
            "a".to_string(),
            "b".to_string(),
            "--next".to_string(),
            "1".to_string(),
        ];
        let result = special_arguments!(args, "");
        assert_eq!(result, vec!["a", "b"]);
        assert_eq!(args, vec!["--next", "1"]);
    }
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Registers a global argument (with value) and its handler.
    pub fn global_argument<F, A>(&mut self, arguments: A, mut do_fn: F)
    where
        F: FnMut(&mut Program<C>, String),
        A: Into<Flag>,
    {
        let flag = arguments.into();
        for argument in flag.iter() {
            let value = special_argument!(self.args, argument);
            if let Some(value) = value {
                do_fn(self, value);
                return;
            }
        }
    }

    /// Registers a global flag (boolean) and its handler.
    pub fn global_flag<F, A>(&mut self, flag: A, mut do_fn: F)
    where
        F: FnMut(&mut Program<C>),
        A: Into<Flag>,
    {
        let flag = flag.into();
        for argument in flag.iter() {
            let enabled = special_flag!(self.args, argument);
            if enabled {
                do_fn(self);
                return;
            }
        }
    }

    /// Extracts a global argument (with value) from arguments
    pub fn pick_global_argument<F>(&mut self, flag: F) -> Option<String>
    where
        F: Into<Flag>,
    {
        let flag: Flag = flag.into();
        for argument in flag.iter() {
            let value = special_argument!(self.args, argument);
            if value.is_some() {
                return value;
            }
        }
        None
    }

    /// Extracts global flags from arguments
    pub fn pick_global_flag<F>(&mut self, flag: F) -> bool
    where
        F: Into<Flag>,
    {
        let flag: Flag = flag.into();
        for argument in flag.iter() {
            let enabled = special_flag!(self.args, argument);
            if enabled {
                return enabled;
            }
        }
        false
    }
}
