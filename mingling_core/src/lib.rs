//! Mingling Core
//!
//! # Intro
//! This crate is the core implementation of `mingling`, containing the complete logic for command dispatching, execution, and rendering.
//!
//! # Note
//! It is not recommended to use [mingling_core](https://crates.io/crates/mingling_core) directly, as this will lose the code generation functionality of [mingling_macros](https://crates.io/crates/mingling_macros).
//!
//! Recommended to import [mingling](https://crates.io/crates/mingling) to use its features.

mod any;
mod asset;
mod program;
mod renderer;

mod tester;

/// Provides a toolkit for `Mingling` testing capabilities.
pub mod test {
    pub use crate::tester::*;
}

#[cfg(feature = "general_renderer")]
pub use crate::renderer::general::GeneralRenderer;

pub use crate::any::group::*;
pub use crate::any::*;

pub use crate::asset::chain::*;
pub use crate::asset::dispatcher::*;
pub use crate::asset::enum_tag::*;
pub use crate::asset::global_resource::*;
pub use crate::asset::help::*;
pub use crate::asset::node::*;
pub use crate::asset::renderer::*;

/// All error types of `Mingling`
pub mod error {
    pub use crate::asset::chain::error::*;
    pub use crate::exec::error::*;
    #[cfg(feature = "general_renderer")]
    pub use crate::renderer::general::error::*;
}

pub use crate::program::*;

pub use crate::renderer::render_result::*;

/// `Mingling`'s Program initialization system
pub mod setup {
    pub use crate::program::setup::*;
}

#[doc(hidden)]
pub mod builds;

/// Provides build scripts for users
pub mod build {
    #[cfg(feature = "comp")]
    pub use crate::builds::comp::*;
}

/// Provided for framework developers
pub mod debug;

#[cfg(feature = "comp")]
#[doc(hidden)]
pub mod comp;

#[cfg(feature = "comp")]
pub use crate::comp::*;

pub use crate::setup::exit_code_control::current_exit_code;
pub use crate::setup::exit_code_control::update_exit_code;
