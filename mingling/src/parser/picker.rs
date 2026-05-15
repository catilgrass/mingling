use crate::parser::Argument;
use mingling_core::{EnumTag, Flag};

#[doc(hidden)]
pub mod builtin;

#[doc(hidden)]
pub mod bools;

/// A builder for extracting values from command-line arguments.
///
/// The `Picker` struct holds parsed arguments and provides a fluent interface
/// to extract values associated with specific flags.
pub struct Picker {
    /// The parsed command-line arguments.
    pub args: Argument,
}

impl Picker {
    /// Creates a new `Picker` from a value that can be converted into `Argument`.
    pub fn new(args: impl Into<Argument>) -> Picker {
        Picker { args: args.into() }
    }

    /// Extracts a value for the given flag and returns a `Pick1` builder (no route).
    ///
    /// The extracted type `TNext` must implement `Pickable` and `Default`.
    /// If the flag is not present, the default value for `TNext` is used.
    pub fn pick<TNext>(mut self, val: impl Into<Flag>) -> Pick1<TNext>
    where
        TNext: Pickable<Output = TNext> + Default,
    {
        let v = TNext::pick(&mut self.args, val.into()).unwrap_or_default();
        Pick1 {
            args: self.args,
            val_1: v,
        }
    }

    /// Extracts a value for the given flag, returning the provided default value if not present,
    /// and returns a `Pick1` builder (no route).
    ///
    /// The extracted type `TNext` must implement `Pickable`.
    /// If the flag is not present, the provided `or` value is used.
    pub fn pick_or<TNext>(mut self, val: impl Into<Flag>, or: impl Into<TNext>) -> Pick1<TNext>
    where
        TNext: Pickable<Output = TNext>,
    {
        let v = TNext::pick(&mut self.args, val.into()).unwrap_or(or.into());
        Pick1 {
            args: self.args,
            val_1: v,
        }
    }

    /// Extracts a value for the given flag, storing the provided route if the flag is not present,
    /// and returns a `PickWithRoute1` builder (with route).
    ///
    /// The extracted type `TNext` must implement `Pickable` and `Default`.
    /// If the flag is not present, the default value for `TNext` is used and the provided `route`
    /// is stored in the returned builder for later error handling.
    pub fn pick_or_route<TNext, R>(
        mut self,
        val: impl Into<Flag>,
        route: R,
    ) -> PickWithRoute1<TNext, R>
    where
        TNext: Pickable<Output = TNext> + Default,
    {
        let v = match TNext::pick(&mut self.args, val.into()) {
            Some(value) => value,
            None => {
                return PickWithRoute1 {
                    args: self.args,
                    val_1: TNext::default(),
                    route: Some(route),
                };
            }
        };
        PickWithRoute1 {
            args: self.args,
            val_1: v,
            route: None,
        }
    }

    /// Extracts a value for the given flag, returning `None` if the flag is not present,
    /// and returns an `Option<Pick1<TNext>>` builder (no route).
    ///
    /// The extracted type `TNext` must implement `Pickable`.
    /// If the flag is not present, `None` is returned.
    pub fn require<TNext>(mut self, val: impl Into<Flag>) -> Option<Pick1<TNext>>
    where
        TNext: Pickable<Output = TNext>,
    {
        let v = TNext::pick(&mut self.args, val.into());
        match v {
            Some(s) => Some(Pick1 {
                args: self.args,
                val_1: s,
            }),
            None => None,
        }
    }

    /// Applies an operation to the parsed arguments and returns the modified `Picker`.
    ///
    /// Takes a closure that receives the current `Argument` and returns a new `Argument`.
    /// The returned `Argument` replaces the original arguments in the builder.
    /// This method can be used to modify or transform the parsed arguments before extracting values.
    pub fn operate_args<F: FnOnce(Argument) -> Argument>(mut self, operation: F) -> Self {
        self.args = operation(self.args);
        self
    }
}

impl<T: Into<Argument>> From<T> for Picker {
    fn from(value: T) -> Self {
        Picker::new(value)
    }
}

/// Extracts values from command-line arguments
///
/// The `Pickable` trait defines how to extract the value of a specific flag from parsed arguments
pub trait Pickable {
    /// The output type produced by the extraction operation, must implement the `Default` trait
    type Output: Default;

