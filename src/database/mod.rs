use anyhow::Result;
use neo4rs::Graph;
use std::fs;

pub mod delete;
pub mod get;
pub mod link;
pub mod promise;
pub mod put;
pub mod update;

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
