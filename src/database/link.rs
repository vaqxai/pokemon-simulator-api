use anyhow::Result;
use neo4rs::Node;

use super::{
    AsDbString, DbHandle, DbRepr,
    get::DbGet,
    promise::{Promise, Promised},
    sanitize,
};

/// Denotes the ability to link this type to another using database relationships
pub trait DbLink<T>: DbRepr
where
    T: DbRepr + DbGet + Promised,
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
        other: &Promise<T>,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()>;

    /// Adds a new link (does nothing if the link already exists) from 'self' to 'other'
    fn link_to(
        &mut self,
        other: &Promise<T>,
        relationship_type: &Self::RelationshipType,
    ) -> impl Future<Output = Result<()>> {
        async move {
            let db = DbHandle::connect().await?;

            let query = format!(
                "MATCH (a:{}), (b:{}) WHERE a.{} = {} AND b.{} = {} MERGE (a)-[:{}]->(b);",
                Self::DB_NODE_KIND,
                T::DB_NODE_KIND,
                Self::DB_IDENTIFIER_FIELD,
                &self.get_identifier(),
                T::DB_IDENTIFIER_FIELD,
                &other.ident_db(),
                relationship_type.as_db_string()
            );

            debug!("Linking query: {}", query);

            let mut q_res = db.inner.execute(query.into()).await?;

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
        other: &Promise<T>,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()>;

    /// Removes a link from 'self' to 'other' with the given relationship name
    fn unlink_from(
        &mut self,
        other: &Promise<T>,
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
                        &self.get_identifier(),
                        T::DB_IDENTIFIER_FIELD,
                        &other.ident_db(),
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
        other: &Promise<T>,
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
                        &self.get_identifier(),
                        T::DB_IDENTIFIER_FIELD,
                        &other.ident_db(),
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
    ) -> impl Future<Output = Result<Vec<Promise<T>>>> {
        async move {
            let db = DbHandle::connect().await?;

            let mut q_res = db
                .inner
                .execute(
                    format!(
                        "MATCH (a:{} {{ {} : {} }})-[:{}]->(b:{}) RETURN b;",
                        Self::DB_NODE_KIND,
                        Self::DB_IDENTIFIER_FIELD,
                        &database_identifier,
                        relationship_type.as_db_string(),
                        T::DB_NODE_KIND
                    )
                    .into(),
                )
                .await?;

            let mut nodes = vec![];

            while let Some(row) = q_res.next().await? {
                let node = row.get::<Node>("b")?;
                nodes.push(T::promise_from_node(node));
            }

            Ok(nodes)
        }
    }

    /// Returns the representations of nodes this node is linked to via the
    /// given relationship name
    fn get_linked_to(
        &self,
        relationship_type: &Self::RelationshipType,
    ) -> impl Future<Output = Result<Vec<Promise<T>>>> {
        Self::get_linked_by_id(relationship_type, self.get_identifier())
    }
}
