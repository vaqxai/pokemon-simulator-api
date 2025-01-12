use anyhow::Result;

use super::{DbHandle, DbRepr};

/// Denotes that a type can be inserted into the database
pub trait DbPut: DbRepr {
    /// Arguments for database insertion query (properties, so)
    /// e.g. "{name: 'John', age: 30}"
    fn put_args(&self) -> String;

    /// Inserts a new node into the database, holding the contents 'self'
    /// Does not duplicate nodes
    /// WARNING: DOES NOT HANDLE RELATIONSHIPS
    fn put_self_only(&self) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        debug!("PutSelfOnly: {}", self.put_args());
        let query = format!("MERGE (n:{} {})", Self::DB_NODE_KIND, self.put_args());
        debug!("PutSelfQuery: {query}");
        async move {
            let db = DbHandle::connect().await?;
            let mut q_res = db.inner.execute(query.into()).await?;
            let _none = q_res.next().await?;
            Ok(())
        }
    }
}
