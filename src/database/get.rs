use anyhow::Result;
use neo4rs::Node;
use std::pin::Pin;
use log::debug;

use super::{DbHandle, DbRepr, sanitize};

async fn get_db_node(id_name: &str, kind: &str, database_identifier: &str) -> Result<Node> {
    let db = DbHandle::connect().await?;

    let mut database_identifier = database_identifier.to_string();

    // if the identifier is not a number, put it in quotes
    if database_identifier.parse::<u64>().is_err() {
        database_identifier = format!("'{}'", sanitize(&database_identifier));
    }

    let query = format!(
                "MATCH (n:{}) WHERE n.{} = {} RETURN n;",
                kind, id_name, &database_identifier
                );

    debug!("Getting Node: {}", query);

    let mut q_out = db
        .inner
        .execute(
            query
            .into(),
        )
        .await?;

    let row = q_out.next().await?.ok_or(anyhow::anyhow!("No rows"))?;
    // row should return one or more nodes

    row.get::<Node>("n").map_err(|e| e.into())
}

/// Denotes that a type can be retrieved from the database
pub trait DbGet: DbRepr {
    /// The future type that resolves to the type
    type Future: Future<Output = Result<Self>> + Send
        = Pin<Box<dyn Future<Output = Result<Self>> + Send>>
    where
        Self: Sized;
    /// this function should make a new instance of the type from a neo4j node
    fn from_db_node(node: Node) -> Self::Future
    where
        Self: Sized;

    /// this function should get the database identifier of the node from the node
    /// e.g. "id" field
    fn identifier_from_node(node: Node) -> String
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

            let query = format!("MATCH (n:{}) RETURN n;", Self::DB_NODE_KIND);

            debug!("GetAll Query: {}", query);

            let mut q_out = db.inner.execute(query.into()).await?;

            debug!("GetAll Query Finished");

            let mut nodes = vec![];

            while let Some(row) = q_out.next().await? {
                let node = row.get::<Node>("n")?;
                nodes.push(Self::from_db_node(node).await?);
                debug!("Total nodes: {}", nodes.len());
            }

            debug!("GetAll Result Count: {}", nodes.len());

            Ok(nodes)
        }
    }

    /// Get a node of this type from the database by its identifier
    fn from_db_identifier(ident: &str) -> impl Future<Output = Result<Self>>
    where
        Self: Sized,
    {
        async move {
            let node = get_db_node(Self::DB_IDENTIFIER_FIELD, Self::DB_NODE_KIND, ident).await?;
            Self::from_db_node(node).await
        }
    }
}
