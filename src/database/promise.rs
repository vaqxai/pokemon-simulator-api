use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use super::DbRepr;

/// Represents a promise to resolve a node in the database
/// This is useful for representing relationships between nodes
/// Because immediately recursively resolving them would create infinite cycles
/// Instead, this type holds the identifier of the node to be resolved
/// And can be made into a full type when needed
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Promise<T: DbRepr + Promised> {
    ident: String,
    _phantom: PhantomData<T>,
}

impl<T: DbRepr + Promised> Promise<T> {
    /// Get the identifier used to make this promise,
    /// this is a valid database identifier (such as an ID)
    pub fn ident(&self) -> String {
        self.ident.clone()
    }

    /// Create a promise using a database identifier
    /// Warning: This does not check if the identifier is valid
    pub fn from_ident(ident: String) -> Self {
        Self {
            ident,
            _phantom: PhantomData,
        }
    }
}

/// Denotes that a type can be promised, i.e. resolved from a promise
pub trait Promised: DbRepr {
    /// Turn this promise into a full type (using a database request)
    fn resolve(promise: Promise<Self>) -> impl Future<Output = Result<Self>>
    where
        Self: Sized;

    /// Turn a database node into a promise using its fields
    fn promise_from_node(node: neo4rs::Node) -> Promise<Self>
    where
        Self: Sized;

    /// Get a promise from this type
    /// So a struct that can be used to make more of this type
    /// Storing promises is also useful because upon resolution they always contain
    /// the latest data from the database
    fn as_promise(&self) -> Promise<Self>
    where
        Self: Sized,
    {
        Promise::from_ident(self.get_identifier())
    }
}
