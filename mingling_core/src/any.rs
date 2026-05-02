#[cfg(feature = "general_renderer")]
use serde::Serialize;

use crate::Groupped;
use crate::error::ChainProcessError;

#[doc(hidden)]
pub mod group;

/// Any type output
///
/// Accepts any type that implements `Send + Groupped<G>`
/// After being passed into AnyOutput, it will be converted to `Box<dyn Any + Send + 'static>`
///
/// Note:
/// - If an enum value that does not belong to this type is incorrectly specified, it will be **unsafely** unwrapped by the scheduler
/// - Under the `general_renderer` feature, the passed value must ensure it implements `serde::Serialize`
/// - It is recommended to use the `pack!` macro from [mingling_macros](https://crates.io/crates/mingling_macros) to create types that can be converted to `AnyOutput`, which guarantees runtime safety
#[derive(Debug)]
pub struct AnyOutput<G> {
    pub(crate) inner: Box<dyn std::any::Any + Send + 'static>,
    pub type_id: std::any::TypeId,
    pub member_id: G,
}

impl<G> AnyOutput<G> {
    /// Create an AnyOutput from a `Send + Groupped<G> + Serialize` type
    #[cfg(feature = "general_renderer")]
    pub fn new<T>(value: T) -> Self
    where
        T: Send + Groupped<G> + Serialize + 'static,
    {
        Self {
            inner: Box::new(value),
            type_id: std::any::TypeId::of::<T>(),
            member_id: T::member_id(),
        }
    }

    /// Create an AnyOutput from a `Send + Groupped<G>` type
    #[cfg(not(feature = "general_renderer"))]
    pub fn new<T>(value: T) -> Self
    where
        T: Send + Groupped<G> + 'static,
    {
        Self {
            inner: Box::new(value),
            type_id: std::any::TypeId::of::<T>(),
            member_id: T::member_id(),
        }
    }

    /// Downcast the AnyOutput to a concrete type T
    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        if self.type_id == std::any::TypeId::of::<T>() {
            Ok(*self.inner.downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }

    /// Check if the inner value is of type T
    pub fn is<T: 'static>(&self) -> bool {
        self.type_id == std::any::TypeId::of::<T>()
    }

    /// Route the output to the next Chain
    pub fn route_chain(self) -> ChainProcess<G> {
        ChainProcess::Ok((self, Next::Chain))
    }

    /// Route the output to the Renderer, ending execution
    pub fn route_renderer(self) -> ChainProcess<G> {
        ChainProcess::Ok((self, Next::Renderer))
    }

    #[cfg(feature = "general_renderer")]
    /// Restore AnyOutput back to the original Serialize type
    pub fn restore<T: Serialize + 'static>(self) -> Option<T> {
        if self.type_id == std::any::TypeId::of::<T>() {
            match self.inner.downcast::<T>() {
                Ok(boxed) => Some(*boxed),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

impl<G> std::ops::Deref for AnyOutput<G> {
    type Target = dyn std::any::Any + Send + 'static;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<G> std::ops::DerefMut for AnyOutput<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

/// Chain exec result type
///
/// Stores `Ok` and `Err` types of execution results, used to notify the scheduler what to execute next
/// - Returns `Ok((`[`AnyOutput`](./struct.AnyOutput.html)`, `[`Next::Chain`](./enum.Next.html)`))` to continue execution with this type next
/// - Returns `Ok((`[`AnyOutput`](./struct.AnyOutput.html)`, `[`Next::Renderer`](./enum.Next.html)`))` to render this type next and output to the terminal
/// - Returns `Err(`[`ChainProcessError`](./error/enum.ChainProcessError.html)`]` to terminate the program directly
pub enum ChainProcess<G> {
    Ok((AnyOutput<G>, Next)),
    Err(ChainProcessError),
}

/// Indicates the next step after processing
///
/// - `Chain`: Continue execution to the next chain
/// - `Renderer`: Send output to renderer and end execution
#[derive(Debug, PartialEq, Eq)]
pub enum Next {
    Chain,
    Renderer,
}

impl std::fmt::Display for Next {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Next::Chain => write!(f, "Chain"),
            Next::Renderer => write!(f, "Renderer"),
        }
    }
}

impl<G> From<AnyOutput<G>> for ChainProcess<G> {
    fn from(value: AnyOutput<G>) -> Self {
        ChainProcess::Ok((value, Next::Chain))
    }
}