    /// Extracts the value associated with the given flag from the provided arguments
    ///
    /// If the flag exists and the value can be successfully extracted, returns `Some(Output)`;
    /// otherwise returns `None`
    fn pick(args: &mut Argument, flag: Flag) -> Option<Self::Output>;
}

// Non-routed Pick structs (no R parameter, no route field)

/// Internal macro: generates the struct definition and common methods
/// (after, after_or_route, operate_args) for non-routed Pick structs.
macro_rules! define_pick_struct {
    ($n:ident $final:ident $final_val:ident $route_self:ident $($T:ident $val:ident),+ $(,)?) => {
        #[doc(hidden)]
        pub struct $n<$($T,)+>
        where
            $($T: Pickable,)+
        {
            #[allow(dead_code)]
            args: Argument,
            $(pub $val: $T,)+
        }

        impl<$($T,)+> $n<$($T,)+>
        where
            $($T: Pickable,)+
        {
            /// Applies a transformation to the last extracted value.
            ///
            /// Takes a closure that receives the last extracted value and returns a new value of the same type.
            /// The transformed value replaces the original value in the builder.
            /// This method can be used to modify or validate the extracted value before final unpacking.
            pub fn after<F>(mut self, mut edit: F) -> Self
            where
                F: FnMut($final) -> $final,
            {
                self.$final_val = edit(self.$final_val);
                self
            }

            /// Applies a transformation to the last extracted value, storing a route if the transformation fails.
            ///
            /// Takes a closure that receives a reference to the last extracted value and returns a `Result`.
            /// If the closure returns `Ok(new_value)`, the new value replaces the original value in the builder.
            /// If the closure returns `Err(route)`, the provided `route` is stored in the builder for later error handling.
            /// If a route was already stored from a previous `pick_or_route` call, the existing route is preserved.
            pub fn after_or_route<F, R>(mut self, mut edit: F) -> $route_self<$($T,)+ R>
            where
                F: FnMut(&$final) -> Result<$final, R>,
            {
                match edit(&self.$final_val) {
                    Ok(new_value) => {
                        self.$final_val = new_value;
                        $route_self {
                            args: self.args,
                            $($val: self.$val,)+
                            route: None,
                        }
                    }
                    Err(err_route) => {
                        $route_self {
                            args: self.args,
                            $($val: self.$val,)+
                            route: Some(err_route),
                        }
                    }
                }
            }

            /// Applies an operation to the parsed arguments and returns the modified builder.
            ///
            /// Takes a closure that receives the current `Argument` and returns a new `Argument`.
            /// The returned `Argument` replaces the original arguments in the builder.
            /// This method can be used to modify or transform the parsed arguments before extracting values.
            pub fn operate_args<F: FnOnce(Argument) -> Argument>(mut self, operation: F) -> Self {
                self.args = operation(self.args);
                self
            }
        }
    };
}

// Pick1 special case (single value)

define_pick_struct! { Pick1 T1 val_1 PickWithRoute1 T1 val_1 }

impl<T1> From<Pick1<T1>> for (T1,)
where
    T1: Pickable,
{
    fn from(pick: Pick1<T1>) -> Self {
        (pick.val_1,)
    }
}

impl<T1> Pick1<T1>
where
    T1: Pickable,
{
    /// Unpacks the builder into the extracted value.
    ///
    /// Always returns the value directly since there is no route.
    pub fn unpack(self) -> T1 {
        self.val_1
    }
}

// Pick2 .. Pick12

macro_rules! impl_pick_from_tuple {
    ($n:ident $($T:ident $val:ident),+) => {
        impl<$($T,)+> From<$n<$($T,)+>> for ($($T,)+)
        where
            $($T: Pickable,)+
        {
            fn from(pick: $n<$($T,)+>) -> Self {
                ($(pick.$val,)+)
            }
        }
    };
}

macro_rules! impl_pick_unpack_tuple {
    ($n:ident $($T:ident $val:ident),+) => {
        impl<$($T,)+> $n<$($T,)+>
        where
            $($T: Pickable,)+
        {
            /// Unpacks the builder into a tuple of extracted values.
            ///
            /// Always returns the tuple directly since there is no route.
            pub fn unpack(self) -> ($($T,)+) {
                ($(self.$val,)+)
            }
        }
    };
}

define_pick_struct! { Pick2 T2 val_2 PickWithRoute2 T1 val_1, T2 val_2 }
impl_pick_from_tuple! { Pick2 T1 val_1, T2 val_2 }
impl_pick_unpack_tuple! { Pick2 T1 val_1, T2 val_2 }

