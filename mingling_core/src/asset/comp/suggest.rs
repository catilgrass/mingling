use std::collections::BTreeSet;

use crate::ShellContext;

/// A completion suggestion that tells the shell how to perform completion.
/// This can be either a set of specific suggestion items or a request for file completion.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
pub enum Suggest {
    /// A set of specific suggestion items for the shell to display.
    Suggest(BTreeSet<SuggestItem>),

    /// A request for the shell to perform file‑path completion.
    #[default]
    FileCompletion,
}

impl Suggest {
    /// Creates a new Suggest variant containing a BTreeSet of suggestions.
    pub fn new() -> Self {
        Self::Suggest(BTreeSet::new())
    }

    /// Creates a FileCompletion variant.
    pub fn file_comp() -> Self {
        Self::FileCompletion
    }

    /// Filters out already typed flag arguments from suggestion results.
    pub fn strip_typed_argument(self, ctx: &ShellContext) -> Self {
        ctx.strip_typed_argument(self)
    }
}

impl<T> From<T> for Suggest
where
    T: IntoIterator,
    T::Item: Into<String>,
{
    fn from(items: T) -> Self {
        let suggests = items
            .into_iter()
            .map(|item| SuggestItem::new(item.into()))
            .collect();
        Suggest::Suggest(suggests)
    }
}

impl std::ops::Deref for Suggest {
    type Target = BTreeSet<SuggestItem>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Suggest(suggests) => suggests,
            Self::FileCompletion => panic!("Cannot deref FileCompletion variant"),
        }
    }
}

impl std::ops::DerefMut for Suggest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Suggest(suggests) => suggests,
            Self::FileCompletion => panic!("Cannot deref_mut FileCompletion variant"),
        }
    }
}

/// Represents a single suggestion item for shell completion.
///
/// This enum has two variants:
/// - `Simple(String)`: A suggestion without any description.
/// - `WithDescription(String, String)`: A suggestion with an associated description.
///
/// The first `String` always holds the suggestion text, and the second `String` (if present)
/// holds an optional description providing additional context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
pub enum SuggestItem {
    /// A simple suggestion with only the suggestion text.
    Simple(String),
    /// A suggestion with both text and a description.
    WithDescription(String, String),
}

impl Default for SuggestItem {
    fn default() -> Self {
        SuggestItem::Simple(String::new())
    }
}

impl PartialOrd for SuggestItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SuggestItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.suggest().cmp(other.suggest())
    }
}

impl SuggestItem {
    /// Creates a new simple suggestion without description.
    pub fn new(suggest: String) -> Self {
        Self::Simple(suggest)
    }

    /// Creates a new suggestion with a description.
    pub fn new_with_desc(suggest: String, description: String) -> Self {
        Self::WithDescription(suggest, description)
    }

    /// Adds a description to this suggestion, replacing any existing description.
    pub fn with_desc(self, description: String) -> Self {
        match self {
            Self::Simple(suggest) => Self::WithDescription(suggest, description),
            Self::WithDescription(suggest, _) => Self::WithDescription(suggest, description),
        }
    }

    /// Returns the suggestion text.
    pub fn suggest(&self) -> &String {
        match self {
            Self::Simple(suggest) => suggest,
            Self::WithDescription(suggest, _) => suggest,
        }
    }

    /// Updates the suggestion text.
    pub fn set_suggest(&mut self, new_suggest: String) {
        match self {
            Self::Simple(suggest) => *suggest = new_suggest,
            Self::WithDescription(suggest, _) => *suggest = new_suggest,
        }
    }

    /// Returns the description if present.
    pub fn description(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::WithDescription(_, description) => Some(description),
        }
    }

    /// Sets or replaces the description.
    pub fn set_description(&mut self, description: String) {
        match self {
            Self::Simple(suggest) => *self = Self::WithDescription(suggest.clone(), description),
            Self::WithDescription(_, desc) => *desc = description,
        }
    }

    /// Removes and returns the description if present.
    pub fn remove_desc(&mut self) -> Option<String> {
        match self {
            Self::Simple(_) => None,
            Self::WithDescription(suggest, description) => {
                let desc = std::mem::take(description);
                *self = Self::Simple(std::mem::take(suggest));
                Some(desc)
            }
        }
    }
}

impl From<String> for SuggestItem {
    fn from(suggest: String) -> Self {
        Self::new(suggest)
    }
}

impl From<(String, String)> for SuggestItem {
    fn from((suggest, description): (String, String)) -> Self {
        Self::new_with_desc(suggest, description)
    }
}
