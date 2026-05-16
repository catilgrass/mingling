use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{ChainProcess, Program, ProgramCollect, this};

pub(crate) type GlobalResources = Arc<Mutex<HashMap<TypeId, Box<dyn Any + Sync + Send>>>>;

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Insert a resource of the given type, cloning the provided value into the store
    pub fn with_resource<Res: 'static + Send + Sync + ResourceMarker>(&mut self, res: Res) {
        if let Ok(mut guard) = self.resources.lock() {
            guard.insert(TypeId::of::<Res>(), Box::new(Arc::new(res)));
        }
    }

    /// Modify a resource by type, applying a closure to the resource if present
    pub fn modify_res<Res>(&self, f: impl FnOnce(&mut Res))
    where
        Res: 'static + Default + ResourceMarker + Send + Sync,
    {
        let mut guard = match self.resources.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return;
            }
        };
        if let Some(arc_res) = guard
            .get_mut(&TypeId::of::<Res>())
            .and_then(|a| a.downcast_mut::<Arc<Res>>())
        {
            let mut new_res = match Arc::try_unwrap(std::mem::take(arc_res)) {
                Ok(val) => val,
                Err(arc) => (*arc).res_clone(),
            };
            f(&mut new_res);
            *arc_res = Arc::new(new_res);
        }
    }

    /// Internal syntax for the `&mut MyResource` syntax of #[chain], do not use directly
    #[doc(hidden)]
    pub fn __modify_res_and_return_any<Res, Return>(
        &self,
        f: impl FnOnce(&mut Res) -> Return,
    ) -> impl Into<ChainProcess<C>>
    where
        Res: 'static + Default + ResourceMarker + Send + Sync,
        Return: Into<ChainProcess<C>>,
    {
        let mut guard = match self.resources.lock() {
            Ok(guard) => guard,
            Err(_) => {
                let mut default_res = Res::res_default();
                return f(&mut default_res);
            }
        };
        if let Some(arc_res) = guard
            .get_mut(&TypeId::of::<Res>())
            .and_then(|a| a.downcast_mut::<Arc<Res>>())
        {
            let mut new_res = match Arc::try_unwrap(std::mem::take(arc_res)) {
                Ok(val) => val,
                Err(arc) => (*arc).res_clone(),
            };
            let r = f(&mut new_res);
            *arc_res = Arc::new(new_res);
            r
        } else {
            let mut default_res = Res::res_default();
            f(&mut default_res)
        }
    }

    /// Get an resources by type, returning `Res` if present
    pub fn res<Res: 'static + Send + Sync>(&self) -> Option<GlobalResource<Res>> {
        let guard = self.resources.lock().ok()?;
        let boxed_any = guard.get(&TypeId::of::<Res>())?;
        let arc_res = boxed_any.as_ref().downcast_ref::<Arc<Res>>()?;
        Some(GlobalResource::from(Arc::clone(arc_res)))
    }

    /// Get a resource by type, returning `GlobalResource<Res>` if present
    pub fn res_or_route<Res: 'static + Send + Sync>(
        &self,
        route: ChainProcess<C>,
    ) -> Result<GlobalResource<Res>, ChainProcess<C>> {
        match self.res() {
            Some(r) => Ok(r),
            None => Err(route),
        }
    }

    /// Get a resource by type, returning `GlobalResource<Res>` or inserting a default
    pub fn res_or_default<Res: 'static + Send + Sync + ResourceMarker>(
        &self,
    ) -> GlobalResource<Res> {
        self.res()
            .unwrap_or_else(|| GlobalResource::from(Arc::new(Res::res_default())))
    }
}

/// Global assets for storing Program global state information
pub struct GlobalResource<ResType: 'static + Send + Sync> {
    res_arc: Arc<ResType>,
}

impl<ResType: 'static + Send + Sync> GlobalResource<ResType> {
    /// Create a new `GlobalAsset` from an `AssetType` value.
    pub fn new(res: ResType) -> Self {
        Self {
            res_arc: Arc::new(res),
        }
    }
}

impl<ResType: 'static + Send + Sync> From<Arc<ResType>> for GlobalResource<ResType> {
    fn from(arc: Arc<ResType>) -> Self {
        Self { res_arc: arc }
    }
}

impl<ResType: 'static + Send + Sync> std::ops::Deref for GlobalResource<ResType> {
    type Target = ResType;

    fn deref(&self) -> &Self::Target {
        &self.res_arc
    }
}

impl<ResType: 'static + Send + Sync> AsRef<ResType> for GlobalResource<ResType> {
    fn as_ref(&self) -> &ResType {
        &self.res_arc
    }
}

/// Resource marker trait, types that implement the Clone and Default traits can be considered as resources
pub trait ResourceMarker {
    fn res_clone(&self) -> Self;
    fn res_default() -> Self;
    fn modify<C>(f: impl FnOnce(&mut Self))
    where
        C: ProgramCollect<Enum = C> + 'static;
}

impl<T: Default + Clone + Send + Sync + 'static> ResourceMarker for T {
    fn res_clone(&self) -> Self {
        Clone::clone(self)
    }

    fn res_default() -> Self {
        Default::default()
    }

    fn modify<C>(f: impl FnOnce(&mut Self))
    where
        C: ProgramCollect<Enum = C> + 'static,
    {
        this::<C>().modify_res(f);
    }
}
