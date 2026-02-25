use std::sync::Arc;

use anyhow::Result;

use crate::Container;

/**
* Inject<T> = small helper wrapper for resolving dependencies from the container.
* 
* Why this exists:
* - keeps controller code cleaner
* - prepares us for future "real" DI ergonomics/macros
* - gives a Nest-like "injectable" vibe, even before decorators
*/
pub struct Inject<T>(Arc<T>);

impl<T> Inject<T>
where
    T: Send + Sync + 'static,
{
    /*
    Resolve T from the container and wrap it in Inject<T>.
    */
    pub fn from(container: &Container) -> Result<Self> {
        let inner = container.resolve::<T>()?;
        Ok(Self(inner))
    }

    /*
    Access the inner Arc<T> if needed
    */
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T> std::ops::Deref for Inject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}