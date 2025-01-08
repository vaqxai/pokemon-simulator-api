use anyhow::Result;

use super::{DbHandle, DbRepr, sanitize};

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
                        sanitize(database_identifier)
                    )
                    .into(),
                )
                .await?;
            let _none = q_res.next().await?;
            Ok(())
        }
    }
}
