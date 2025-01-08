use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::database::sanitize;

use super::{DbRepr, get::DbGet};

/// Represents a promise to resolve a node in the database
/// This is useful for representing relationships between nodes
/// Because immediately recursively resolving them would create infinite cycles
/// Instead, this type holds the identifier of the node to be resolved
/// And can be made into a full type when needed
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct Promise<T: DbRepr + Promised> {
    ident: String,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T: DbRepr + Promised> Promise<T> {
    /// Get the identifier used to make this promise,
    /// this is a valid database identifier (such as an ID)
    pub fn ident(&self) -> &str {
        self.ident.as_str()
    }

    /// Get the identifier in a database friendly format
    /// By default wraps the .ident() in single quotes
    pub fn ident_db(&self) -> String {
        let ident = self.ident();
        // if the identifier is not a number, put it in quotes
        if ident.parse::<u64>().is_err() {
            format!("'{}'", sanitize(ident))
        } else {
            ident.to_string()
        }
    }

    /// Create a promise using a database identifier
    /// Warning: This does not check if the identifier is valid
    pub fn from_ident_unchecked(ident: String) -> Self {
        Self {
            ident,
            _phantom: PhantomData,
        }
    }
}

/// Denotes that a type can be promised, i.e. resolved from a promise
pub trait Promised: DbRepr + DbGet {
    /// Turn this promise into a full type (using a database request)
    fn resolve(promise: Promise<Self>) -> impl Future<Output = Result<Self>>
    where
        Self: Sized,
    {
        async move {
            let ident = promise.ident();
            Self::from_db_identifier(ident).await
        }
    }

    /// Turn a database node into a promise using its fields
    fn promise_from_node(node: neo4rs::Node) -> Promise<Self>
    where
        Self: Sized,
    {
        Promise::from_ident_unchecked(
            node.get::<String>(Self::DB_IDENTIFIER_FIELD)
                .unwrap()
                .to_string(),
        )
    }

    /// Get a promise from this type
    /// So a struct that can be used to make more of this type
    /// Storing promises is also useful because upon resolution they always contain
    /// the latest data from the database
    fn as_promise(&self) -> Promise<Self>
    where
        Self: Sized,
    {
        Promise::from_ident_unchecked(self.get_identifier())
    }
}
