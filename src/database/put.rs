use anyhow::Result;

use super::{DbHandle, DbRepr};

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
