use anyhow::Result;
use neo4rs::{Graph, Node};
use std::fs;

/// Represents a handle to the database connection
pub struct DBHandle {
    /// The neo4j graph database connection
    pub inner: Graph,
}

impl DBHandle {
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

async fn get_db_node(kind: &str, database_identifier: &str) -> Result<Node> {
    let db = DBHandle::connect().await?;
    let mut q_out = db
        .inner
        .execute(
            format!(
                "MATCH (n:{}) WHERE n.id = {} RETURN n;",
                kind, database_identifier
            )
            .into(),
        )
        .await?;

    let row = q_out.next().await?.ok_or(anyhow::anyhow!("No rows"))?;
    // row should return one or more nodes

    row.get::<neo4rs::Node>("n").map_err(|e| e.into())
}

/// Represents a database-representable type that has a specific node kind in the Neo4j graph
pub trait DbRepr {
    /// The kind of node in the Neo4j graph that represents this type
    const DB_NODE_KIND: &'static str;
}

/// Denotes that a type can be retrieved from the database
pub trait DbGet: DbRepr {
    /// this function should make a new instance of the type from a neo4j node
    fn from_db_node(node: neo4rs::Node) -> Result<Self>
    where
        Self: Sized;

    /// the default impl of this function gets the first node of this type from the database
    /// matching the given identifier (the node needs to have an "id" field)
    fn get_first(
        database_identifier: &str,
    ) -> impl std::future::Future<Output = Result<Self>> + Send
    where
        Self: Sized,
    {
        async move {
            let node = get_db_node(Self::DB_NODE_KIND, database_identifier).await?;
            Self::from_db_node(node)
        }
    }
}

/// Denotes that a type can be inserted into the database
pub trait DbPut: DbRepr {
    /// Arguments for database insertion query (properties, so)
    /// e.g. "{name: 'John', age: 30}"
    fn put_args(&self) -> String;

    /// Inserts a new node into the database, holding the contents 'self'
    fn put_self(&self) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        let put_args = self.put_args();
        async move {
            let db = DBHandle::connect().await?;
            let mut q_res = db
                .inner
                .execute(format!("CREATE (n:{} {})", Self::DB_NODE_KIND, put_args).into())
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
            let db = DBHandle::connect().await?;
            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (n:{}) WHERE n.id = {} DELETE n;",
                        Self::DB_NODE_KIND,
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
            let db = DBHandle::connect().await?;

            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (n:{}) WHERE n.id = {} SET {}",
                        Self::DB_NODE_KIND,
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
