//! Table mapping information for keys.
//!
//! The key trait is implemented for every key struct the the Toql derive gernerates..
//! It is used for internal purposes to build key predicates and for trait bounds.
//!
//! Library users may also use it for trait bounds.
//!
//! The trait cannot be used to get a key from an entity. This can be done with the [Keyed](crate::keyed::Keyed) trait.

use crate::{sql_arg::SqlArg, sql_expr::SqlExpr};

/// Trait to provide the entity type for a key. This is only used
/// for ergonomics of the api.
pub trait Key {
    type Entity;

    /// Return primary key columns for a given entity.
    fn columns() -> Vec<String>;

    /// Return foreign key columns that match the primary keys for a given entity.
    /// This is only needed to merge entities.
    /// The names are calculated and do not necessarily match
    /// the actual foreign keys on the other table.
    /// The translation rules are (for snake case column format):
    ///
    /// | Type          | Guessing rule             | Example      |
    /// | --------------|---------------------------|---------------|
    /// | Normal fields |  tablename + normal field | `id` -> `user_id`, `access_code` -> `user_access_code` |
    /// | Joins         |  *No change* | `language_id` -> `language_id` |
    ///
    /// If the automatic generated names are not correct, the user is required to correct them by attributing
    /// the relevant field with
    ///  
    /// `#[toql( merge( columns( self = "id", other = "user_code")))]`
    ///
    fn default_inverse_columns() -> Vec<String>;

    /// Return key values as params. Useful to loop across a composite key.
    fn params(&self) -> Vec<SqlArg>;

    fn unaliased_predicate_expr(&self) -> SqlExpr {
        let columns = Self::columns();
        let mut params = self.params().into_iter();
        let mut expr = SqlExpr::new();

        for c in columns {
            if !expr.is_empty() {
                expr.push_literal(" AND ".to_string());
            }
            expr.push_literal(c);
            expr.push_literal(" = ".to_string());
            expr.push_arg(params.next().unwrap_or(SqlArg::Null));
        }
        expr
    }
}
