use anyhow::Result;
use neo4rs::{Graph, Node};
use std::{fs, pin::Pin};

/// Represents a handle to the database connection
pub struct DbHandle {
    /// The neo4j graph database connection
    pub inner: Graph,
}

impl DbHandle {
    /// Connects to the database using the configuration in `config.toml`
    pub async fn connect() -> Result<Self> {
        let cfg = fs::read_to_string("config.toml")?.parse::<toml::Table>()?;
        let url = format!(
            "neo4j://{}:{}",
            cfg["database"]["host"]
                .as_str()
                .ok_or(anyhow::anyhow!("No host"))?,
            cfg["database"]["port"]
        );

        let uname = cfg["database"]["username"]
            .as_str()
            .ok_or(anyhow::anyhow!("No username"))?;
        let pass = cfg["database"]["password"]
            .as_str()
            .ok_or(anyhow::anyhow!("No password"))?;

        let graph = Graph::new(url, uname, pass).await?;

        Ok(Self { inner: graph })
    }
}

async fn get_db_node(id_name: &str, kind: &str, database_identifier: &str) -> Result<Node> {
    let db = DbHandle::connect().await?;

    let mut database_identifier = database_identifier.to_string();

    // if the identifier is not a number, put it in quotes
    if database_identifier.parse::<u64>().is_err() {
        database_identifier = format!("'{}'", database_identifier);
    }

    let mut q_out = db
        .inner
        .execute(
            format!(
                "MATCH (n:{}) WHERE n.{} = {} RETURN n;",
                kind, id_name, database_identifier
            )
            .into(),
        )
        .await?;

    let row = q_out.next().await?.ok_or(anyhow::anyhow!("No rows"))?;
    // row should return one or more nodes

    row.get::<neo4rs::Node>("n").map_err(|e| e.into())
}

/// Represents a string that can be used in a database query
pub trait AsDbString {
    /// Get the string representation of this type for use in a database query
    fn as_db_string(&self) -> &'static str;
}

/// Represents a database-representable type that has a specific node kind in the Neo4j graph
pub trait DbRepr {
    /// The kind of node in the Neo4j graph that represents this type
    const DB_NODE_KIND: &'static str;

    /// The name of the identifier field
    const DB_IDENTIFIER_FIELD: &'static str = "id";

    /// Get the identifier of the database node
    /// In a database friendly format (strings in single quotes)
    fn get_identifier(&self) -> String;
}

/// Denotes that a type can be retrieved from the database
pub trait DbGet: DbRepr {
    /// The future type that resolves to the type
    type Future: Future<Output = Result<Self>> + Send
        = Pin<Box<dyn std::future::Future<Output = Result<Self>> + Send>>
    where
        Self: Sized;
    /// this function should make a new instance of the type from a neo4j node
    fn from_db_node(node: neo4rs::Node) -> Self::Future
    where
        Self: Sized;

    /// the default impl of this function gets the first node of this type from the database
    /// matching the given identifier (the node needs to have an "id" field)
    fn get_first(database_identifier: &str) -> impl Future<Output = Result<Self>> + Send
    where
        Self: Sized,
    {
        async move {
            let node = get_db_node(
                Self::DB_IDENTIFIER_FIELD,
                Self::DB_NODE_KIND,
                database_identifier,
            )
            .await?;
            Self::from_db_node(node).await
        }
    }

    /// Get all nodes of this type from the database
    fn get_all() -> impl Future<Output = Result<Vec<Self>>>
    where
        Self: Sized,
    {
        async move {
            let db = DbHandle::connect().await?;
            let mut q_out = db
                .inner
                .execute(format!("MATCH (n:{}) RETURN n;", Self::DB_NODE_KIND).into())
                .await?;

            let mut nodes = vec![];

            while let Some(row) = q_out.next().await? {
                let node = row.get::<Node>("n")?;
                nodes.push(Self::from_db_node(node).await?);
            }

            Ok(nodes)
        }
    }
}