define_pick_struct! { Pick3 T3 val_3 PickWithRoute3 T1 val_1, T2 val_2, T3 val_3 }
impl_pick_from_tuple! { Pick3 T1 val_1, T2 val_2, T3 val_3 }
impl_pick_unpack_tuple! { Pick3 T1 val_1, T2 val_2, T3 val_3 }

define_pick_struct! { Pick4 T4 val_4 PickWithRoute4 T1 val_1, T2 val_2, T3 val_3, T4 val_4 }
impl_pick_from_tuple! { Pick4 T1 val_1, T2 val_2, T3 val_3, T4 val_4 }
impl_pick_unpack_tuple! { Pick4 T1 val_1, T2 val_2, T3 val_3, T4 val_4 }

define_pick_struct! { Pick5 T5 val_5 PickWithRoute5 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5 }
impl_pick_from_tuple! { Pick5 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5 }
impl_pick_unpack_tuple! { Pick5 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5 }

define_pick_struct! { Pick6 T6 val_6 PickWithRoute6 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6 }
impl_pick_from_tuple! { Pick6 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6 }
impl_pick_unpack_tuple! { Pick6 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6 }

define_pick_struct! { Pick7 T7 val_7 PickWithRoute7 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7 }
impl_pick_from_tuple! { Pick7 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7 }
impl_pick_unpack_tuple! { Pick7 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7 }

define_pick_struct! { Pick8 T8 val_8 PickWithRoute8 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8 }
impl_pick_from_tuple! { Pick8 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8 }
impl_pick_unpack_tuple! { Pick8 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8 }

define_pick_struct! { Pick9 T9 val_9 PickWithRoute9 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9 }
impl_pick_from_tuple! { Pick9 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9 }
impl_pick_unpack_tuple! { Pick9 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9 }

define_pick_struct! { Pick10 T10 val_10 PickWithRoute10 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10 }
impl_pick_from_tuple! { Pick10 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10 }
impl_pick_unpack_tuple! { Pick10 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10 }

define_pick_struct! { Pick11 T11 val_11 PickWithRoute11 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11 }
impl_pick_from_tuple! { Pick11 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11 }
impl_pick_unpack_tuple! { Pick11 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11 }

define_pick_struct! { Pick12 T12 val_12 PickWithRoute12 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11, T12 val_12 }
impl_pick_from_tuple! { Pick12 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11, T12 val_12 }
impl_pick_unpack_tuple! { Pick12 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11, T12 val_12 }

// Non-routed Pick chaining methods (pick, pick_or, pick_or_route, require)

#[doc(hidden)]
macro_rules! impl_pick_next {
    ($n:ident $next:ident $next_val:ident $route_next:ident $($T:ident $val:ident),+) => {
        impl<$($T,)+> $n<$($T,)+>
        where
            $($T: Pickable,)+
        {
            /// Extracts a value for the given flag and returns a `PickN` builder (no route).
            pub fn pick<TNext>(mut self, val: impl Into<mingling_core::Flag>) -> $next<$($T,)+ TNext>
            where
                TNext: Pickable<Output = TNext> + Default,
            {
                let v = TNext::pick(&mut self.args, val.into()).unwrap_or_default();
                $next {
                    args: self.args,
                    $($val: self.$val,)+
                    $next_val: v,
                }
            }

            /// Extracts a value for the given flag, returning the provided default value if not present,
            /// and returns a `PickN` builder (no route).
            pub fn pick_or<TNext>(mut self, val: impl Into<mingling_core::Flag>, or: impl Into<TNext>) -> $next<$($T,)+ TNext>
            where
                TNext: Pickable<Output = TNext>,
            {
                let v = TNext::pick(&mut self.args, val.into()).unwrap_or(or.into());
                $next {
                    args: self.args,
                    $($val: self.$val,)+
                    $next_val: v,
                }
            }

            /// Extracts a value for the given flag, storing the provided route if the flag is not present,
            /// and returns a `PickWithRouteN` builder (with route).
            pub fn pick_or_route<TNext, R>(
                mut self,
                val: impl Into<mingling_core::Flag>,
                route: R,
            ) -> $route_next<$($T,)+ TNext, R>
            where
                TNext: Pickable<Output = TNext> + Default,
            {
                let v = match TNext::pick(&mut self.args, val.into()) {
                    Some(value) => value,
                    None => {
                        return $route_next {
                            args: self.args,
                            $($val: self.$val,)+
                            $next_val: TNext::default(),
                            route: Some(route),
                        };
                    }
                };
                $route_next {
                    args: self.args,
                    $($val: self.$val,)+
                    $next_val: v,
                    route: None,
                }
            }

            /// Extracts a value for the given flag, returning `None` if the flag is not present,
            /// and returns an `Option<PickN<TNext>>` builder (no route).
            pub fn require<TNext>(mut self, val: impl Into<mingling_core::Flag>) -> Option<$next<$($T,)+ TNext>>
            where
                TNext: Pickable<Output = TNext>,
            {
                let v = TNext::pick(&mut self.args, val.into());
                match v {
                    Some(s) => Some($next {
                        args: self.args,
                        $($val: self.$val,)+
                        $next_val: s,
                    }),
                    None => None,
                }
            }
        }
    };
}

