use anyhow::Result;

use super::{DbHandle, DbRepr};

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
    ) -> impl Future<Output = Result<()>> + Send
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
