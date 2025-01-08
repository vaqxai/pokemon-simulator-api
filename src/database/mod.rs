use anyhow::Result;
use neo4rs::Graph;
use std::fs;

/// The delete module contains traits to allow a type to be deleted from database
pub mod delete;

/// The get module contains traits to allow a type to be retrieved from the database
pub mod get;

/// The link module contains traits to allow a type to be linked to another type in the database
pub mod link;

/// The promise module contains traits to allow a type to be promised to be available from database
pub mod promise;

/// The put module contains traits to allow a type to be inserted into the database
pub mod put;

/// The update module contains traits to allow a type to be updated in the database
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

        let dbconfig = neo4rs::ConfigBuilder::new()
            .fetch_size(1000)
            .uri(url)
            .user(uname)
            .password(pass)
            .build()?;

        let graph = Graph::connect(dbconfig).await?;

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

/// Sanitize a string for use in a cypher query
pub fn sanitize(s: &str) -> String {
    // 1. remove all backslashes
    let mut s = s.replace("\\", "");
    // 2. escape all quotes and double quotes
    s = s.replace("'", "\\'");
    s = s.replace("\"", "\\\"");

    s
}
