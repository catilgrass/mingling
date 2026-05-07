use std::fmt::Display;

use crate::{ChainProcess, Program, asset::node::Node};

/// Dispatches user input commands to specific [ChainProcess](./enum.ChainProcess.html)
///
/// Note: If you are using [mingling_macros](https://crates.io/crates/mingling_macros),
/// you can use the `dispatcher!("node.subnode", CommandType => Entry)` macro to declare a `Dispatcher`
pub trait Dispatcher<C> {
    /// Returns a command node for matching user input
    fn node(&self) -> Node;

    /// Returns a [ChainProcess](./enum.ChainProcess.html) based on user input arguments,
    /// to be sent to the specific invocation
    fn begin(&self, args: Vec<String>) -> ChainProcess<C>;

    /// Clones the current dispatcher for implementing the `Clone` trait
    fn clone_dispatcher(&self) -> Box<dyn Dispatcher<C>>;
}

impl<G> Clone for Box<dyn Dispatcher<G>>
where
    G: Display,
{
    fn clone(&self) -> Self {
        self.clone_dispatcher()
    }
}

impl<C: crate::program::ProgramCollect> Program<C> {
    /// Adds a dispatcher to the program.
    #[cfg(not(feature = "dispatch_tree"))]
    pub fn with_dispatcher<Disp>(&mut self, dispatcher: Disp)
    where
        Disp: Dispatcher<C> + Send + Sync + 'static,
    {
        self.dispatcher.push(Box::new(dispatcher));
    }

    /// Add some dispatchers to the program.
    #[cfg(not(feature = "dispatch_tree"))]
    pub fn with_dispatchers<D>(&mut self, dispatchers: D)
    where
        D: Into<Dispatchers<C>>,
    {
        let dispatchers = dispatchers.into();
        self.dispatcher.extend(dispatchers.dispatcher);
    }
}

/// A collection of dispatchers.
///
/// This struct holds a vector of boxed `Dispatcher` trait objects,
/// allowing multiple dispatchers to be grouped together and passed
/// to the program via `Program::with_dispatchers`.
/// A collection of dispatchers.
///
/// This struct holds a vector of boxed `Dispatcher` trait objects,
/// allowing multiple dispatchers to be grouped together and passed
/// to the program via `Program::with_dispatchers`.
pub struct Dispatchers<G> {
    dispatcher: Vec<Box<dyn Dispatcher<G> + Send + Sync + 'static>>,
}

impl<G> From<Vec<Box<dyn Dispatcher<G> + Send + Sync>>> for Dispatchers<G> {
    fn from(dispatcher: Vec<Box<dyn Dispatcher<G> + Send + Sync>>) -> Self {
        Self { dispatcher }
    }
}

impl<G> From<Box<dyn Dispatcher<G> + Send + Sync>> for Dispatchers<G> {
    fn from(dispatcher: Box<dyn Dispatcher<G> + Send + Sync>) -> Self {
        Self {
            dispatcher: vec![dispatcher],
        }
    }
}

impl<D, G> From<(D,)> for Dispatchers<G>
where
    D: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatcher: (D,)) -> Self {
        Self {
            dispatcher: vec![Box::new(dispatcher.0)],
        }
    }
}

impl<D1, D2, G> From<(D1, D2)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2)) -> Self {
        Self {
            dispatcher: vec![Box::new(dispatchers.0), Box::new(dispatchers.1)],
        }
    }
}

impl<D1, D2, D3, G> From<(D1, D2, D3)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
            ],
        }
    }
}

impl<D1, D2, D3, D4, G> From<(D1, D2, D3, D4)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    D4: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3, D4)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
                Box::new(dispatchers.3),
            ],
        }
    }
}

impl<D1, D2, D3, D4, D5, G> From<(D1, D2, D3, D4, D5)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    D4: Dispatcher<G> + Send + Sync + 'static,
    D5: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3, D4, D5)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
                Box::new(dispatchers.3),
                Box::new(dispatchers.4),
            ],
        }
    }
}

impl<D1, D2, D3, D4, D5, D6, G> From<(D1, D2, D3, D4, D5, D6)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    D4: Dispatcher<G> + Send + Sync + 'static,
    D5: Dispatcher<G> + Send + Sync + 'static,
    D6: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3, D4, D5, D6)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
                Box::new(dispatchers.3),
                Box::new(dispatchers.4),
                Box::new(dispatchers.5),
            ],
        }
    }
}

impl<D1, D2, D3, D4, D5, D6, D7, G> From<(D1, D2, D3, D4, D5, D6, D7)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    D4: Dispatcher<G> + Send + Sync + 'static,
    D5: Dispatcher<G> + Send + Sync + 'static,
    D6: Dispatcher<G> + Send + Sync + 'static,
    D7: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3, D4, D5, D6, D7)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
                Box::new(dispatchers.3),
                Box::new(dispatchers.4),
                Box::new(dispatchers.5),
                Box::new(dispatchers.6),
            ],
        }
    }
}

impl<G> std::ops::Deref for Dispatchers<G> {
    type Target = Vec<Box<dyn Dispatcher<G> + Send + Sync + 'static>>;

    fn deref(&self) -> &Self::Target {
        &self.dispatcher
    }
}

impl<G> From<Dispatchers<G>> for Vec<Box<dyn Dispatcher<G> + Send + Sync + 'static>> {
    fn from(val: Dispatchers<G>) -> Self {
        val.dispatcher
    }
}
