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

/// Represents a promise that may or may not be resolved
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum MaybePromise<T: Promised + DbRepr> {
    /// A promise
    Promise(Promise<T>),
    /// A concrete value
    Concrete(T),
}

impl<T: DbRepr + Promised> MaybePromise<T> {
    /// Returns the promise's ident or the concrete type's identifier
    pub fn ident(&self) -> &str {
        match self {
            MaybePromise::Promise(p) => p.ident(),
            MaybePromise::Concrete(c) => c.get_raw_identifier(),
        }
    }

    /// Resolves the promise to a concrete type or returns the concrete type
    pub async fn resolve(self) -> Result<T> {
        match self {
            MaybePromise::Promise(p) => p.resolve().await,
            MaybePromise::Concrete(c) => Ok(c),
        }
    }

    /// Returns the promise's database-compatible identifier or the concrete type's db identifier
    pub fn ident_db(&self) -> String {
        match self {
            MaybePromise::Promise(p) => p.ident_db(),
            MaybePromise::Concrete(c) => c.get_db_identifier(),
        }
    }

    /// Create a promise using a database identifier
    pub fn from_ident_unchecked(ident: String) -> Self {
        MaybePromise::Promise(Promise::from_ident_unchecked(ident))
    }

    /// Create a maybepromise holding a concrete type
    pub fn from_concrete(concrete: T) -> Self {
        MaybePromise::Concrete(concrete)
    }

    /// Create a maybepromise holding a promise
    pub fn from_promise(promise: Promise<T>) -> Self {
        MaybePromise::Promise(promise)
    }
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

    /// Resolves &self to concrete type by cloning
    pub fn resolve(&self) -> impl Future<Output = Result<T>> {
        T::resolve(self)
    }
}

/// Denotes that a type can be promised, i.e. resolved from a promise
pub trait Promised: DbRepr + DbGet {
    /// Turn this promise into a full type (using a database request)
    fn resolve(promise: &Promise<Self>) -> impl Future<Output = Result<Self>>
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
        Promise::from_ident_unchecked(self.get_db_identifier())
    }
}
