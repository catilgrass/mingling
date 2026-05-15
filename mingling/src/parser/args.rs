use std::mem::replace;

use mingling_core::{Flag, special_argument, special_arguments, special_flag};

/// User input arguments
#[derive(Debug, Default, Clone)]
pub struct Argument {
    vec: Vec<String>,
}

impl From<Vec<&str>> for Argument {
    fn from(vec: Vec<&str>) -> Self {
        Argument {
            vec: vec.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl From<&'static str> for Argument {
    fn from(s: &'static str) -> Self {
        Argument {
            vec: vec![s.to_string()],
        }
    }
}

impl From<&'static [&'static str]> for Argument {
    fn from(slice: &'static [&'static str]) -> Self {
        Argument {
            vec: slice.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

impl<const N: usize> From<[&'static str; N]> for Argument {
    fn from(slice: [&'static str; N]) -> Self {
        Argument {
            vec: slice.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

impl<const N: usize> From<&'static [&'static str; N]> for Argument {
    fn from(slice: &'static [&'static str; N]) -> Self {
        Argument {
            vec: slice.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

impl From<Vec<String>> for Argument {
    fn from(vec: Vec<String>) -> Self {
        Argument { vec }
    }
}

impl AsRef<[String]> for Argument {
    fn as_ref(&self) -> &[String] {
        &self.vec
    }
}

impl std::ops::Deref for Argument {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl std::ops::DerefMut for Argument {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl Argument {
    /// Picks a single argument with the given flag
    pub fn pick_argument<F>(&mut self, flag: F) -> Option<String>
    where
        F: Into<Flag>,
    {
        if self.is_empty() {
            return None;
        }

        let flag: Flag = flag.into();
        if !flag.is_empty() {
            // Has any flag
            for argument in flag.iter() {
                let value = special_argument!(self.vec, argument);
                if value.is_some() {
                    return value;
                }
            }
        } else {
            // No flag
            return Some(self.vec.remove(0));
        }
        None
    }

    /// Picks arguments with the given flag
    pub fn pick_arguments<F>(&mut self, flag: F) -> Vec<String>
    where
        F: Into<Flag>,
    {
        let mut str_result = Vec::new();

        if self.is_empty() {
            return str_result;
        }

        let flag: Flag = flag.into();
        if flag.is_empty() {
            let value = special_arguments!(self.vec, "");
            str_result.extend(value);
        } else {
            for argument in flag.iter() {
                let value = special_arguments!(self.vec, argument);
                str_result.extend(value);
            }
        }

        str_result
    }

    /// Picks a flag with the given flag
    pub fn pick_flag<F>(&mut self, flag: F) -> bool
    where
        F: Into<Flag>,
    {
        if self.is_empty() {
            return false;
        }

        let flag: Flag = flag.into();
        if !flag.is_empty() {
            // Has any flag
            for argument in flag.iter() {
                let enabled = special_flag!(self.vec, argument);
                if enabled {
                    return enabled;
                }
            }
        } else {
            let first = self.vec.remove(0);
            let first_lower = first.to_lowercase();
            let trimmed = first_lower.trim();
            let result = match trimmed {
                "y" | "yes" | "true" | "1" => return true,
                "n" | "no" | "false" | "0" => return false,
                _ => false,
            };
            return result;
        }
        false
    }

    /// Dump all remaining arguments
    pub fn dump_remains(&mut self) -> Vec<String> {
        let new = Vec::new();
        replace(&mut self.vec, new)
    }

    /// Removes all arguments that start with a dash ('-')
    ///
    /// This method filters out all command-line style flags from the arguments,
    /// returning a new `Argument` instance containing only non-flag arguments.
    pub fn strip_all_flags(mut self) -> Self {
        self.vec.retain(|f| !f.starts_with('-'));
        self
    }
}