impl_pick_next! { Pick1 Pick2 val_2 PickWithRoute2 T1 val_1 }
impl_pick_next! { Pick2 Pick3 val_3 PickWithRoute3 T1 val_1, T2 val_2 }
impl_pick_next! { Pick3 Pick4 val_4 PickWithRoute4 T1 val_1, T2 val_2, T3 val_3 }
impl_pick_next! { Pick4 Pick5 val_5 PickWithRoute5 T1 val_1, T2 val_2, T3 val_3, T4 val_4 }
impl_pick_next! { Pick5 Pick6 val_6 PickWithRoute6 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5 }
impl_pick_next! { Pick6 Pick7 val_7 PickWithRoute7 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6 }
impl_pick_next! { Pick7 Pick8 val_8 PickWithRoute8 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7 }
impl_pick_next! { Pick8 Pick9 val_9 PickWithRoute9 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8 }
impl_pick_next! { Pick9 Pick10 val_10 PickWithRoute10 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9 }
impl_pick_next! { Pick10 Pick11 val_11 PickWithRoute11 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10 }
impl_pick_next! { Pick11 Pick12 val_12 PickWithRoute12 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11 }

// Routed PickWithRoute structs (with R parameter, route field)

/// Internal macro: generates the routed struct definition and common methods
/// (after, after_or_route, operate_args) for PickWithRoute structs.
macro_rules! define_pick_with_route_struct {
    ($n:ident $final:ident $final_val:ident $($T:ident $val:ident),+) => {
        #[doc(hidden)]
        pub struct $n<$($T,)+ R>
        where
            $($T: Pickable,)+
        {
            #[allow(dead_code)]
            args: Argument,
            $(pub $val: $T,)+
            route: Option<R>,
        }

        impl<$($T,)+ R> $n<$($T,)+ R>
        where
            $($T: Pickable,)+
        {
            /// Applies a transformation to the last extracted value.
            ///
            /// Takes a closure that receives the last extracted value and returns a new value of the same type.
            /// The transformed value replaces the original value in the builder.
            /// This method can be used to modify or validate the extracted value before final unpacking.
            pub fn after<F>(mut self, mut edit: F) -> Self
            where
                F: FnMut($final) -> $final,
            {
                self.$final_val = edit(self.$final_val);
                self
            }

            /// Applies a transformation to the last extracted value, storing a route if the transformation fails.
            ///
            /// Takes a closure that receives a reference to the last extracted value and returns a `Result`.
            /// If the closure returns `Ok(new_value)`, the new value replaces the original value in the builder.
            /// If the closure returns `Err(route)`, the provided `route` is stored in the builder for later error handling.
            /// If a route was already stored from a previous `pick_or_route` call, the existing route is preserved.
            pub fn after_or_route<F>(mut self, mut edit: F) -> Self
            where
                F: FnMut(&$final) -> Result<$final, R>,
            {
                let value = &self.$final_val;
                match edit(value) {
                    Ok(new_value) => {
                        self.$final_val = new_value;
                    }
                    Err(err_route) => {
                        let new_route = match self.route {
                            Some(existing_route) => Some(existing_route),
                            None => Some(err_route),
                        };
                        self.route = new_route;
                    }
                }
                self
            }

            /// Applies an operation to the parsed arguments and returns the modified builder.
            ///
            /// Takes a closure that receives the current `Argument` and returns a new `Argument`.
            /// The returned `Argument` replaces the original arguments in the builder.
            /// This method can be used to modify or transform the parsed arguments before extracting values.
            pub fn operate_args<F: FnOnce(Argument) -> Argument>(mut self, operation: F) -> Self {
                self.args = operation(self.args);
                self
            }
        }
    };
}

