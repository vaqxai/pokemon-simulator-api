use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use super::DbRepr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Promise<T: DbRepr + Promised> {
    ident: String,
    _phantom: PhantomData<T>,
}

impl<T: DbRepr + Promised> Promise<T> {
    pub fn ident(&self) -> String {
        self.ident.clone()
    }

    pub fn from_ident(ident: String) -> Self {
        Self {
            ident,
            _phantom: PhantomData,
        }
    }
}

pub trait Promised: DbRepr {
    async fn resolve(promise: Promise<Self>) -> Result<Self>
    where
        Self: Sized;

    fn promise_from_node(node: neo4rs::Node) -> Promise<Self>
    where
        Self: Sized;

    fn as_promise(&self) -> Promise<Self>
    where
        Self: Sized,
    {
        Promise::from_ident(self.get_identifier())
    }
}
