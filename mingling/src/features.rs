#[cfg(not(feature = "nightly"))]
pub const MINGLING_NIGHTLY: bool = false;

#[cfg(feature = "nightly")]
pub const MINGLING_NIGHTLY: bool = true;

#[cfg(not(feature = "debug"))]
pub const MINGLING_DEBUG: bool = false;

#[cfg(feature = "debug")]
pub const MINGLING_DEBUG: bool = true;

#[cfg(not(feature = "async"))]
pub const MINGLING_ASYNC: bool = false;

#[cfg(feature = "async")]
pub const MINGLING_ASYNC: bool = true;

#[cfg(not(feature = "clap"))]
pub const MINGLING_CLAP: bool = false;

#[cfg(feature = "clap")]
pub const MINGLING_CLAP: bool = true;

#[cfg(not(feature = "dispatch_tree"))]
pub const MINGLING_DISPATCH_TREE: bool = false;

#[cfg(feature = "dispatch_tree")]
pub const MINGLING_DISPATCH_TREE: bool = true;

#[cfg(not(feature = "general_renderer"))]
pub const MINGLING_GENERAL_RENDERER: bool = false;

#[cfg(feature = "general_renderer")]
pub const MINGLING_GENERAL_RENDERER: bool = true;

#[cfg(not(feature = "repl"))]
pub const MINGLING_REPL: bool = false;

#[cfg(feature = "repl")]
pub const MINGLING_REPL: bool = true;

#[cfg(not(feature = "comp"))]
pub const MINGLING_COMP: bool = false;

#[cfg(feature = "comp")]
pub const MINGLING_COMP: bool = true;

#[cfg(not(feature = "parser"))]
pub const MINGLING_PARSER: bool = false;

#[cfg(feature = "parser")]
pub const MINGLING_PARSER: bool = true;