/// Internal macro: generates `From` impl for routed PickWithRouteN into a tuple.
macro_rules! impl_pick_with_route_from_tuple {
    ($n:ident $($T:ident $val:ident),+) => {
        impl<$($T,)+ R> From<$n<$($T,)+ R>> for ($($T,)+)
        where
            $($T: Pickable,)+
        {
            fn from(pick: $n<$($T,)+ R>) -> Self {
                ($(pick.$val,)+)
            }
        }
    };
}

/// Internal macro: generates `unpack` and `unpack_directly` for routed PickWithRouteN (N >= 2).
macro_rules! impl_pick_with_route_unpack_tuple {
    ($n:ident $($T:ident $val:ident),+) => {
        impl<$($T,)+ R> $n<$($T,)+ R>
        where
            $($T: Pickable,)+
        {
            /// Unpacks the builder into a tuple of extracted values.
            ///
            /// Returns `Ok((T1, T2, ...))` if no route was stored.
            /// Returns `Err(R)` if a route was stored via `pick_or_route` or `after_or_route`.
            pub fn unpack(self) -> Result<($($T,)+), R> {
                match self.route {
                    Some(route) => Err(route),
                    None => Ok(($(self.$val,)+)),
                }
            }

            /// Unpacks the builder into a tuple of extracted values.
            ///
            /// Returns the tuple of extracted values regardless of route state.
            pub fn unpack_directly(self) -> ($($T,)+) {
                ($(self.$val,)+)
            }
        }
    };
}

// PickWithRoute1 special case (single value)

define_pick_with_route_struct! { PickWithRoute1 T1 val_1 T1 val_1 }

impl<T1, R> From<PickWithRoute1<T1, R>> for (T1,)
where
    T1: Pickable,
{
    fn from(pick: PickWithRoute1<T1, R>) -> Self {
        (pick.val_1,)
    }
}

impl<T1, R> PickWithRoute1<T1, R>
where
    T1: Pickable,
{
    /// Unpacks the builder into the extracted value.
    ///
    /// Returns `Ok(T1)` if no route was stored.
    /// Returns `Err(R)` if a route was stored via `pick_or_route` or `after_or_route`.
    pub fn unpack(self) -> Result<T1, R> {
        match self.route {
            Some(route) => Err(route),
            None => Ok(self.val_1),
        }
    }

    /// Unpacks the builder into the extracted value.
    ///
    /// Returns the extracted value regardless of route state.
    pub fn unpack_directly(self) -> T1 {
        self.val_1
    }
}

// PickWithRoute2 .. PickWithRoute12

define_pick_with_route_struct! { PickWithRoute2 T2 val_2 T1 val_1, T2 val_2 }
impl_pick_with_route_from_tuple! { PickWithRoute2 T1 val_1, T2 val_2 }
impl_pick_with_route_unpack_tuple! { PickWithRoute2 T1 val_1, T2 val_2 }

define_pick_with_route_struct! { PickWithRoute3 T3 val_3 T1 val_1, T2 val_2, T3 val_3 }
impl_pick_with_route_from_tuple! { PickWithRoute3 T1 val_1, T2 val_2, T3 val_3 }
impl_pick_with_route_unpack_tuple! { PickWithRoute3 T1 val_1, T2 val_2, T3 val_3 }

define_pick_with_route_struct! { PickWithRoute4 T4 val_4 T1 val_1, T2 val_2, T3 val_3, T4 val_4 }
impl_pick_with_route_from_tuple! { PickWithRoute4 T1 val_1, T2 val_2, T3 val_3, T4 val_4 }
impl_pick_with_route_unpack_tuple! { PickWithRoute4 T1 val_1, T2 val_2, T3 val_3, T4 val_4 }

define_pick_with_route_struct! { PickWithRoute5 T5 val_5 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5 }
impl_pick_with_route_from_tuple! { PickWithRoute5 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5 }
impl_pick_with_route_unpack_tuple! { PickWithRoute5 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5 }

define_pick_with_route_struct! { PickWithRoute6 T6 val_6 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6 }
impl_pick_with_route_from_tuple! { PickWithRoute6 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6 }
impl_pick_with_route_unpack_tuple! { PickWithRoute6 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6 }