/// Denotes that a type can be inserted into the database
pub trait DbPut: DbRepr {
    /// Arguments for database insertion query (properties, so)
    /// e.g. "{name: 'John', age: 30}"
    fn put_args(&self) -> String;

    /// Inserts a new node into the database, holding the contents 'self'
    /// Does not duplicate nodes
    fn put_self(&self) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        let put_args = self.put_args();
        async move {
            let db = DbHandle::connect().await?;
            let mut q_res = db
                .inner
                .execute(format!("MERGE (n:{} {})", Self::DB_NODE_KIND, put_args).into())
                .await?;
            let _none = q_res.next().await?;
            Ok(())
        }
    }
}

/// Denotes an ability to delete a node from the database
pub trait DbDelete: DbRepr {
    /// Deletes the node from the database with the given identifier
    fn delete(database_identifier: &str) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        async move {
            let db = DbHandle::connect().await?;
            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (n:{}) WHERE n.{} = {} DELETE n;",
                        Self::DB_NODE_KIND,
                        Self::DB_IDENTIFIER_FIELD,
                        database_identifier
                    )
                    .into(),
                )
                .await?;
            let _none = q_res.next().await?;
            Ok(())
        }
    }
}

/// Denotes an ability to update a node in the database
pub trait DbUpdate: DbRepr {
    /// Give the string representation of the update query
    /// e.g. "n.name = 'John', n.age = 30", the node is always 'n'
    fn update_args(&self) -> String
    where
        Self: Sized;

    /// Update database node at given identifier with contents of 'self'
    fn update(
        &self,
        database_identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        let update_args = self.update_args();
        async move {
            // first get old database node
            let db = DbHandle::connect().await?;

            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (n:{}) WHERE n.{} = {} SET {}",
                        Self::DB_NODE_KIND,
                        Self::DB_IDENTIFIER_FIELD,
                        database_identifier,
                        update_args
                    )
                    .into(),
                )
                .await?;

            let _none = q_res.next().await?;

            Ok(())
        }
    }
}

