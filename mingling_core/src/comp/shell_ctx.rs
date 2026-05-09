use std::collections::HashSet;

use crate::{Flag, ShellFlag, Suggest};

/// Context passed from the shell to the completion system,
/// providing information about the current command line state
/// to guide how completions should be generated.
#[derive(Default, Debug)]
#[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
pub struct ShellContext {
    /// The full command line (-f / --command-line)
    pub command_line: String,

    /// Cursor position (-C / --cursor-position)
    pub cursor_position: usize,

    /// Current word (-w / --current-word)
    pub current_word: String,

    /// Previous word (-p / --previous-word)
    pub previous_word: String,

    /// Command name (-c / --command-name)
    pub command_name: String,

    /// Word index (-i / --word-index)
    pub word_index: usize,

    /// All words (-a / --all-words)
    pub all_words: Vec<String>,

    /// Flag to indicate completion context (-F / --shell-flag)
    pub shell_flag: ShellFlag,
}

impl TryFrom<Vec<String>> for ShellContext {
    type Error = String;

    fn try_from(args: Vec<String>) -> Result<Self, Self::Error> {
        use std::collections::HashMap;

        // Parse arguments into a map for easy lookup
        let mut arg_map = HashMap::new();
        let mut i = 0;
        while i < args.len() {
            if args[i].starts_with('-') {
                let key = args[i].clone();
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    arg_map.insert(key, args[i + 1].clone());
                    i += 2;
                } else {
                    arg_map.insert(key, String::new());
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        // Extract values with defaults
        let command_line = arg_map.get("-f").cloned().unwrap_or_default();
        let cursor_position = arg_map
            .get("-C")
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();
        let current_word = arg_map.get("-w").cloned().unwrap_or_default();
        let previous_word = arg_map.get("-p").cloned().unwrap_or_default();
        let command_name = arg_map.get("-c").cloned().unwrap_or_default();
        let word_index = arg_map
            .get("-i")
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();
        let shell_flag = arg_map
            .get("-F")
            .cloned()
            .map(ShellFlag::from)
            .unwrap_or(ShellFlag::Other("unknown".to_string()));

        let all_words = command_line
            .split_whitespace()
            .map(|s| s.replace('^', "-"))
            .collect();

        Ok(ShellContext {
            command_line: command_line.replace('^', "-"),
            cursor_position,
            current_word: current_word.replace('^', "-"),
            previous_word: previous_word.replace('^', "-"),
            command_name: command_name.replace('^', "-"),
            word_index,
            all_words,
            shell_flag,
        })
    }
}

impl ShellContext {
    /// Checks if a flag appears exactly once in the command line arguments.
    ///
    /// This method is useful for determining whether a flag should be processed
    /// when it should only be applied once, even if it appears multiple times
    /// in the command line. It returns `true` if the flag is present and
    /// appears exactly once among all words in the shell context.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::ShellContext;
    /// # use mingling_macros::suggest;
    /// # use mingling::comp_tools::ShellContextHelper;
    ///
    /// let ctx = ShellContext::default();
    /// let helper = ShellContextHelper::from(ctx);
    ///
    /// // Check if either "--insert" or "-i" appears exactly once
    /// if helper.filling_argument_first(["--insert", "-i"]) {
    ///     // Perform action that should only happen once, example:
    ///     // return suggest! {
    ///     //     "A", "B", "C"
    ///     // }
    /// }
    /// ```
    pub fn filling_argument_first(&self, flag: impl Into<Flag>) -> bool {
        let flag = flag.into();
        if self.filling_argument(&flag) {
            let mut flag_appears = 0;
            for w in self.all_words.iter() {
                for f in flag.iter() {
                    if *f == w {
                        flag_appears += 1;
                    }
                }
            }
            if flag_appears < 2 {
                return true;
            }
        }
        return false;
    }

    /// Checks if the previous word in the command line arguments matches any of the given flags.
    ///
    /// This method determines whether a flag is currently being processed
    /// by checking the word immediately before the cursor position. It returns
    /// `true` if the previous word matches any of the provided flag strings.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::ShellContext;
    /// # use mingling_macros::suggest;
    /// # use mingling::comp_tools::ShellContextHelper;
    ///
    /// let ctx = ShellContext::default();
    /// let helper = ShellContextHelper::from(ctx);
    ///
    /// // Check if the previous word is either "--file" or "-f"
    /// if helper.filling_argument(["--file", "-f"]) {
    ///     // The user is likely expecting a file argument next, example:
    ///     // return suggest! {
    ///     //     "src/main.rs", "Cargo.toml", "README.md"
    ///     // }
    /// }
    /// ```
    pub fn filling_argument(&self, flag: impl Into<Flag>) -> bool {
        for f in flag.into().iter() {
            if self.previous_word == **f {
                return true;
            }
        }
        return false;
    }

    /// Checks if the user is currently typing a flag argument.
    ///
    /// This method determines whether the current word being typed starts with
    /// a dash (`-`), indicating that the user is likely in the process of
    /// entering a command-line flag. On Windows, an empty current word is also
    /// considered as typing a flag to accommodate shell behavior differences.
    /// It returns `true` if the current word begins with a dash character.
    ///
    /// # Platform-specific behavior
    ///
    /// - **Windows**: Returns `true` if `current_word` is empty or starts with `-`
    /// - **Other platforms**: Returns `true` only if `current_word` starts with `-`
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::ShellContext;
    /// # use mingling_macros::suggest;
    /// # use mingling::comp_tools::ShellContextHelper;
    ///
    /// let ctx = ShellContext::default();
    /// let helper = ShellContextHelper::from(ctx);
    ///
    /// // Check if the user is typing a flag
    /// if helper.typing_argument() {
    ///     // The user is likely entering a flag, example:
    ///     // return suggest! {
    ///     //     "--help", "--version", "--verbose"
    ///     // }
    /// }
    /// ```
    pub fn typing_argument(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            self.current_word.is_empty()
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.current_word.starts_with("-")
        }
    }

    /// Filters out already typed flag arguments from suggestion results.
    ///
    /// This method removes any suggestions that match flag arguments already present
    /// in the command line. It is useful for preventing duplicate flag suggestions
    /// when the user has already typed certain flags. The method processes both
    /// regular suggestion sets and file completion suggestions differently.
    pub fn strip_typed_argument(&self, suggest: Suggest) -> Suggest {
        let typed = Self::get_typed_arguments(&self);
        match suggest {
            Suggest::Suggest(mut set) => {
                set.retain(|item| !typed.contains(item.suggest()));
                Suggest::Suggest(set)
            }
            Suggest::FileCompletion => Suggest::FileCompletion,
        }
    }

    /// Retrieves all flag arguments from the command line.
    ///
    /// This method collects all words in the shell context that start with a dash (`-`),
    /// which typically represent command-line flags or options. It returns a vector
    /// containing these flag strings, converted to owned `String` values.
    pub fn get_typed_arguments(&self) -> HashSet<String> {
        self.all_words
            .iter()
            .filter(|word| word.starts_with("-"))
            .map(|word| word.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_full_args() {
        let args = vec![
            "-f".to_string(),
            "git commit ^m 'test'".to_string(),
            "-C".to_string(),
            "12".to_string(),
            "-w".to_string(),
            "commit".to_string(),
            "-p".to_string(),
            "git".to_string(),
            "-c".to_string(),
            "git".to_string(),
            "-i".to_string(),
            "1".to_string(),
            "-F".to_string(),
            "bash".to_string(),
        ];

        let context = ShellContext::try_from(args).unwrap();
        assert_eq!(context.command_line, "git commit -m 'test'");
        assert_eq!(context.cursor_position, 12);
        assert_eq!(context.current_word, "commit");
        assert_eq!(context.previous_word, "git");
        assert_eq!(context.command_name, "git");
        assert_eq!(context.word_index, 1);
        assert_eq!(context.all_words, vec!["git", "commit", "-m", "'test'"]);
        assert!(matches!(context.shell_flag, ShellFlag::Bash));
    }

    #[test]
    fn test_try_from_partial_args() {
        let args = vec![
            "-f".to_string(),
            "ls ^la".to_string(),
            "-C".to_string(),
            "5".to_string(),
        ];

        let context = ShellContext::try_from(args).unwrap();
        assert_eq!(context.command_line, "ls -la");
        assert_eq!(context.cursor_position, 5);
        assert_eq!(context.current_word, "");
        assert_eq!(context.previous_word, "");
        assert_eq!(context.command_name, "");
        assert_eq!(context.word_index, 0);
        assert_eq!(context.all_words, vec!["ls", "-la"]);
        assert!(matches!(context.shell_flag, ShellFlag::Other(ref s) if s == "unknown"));
    }

    #[test]
    fn test_try_from_empty_args() {
        let args = vec![];
        let context = ShellContext::try_from(args).unwrap();
        assert_eq!(context.command_line, "");
        assert_eq!(context.cursor_position, 0);
        assert_eq!(context.current_word, "");
        assert_eq!(context.previous_word, "");
        assert_eq!(context.command_name, "");
        assert_eq!(context.word_index, 0);
        assert!(context.all_words.is_empty());
        assert!(matches!(context.shell_flag, ShellFlag::Other(ref s) if s == "unknown"));
    }

    #[test]
    fn test_try_from_flag_without_value() {
        let args = vec!["-F".to_string()];
        let context = ShellContext::try_from(args).unwrap();
        assert!(matches!(context.shell_flag, ShellFlag::Other(ref s) if s == ""));
    }

    #[test]
    fn test_all_words_splitting() {
        let args = vec!["-f".to_string(), "  cmd   arg1   arg2  ".to_string()];
        let context = ShellContext::try_from(args).unwrap();
        assert_eq!(context.all_words, vec!["cmd", "arg1", "arg2"]);
    }
}