define_pick_with_route_struct! { PickWithRoute7 T7 val_7 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7 }
impl_pick_with_route_from_tuple! { PickWithRoute7 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7 }
impl_pick_with_route_unpack_tuple! { PickWithRoute7 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7 }

define_pick_with_route_struct! { PickWithRoute8 T8 val_8 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8 }
impl_pick_with_route_from_tuple! { PickWithRoute8 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8 }
impl_pick_with_route_unpack_tuple! { PickWithRoute8 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8 }

define_pick_with_route_struct! { PickWithRoute9 T9 val_9 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9 }
impl_pick_with_route_from_tuple! { PickWithRoute9 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9 }
impl_pick_with_route_unpack_tuple! { PickWithRoute9 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9 }

define_pick_with_route_struct! { PickWithRoute10 T10 val_10 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10 }
impl_pick_with_route_from_tuple! { PickWithRoute10 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10 }
impl_pick_with_route_unpack_tuple! { PickWithRoute10 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10 }

define_pick_with_route_struct! { PickWithRoute11 T11 val_11 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11 }
impl_pick_with_route_from_tuple! { PickWithRoute11 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11 }
impl_pick_with_route_unpack_tuple! { PickWithRoute11 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11 }

define_pick_with_route_struct! { PickWithRoute12 T12 val_12 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11, T12 val_12 }
impl_pick_with_route_from_tuple! { PickWithRoute12 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11, T12 val_12 }
impl_pick_with_route_unpack_tuple! { PickWithRoute12 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11, T12 val_12 }

// Routed PickWithRoute chaining methods (pick, pick_or, pick_or_route, require)

#[doc(hidden)]
macro_rules! impl_pick_with_route_next {
    ($n:ident $next:ident $next_val:ident $($T:ident $val:ident),+) => {
        impl<$($T,)+ R> $n<$($T,)+ R>
        where
            $($T: Pickable,)+
        {
            /// Extracts a value for the given flag and returns a `PickWithRouteN` builder.
            pub fn pick<TNext>(mut self, val: impl Into<mingling_core::Flag>) -> $next<$($T,)+ TNext, R>
            where
                TNext: Pickable<Output = TNext> + Default,
            {
                let v = TNext::pick(&mut self.args, val.into()).unwrap_or_default();
                $next {
                    args: self.args,
                    $($val: self.$val,)+
                    $next_val: v,
                    route: self.route,
                }
            }

            /// Extracts a value for the given flag, returning the provided default value if not present,
            /// and returns a `PickWithRouteN` builder.
            pub fn pick_or<TNext>(mut self, val: impl Into<mingling_core::Flag>, or: impl Into<TNext>) -> $next<$($T,)+ TNext, R>
            where
                TNext: Pickable<Output = TNext>,
            {
                let v = TNext::pick(&mut self.args, val.into()).unwrap_or(or.into());
                $next {
                    args: self.args,
                    $($val: self.$val,)+
                    $next_val: v,
                    route: self.route,
                }
            }

            /// Extracts a value for the given flag, storing the provided route if the flag is not present,
            /// and returns a `PickWithRouteN` builder.
            ///
            /// If a route was already stored from a previous `pick_or_route` or `after_or_route` call,
            /// the existing route is preserved and the new `route` parameter is ignored.
            pub fn pick_or_route<TNext>(mut self, val: impl Into<mingling_core::Flag>, route: R) -> $next<$($T,)+ TNext, R>
            where
                TNext: Pickable<Output = TNext> + Default,
            {
                let v = match TNext::pick(&mut self.args, val.into()) {
                    Some(value) => value,
                    None => {
                        let new_route = match self.route {
                            Some(existing_route) => Some(existing_route),
                            None => Some(route),
                        };
                        return $next {
                            args: self.args,
                            $($val: self.$val,)+
                            $next_val: TNext::default(),
                            route: new_route,
                        };
                    }
                };
                $next {
                    args: self.args,
                    $($val: self.$val,)+
                    $next_val: v,
                    route: self.route,
                }
            }

            /// Extracts a value for the given flag, returning `None` if the flag is not present,
            /// and returns an `Option<PickWithRouteN>` builder.
            pub fn require<TNext>(mut self, val: impl Into<mingling_core::Flag>) -> Option<$next<$($T,)+ TNext, R>>
            where
                TNext: Pickable<Output = TNext>,
            {
                let v = TNext::pick(&mut self.args, val.into());
                match v {
                    Some(s) => Some($next {
                        args: self.args,
                        $($val: self.$val,)+
                        $next_val: s,
                        route: self.route,
                    }),
                    None => None,
                }
            }
        }
    };
}