/// Denotes the ability to link this type to another using database relationships
pub trait DbLink<T>: DbRepr
where
    T: DbRepr + DbGet,
{
    /// The type of relationship between the two nodes,
    /// ideally should be an enum of possible relationships
    type RelationshipType: AsDbString;
    /// A function that's called when making a link in the database
    /// Useful for e.g. setting type fields when linking
    /// to keep local fields up to date with the database
    /// this is mandatory to help remember to update local fields
    fn link_side_effect(
        &mut self,
        other: &T,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()>;

    /// Adds a new link (does nothing if the link already exists) from 'self' to 'other'
    fn link_to(
        &mut self,
        other: &T,
        relationship_type: &Self::RelationshipType,
    ) -> impl Future<Output = Result<()>> {
        async move {
            let db = DbHandle::connect().await?;

            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (a:{}), (b:{}) WHERE a.{} = {} AND b.{} = {} MERGE (a)-[:{}]->(b);",
                        Self::DB_NODE_KIND,
                        T::DB_NODE_KIND,
                        Self::DB_IDENTIFIER_FIELD,
                        self.get_identifier(),
                        T::DB_IDENTIFIER_FIELD,
                        other.get_identifier(),
                        relationship_type.as_db_string()
                    )
                    .into(),
                )
                .await?;

            let _none = q_res.next().await?;

            // TODO: If side effect fails, roll back the link
            self.link_side_effect(other, relationship_type)?;

            Ok(())
        }
    }

    /// A function called when a link gets dissolved,
    /// useful for e.g. setting type fields when unlinking
    /// this is mandatory to help remember to update local fields
    fn unlink_side_effect(
        &mut self,
        other: &T,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()>;

    /// Removes a link from 'self' to 'other' with the given relationship name
    fn unlink_from(
        &mut self,
        other: &T,
        relationship_type: &Self::RelationshipType,
    ) -> impl Future<Output = Result<()>> {
        async move {
            let db = DbHandle::connect().await?;

            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (a:{}), (b:{}) WHERE a.{} = {} AND b.{} = {} MATCH (a)-[r:{}]->(b) DELETE r;",
                        Self::DB_NODE_KIND,
                        T::DB_NODE_KIND,
                        Self::DB_IDENTIFIER_FIELD,
                        self.get_identifier(),
                        T::DB_IDENTIFIER_FIELD,
                        other.get_identifier(),
                        relationship_type.as_db_string()
                    )
                    .into(),
                )
                .await?;

            let _none = q_res.next().await?;

            // TODO: If side effect fails, roll back the unlink
            self.unlink_side_effect(other, relationship_type)?;

            Ok(())
        }
    }

    /// Checks whether a link exists from 'self' to 'other' with the given relationship name
    fn is_linked_by(
        &self,
        other: &T,
        relationship_name: &str,
    ) -> impl Future<Output = Result<bool>> {
        async move {
            let db = DbHandle::connect().await?;

            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (a:{}), (b:{}) WHERE a.{} = {} AND b.{} = {} RETURN exists((a)-[:{}]->(b));",
                        Self::DB_NODE_KIND,
                        T::DB_NODE_KIND,
                        Self::DB_IDENTIFIER_FIELD,
                        self.get_identifier(),
                        T::DB_IDENTIFIER_FIELD,
                        other.get_identifier(),
                        relationship_name
                    )
                    .into(),
                )
                .await?;

            // One row if successful
            Ok(q_res.next().await?.is_some())
        }
    }

    /// Returns the representations of nodes linked to this node via the given relationship name
    /// with the given identifier
    ///
    /// # Arguments
    ///
    /// * `relationship_name` - The name of the relationship to follow
    /// * `database_identifier` - The identifier of the node to get linked nodes from
    ///
    /// # Returns
    ///
    /// A future that resolves to a vector of the linked nodes
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// use crate::database::DbLink;
    /// use crate::pokemon::PokemonType;
    ///
    /// let water = PokemonType::get_first("Water").await.unwrap();
    /// let strong_against = water.get_linked_by_id("strong_against", water.get_identifier()).await.unwrap();
    ///
    /// ```
    fn get_linked_by_id(
        relationship_type: &Self::RelationshipType,
        database_identifier: String,
    ) -> impl Future<Output = Result<Vec<DbPromise<T>>>> {
        async move {
            let db = DbHandle::connect().await?;

            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (a:{} {{ {} : {} }})-[:{}]->(b:{}) RETURN b;",
                        Self::DB_NODE_KIND,
                        Self::DB_IDENTIFIER_FIELD,
                        database_identifier,
                        relationship_type.as_db_string(),
                        T::DB_NODE_KIND
                    )
                    .into(),
                )
                .await?;

            let mut nodes = vec![];

            while let Some(row) = q_res.next().await? {
                let node = row.get::<Node>("b")?;
                nodes.push(T::get_db_promise(node).await?);
            }

            Ok(nodes)
        }
    }

    /// Returns the representations of nodes this node is linked to via the
    /// given relationship name
    fn get_linked_to(
        &self,
        relationship_type: &Self::RelationshipType,
    ) -> impl Future<Output = Result<Vec<T>>> {
        Self::get_linked_by_id(relationship_type, self.get_identifier())
    }
}

pub struct DbPromise<T: DbRepr> {
    identifier: String,
    resolve_fn: Box<dyn FnOnce(&'static str, String) -> Result<T>>,
}

pub trait Promised: DbRepr {
    /// get function to resolve promises of this type
    fn get_resolve_fn() -> Box<dyn FnOnce(&'static str, String) -> Result<Self>>
    where
        Self: Sized;

    fn promise(database_identifier: &str) -> DbPromise<Self>
    where
        Self: Sized,
    {
        DbPromise {
            identifier: database_identifier.to_string(),
            resolve_fn: Self::get_resolve_fn(),
        }
    }
}

impl<T: DbRepr> DbPromise<T> {
    pub async fn resolve(self) -> Result<T> {
        (self.resolve_fn)(T::DB_NODE_KIND, self.identifier)
    }
}