impl_pick_with_route_next! { PickWithRoute1 PickWithRoute2 val_2 T1 val_1 }
impl_pick_with_route_next! { PickWithRoute2 PickWithRoute3 val_3 T1 val_1, T2 val_2 }
impl_pick_with_route_next! { PickWithRoute3 PickWithRoute4 val_4 T1 val_1, T2 val_2, T3 val_3 }
impl_pick_with_route_next! { PickWithRoute4 PickWithRoute5 val_5 T1 val_1, T2 val_2, T3 val_3, T4 val_4 }
impl_pick_with_route_next! { PickWithRoute5 PickWithRoute6 val_6 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5 }
impl_pick_with_route_next! { PickWithRoute6 PickWithRoute7 val_7 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6 }
impl_pick_with_route_next! { PickWithRoute7 PickWithRoute8 val_8 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7 }
impl_pick_with_route_next! { PickWithRoute8 PickWithRoute9 val_9 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8 }
impl_pick_with_route_next! { PickWithRoute9 PickWithRoute10 val_10 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9 }
impl_pick_with_route_next! { PickWithRoute10 PickWithRoute11 val_11 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10 }
impl_pick_with_route_next! { PickWithRoute11 PickWithRoute12 val_12 T1 val_1, T2 val_2, T3 val_3, T4 val_4, T5 val_5, T6 val_6, T7 val_7, T8 val_8, T9 val_9, T10 val_10, T11 val_11 }

pub trait PickableEnum: EnumTag + Default {}

impl<T> Pickable for T
where
    T: PickableEnum,
{
    type Output = T;

    fn pick(args: &mut Argument, flag: Flag) -> Option<Self::Output> {
        let name = args.pick_argument(flag)?;
        T::build_enum(name)
    }
}

pub trait AsPicker
where
    Self: Into<Vec<String>>,
{
    /// Converts the value into a `Picker` by first converting it into a `Vec<String>`.
    fn to_picker(self) -> Picker
    where
        Self: Sized,
        Vec<String>: From<Self>,
    {
        let vec: Vec<String> = self.into();
        Picker { args: vec.into() }
    }

    /// Extracts a value for the given flag and returns a `Pick1` builder (no route).
    ///
    /// The extracted type `TNext` must implement `Pickable` and `Default`.
    /// If the flag is not present, the default value for `TNext` is used.
    fn pick<TNext>(self, val: impl Into<Flag>) -> Pick1<TNext>
    where
        Self: Sized,
        TNext: Pickable<Output = TNext> + Default,
    {
        let vec: Vec<String> = self.into();
        let picker: Picker = vec.into();
        picker.pick(val)
    }

    /// Extracts a value for the given flag, returning the provided default value if not present,
    /// and returns a `Pick1` builder (no route).
    ///
    /// The extracted type `TNext` must implement `Pickable`.
    /// If the flag is not present, the provided `or` value is used.
    fn pick_or<TNext>(self, val: impl Into<Flag>, or: impl Into<TNext>) -> Pick1<TNext>
    where
        TNext: Pickable<Output = TNext>,
    {
        let vec: Vec<String> = self.into();
        let picker: Picker = vec.into();
        picker.pick_or(val, or)
    }

    /// Extracts a value for the given flag, storing the provided route if the flag is not present,
    /// and returns a `PickWithRoute1` builder (with route).
    ///
    /// The extracted type `TNext` must implement `Pickable` and `Default`.
    /// If the flag is not present, the default value for `TNext` is used and the provided `route`
    /// is stored in the returned builder for later error handling.
    fn pick_or_route<TNext, R>(self, val: impl Into<Flag>, route: R) -> PickWithRoute1<TNext, R>
    where
        TNext: Pickable<Output = TNext> + Default,
    {
        let vec: Vec<String> = self.into();
        let picker: Picker = vec.into();
        picker.pick_or_route(val, route)
    }
}

// Implement AsPicker for any type that can be converted into a Vec<String>
impl<T> AsPicker for T
where
    T: Sized,
    Vec<String>: From<T>,
{
}
