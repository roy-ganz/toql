//! Translate Toql query fields to database columns, SQL expressions, joins and merges.
pub mod field_options;
pub mod join_options;
pub mod join_type;
pub mod mapped;
pub mod merge_options;
pub mod predicate_options;

pub(crate) mod field;
pub(crate) mod join;
pub(crate) mod merge;
pub(crate) mod predicate;

use crate::{
    field_handler::{DefaultFieldHandler, FieldHandler},
    predicate_handler::{DefaultPredicateHandler, PredicateHandler},
    result::Result,
    role_expr::RoleExpr,
    sql_expr::SqlExpr,
    table_mapper::{
        field::Field, field_options::FieldOptions, join::Join, join_options::JoinOptions,
        mapped::Mapped, merge::Merge, merge_options::MergeOptions, predicate::Predicate,
        predicate_options::PredicateOptions,
    },
};
use heck::{CamelCase, MixedCase};
use join_type::JoinType;
use std::{collections::HashMap, fmt, sync::Arc};

/// Enum to hold different types that can be deserialization types
/// The mapper keeps an ordered list of all deserialization types of a Struct type.
/// The details for each can then be looked up seperately.
#[derive(Debug)]
pub enum DeserializeType {
    /// Field -or expression-, contains Toql query field name
    Field(String),
    /// Join, contains Toql query path name
    Join(String),
    /// Merge, contains Toql query path name
    Merge(String),
}

#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum TableMapperError {
    /// The requested canonical alias is not used. Contains the alias name.
    CanonicalAliasMissing(String),
    // Table column is missing. Contains table and column name.
    ColumnMissing(String, String),
}
impl fmt::Display for TableMapperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TableMapperError::CanonicalAliasMissing(ref s) => {
                write!(f, "canonical sql alias `{}` is missing", s)
            }
            TableMapperError::ColumnMissing(ref t, ref c) => {
                write!(f, "database table `{}` is missing column `{}`", t, c)
            }
        }
    }
}

/* trait MapperFilter {
    fn build(field: crate::query::query_token::QueryToken) -> String;
} */

/// Translates Toql fields into columns or SQL expressions.
///
/// It's needed by the  [SQL Builder](crate::sql_builder::SqlBuilder) to turn a [Query](crate::query::Query)
/// into a [SQL Builder Result](crate::sql_builder/build_result::BuildResult).
///
/// The Toql derive generates the TabeMapper instructions and puts them into the [Mapped](crate::table_mapper::mapped::Mapped) trait.
/// Every [ToqlApi](crate::toql_api::ToqlApi) function quickly checks, 
/// if [TableMapperRegistry](crate::table_mapper_registry::TableMapperRegistry)
/// contains the `TableMapper`. If the mapper is missing it will call [TreeMap](crate::tree::tree_map::TreeMap) to map an entity and all its dependencies.
/// TreeMap itself uses [from_mapped](TableMapper::from_mapped) to map an entity.
#[derive(Debug)]
pub struct TableMapper {
    /// Database table name
    pub table_name: String,

    /// Calculated alias from table_name
    pub canonical_table_alias: String,

    /// Default field handler for the mapped `struct`.
    pub(crate) field_handler: Arc<dyn FieldHandler + Send + Sync>,

    /// Default predicate handler for the mapped `struct`.
    pub(crate) predicate_handler: Arc<dyn PredicateHandler + Send + Sync>,

    /// Deserialization order for selects statements.
    pub(crate) deserialize_order: Vec<DeserializeType>,

    /// Maps a Toql query field name to field details.
    pub(crate) fields: HashMap<String, Field>,

    /// Maps a Toql query predicate name to predicate details.
    pub(crate) predicates: HashMap<String, Predicate>,

    /// Maps a Toql query path name to a joined mapper.
    pub(crate) joins: HashMap<String, Join>,

    /// Maps a Toql query path name to a merged mapper.
    pub(crate) merges: HashMap<String, Merge>,

    /// Load role expressions for the struct.
    pub(crate) load_role_expr: Option<RoleExpr>,

    /// Delete role expressions for the struct.
    pub(crate) delete_role_expr: Option<RoleExpr>,

    /// Maps a selection name to Toql query pathed fields or paths with wildcard
    /// Automatic created selection are
    /// $cnt - Fields for count query
    /// $mut - Fields for insert
    /// $all - All mapped fields
    pub(crate) selections: HashMap<String, Vec<String>>,
}

impl TableMapper {
    /// Create new mapper for _table_ or _table alias_.
    /// Example: `::new("Book")` or `new("Book b")`.
    /// If you use an alias you must map all
    /// SQL columns with the alias too.
    pub fn new<T>(sql_table_name: &str) -> Self
where {
        let f = DefaultFieldHandler {};
        Self::new_with_handler(sql_table_name, f)
    }

    /// Create a new mapper with a custom handler.
    /// Use this to provide custom filter functions for all fields.
    pub fn new_with_handler<H>(sql_table_name: &str, handler: H) -> Self
    where
        H: 'static + FieldHandler + Send + Sync, // TODO improve lifetime
    {
        TableMapper {
            field_handler: Arc::new(handler),
            predicate_handler: Arc::new(DefaultPredicateHandler::new()),
            table_name: sql_table_name.to_string(),
            canonical_table_alias: sql_table_name.to_mixed_case(),
            joins: HashMap::new(),
            merges: HashMap::new(),
            fields: HashMap::new(),
            predicates: HashMap::new(),
            deserialize_order: Vec::new(),
            selections: HashMap::new(),
            load_role_expr: None,
            delete_role_expr: None,
        }
    }
    /// Create a new mapper from a struct that implements the [Mapped] trait.
    pub fn from_mapped<M: Mapped>() -> Result<TableMapper> {
        Self::from_mapped_with_handler::<M, _>(DefaultFieldHandler::new())
    }
    /// Create a new mapper from a struct that implements the [Mapped] trait with a custom [FieldHandler].
    pub fn from_mapped_with_handler<M: Mapped, H>(handler: H) -> Result<TableMapper>
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let mut m = TableMapper::new_with_handler(&M::table_name(), handler);

        M::map(&mut m)?;
        Ok(m)
    }
    /// Returns joined mapper for a path name, if any.
    pub fn joined_mapper(&self, path_name: &str) -> Option<String> {
        self.join(path_name).map(|j| j.joined_mapper.to_owned())
    }
    /// Returns true, if path name refers to a partial join table.
    pub fn is_partial_join(&self, path_name: &str) -> bool {
        self.join(path_name)
            .filter(|j| j.options.partial_table)
            .is_some()
    }
    /// Returns joined mapper for a path name, if any.
    pub fn merged_mapper(&self, path_name: &str) -> Option<String> {
        self.merge(path_name).map(|m| m.merged_mapper.to_owned())
    }
    /// Returns join details for a path name, if any.
    pub(crate) fn join(&self, path_name: &str) -> Option<&Join> {
        self.joins.get(path_name)
    }
    /// Returns merge details for a path name, if any.
    pub(crate) fn merge(&self, path_name: &str) -> Option<&Merge> {
        self.merges.get(path_name)
    }
    /// Returns field details for a path name, if any.
    pub(crate) fn field(&self, field_name: &str) -> Option<&Field> {
        self.fields.get(field_name)
    }
    /// Returns all mappers that are partial joins from this `struct`.
    pub(crate) fn joined_partial_mappers(&self) -> Vec<(String, String)> {
        self.joins
            .iter()
            .filter_map(|(n, j)| {
                if j.options.partial_table {
                    Some((n.to_string(), j.joined_mapper.to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Maps a Toql field name to a field handler.
    /// The [FieldHandler] defines how to generate SQL.
    pub fn map_handler<'a, H>(
        &'a mut self,
        toql_field: &str,
        sql_expression: SqlExpr,
        handler: H,
    ) -> &'a mut Self
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        self.map_handler_with_options(toql_field, sql_expression, handler, FieldOptions::new())
    }
    /// Maps a Toql field with options to a field handler.
    /// The [FieldHandler] defines how to generate SQL.
    pub fn map_handler_with_options<'a, H>(
        &'a mut self,
        toql_field: &str,
        expression: SqlExpr,
        handler: H,
        options: FieldOptions,
    ) -> &'a mut Self
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let t = Field {
            options,
            handler: Arc::new(handler),
            expression,
        };
        self.deserialize_order
            .push(DeserializeType::Field(toql_field.to_string()));
        self.fields.insert(toql_field.to_string(), t);
        self
    }

    /// Map a column with default [FieldOptions]
    pub fn map_column<'a, T>(&'a mut self, toql_field: &str, column_name: T) -> &'a mut Self
    where
        T: Into<String>,
    {
        self.map_expr_with_options(
            toql_field,
            SqlExpr::aliased_column(column_name.into()),
            FieldOptions::new(),
        )
    }

    //// Map a column.
    /// Convenience function for generic [TableMapper::map_expr_with_options].
    pub fn map_column_with_options<'a, T>(
        &'a mut self,
        toql_field: &str,
        column_name: T,
        options: FieldOptions,
    ) -> &'a mut Self
    where
        T: Into<String>,
    {
        self.map_expr_with_options(
            toql_field,
            SqlExpr::aliased_column(column_name.into()),
            options,
        )
    }

    //// Map an expression
    pub fn map_expr_with_options<'a>(
        &'a mut self,
        toql_field: &str,
        expression: SqlExpr,
        options: FieldOptions,
    ) -> &'a mut Self {
        // Add count field to selection for quicker lookup
        if options.count_filter || options.count_select {
            let s = self
                .selections
                .entry("count".to_string())
                .or_insert_with(Vec::new);
            s.push(toql_field.to_string());
        }

        // Add mut field to selection for quicker lookup
        if !options.skip_mut {
            let s = self
                .selections
                .entry("mut".to_string())
                .or_insert_with(Vec::new);
            s.push(toql_field.to_string());
        }

        let t = Field {
            expression,
            options,
            handler: Arc::clone(&self.field_handler),
            //  sql_aux_param_names,
        };

        self.deserialize_order
            .push(DeserializeType::Field(toql_field.to_string()));
        self.fields.insert(toql_field.to_string(), t);

        self
    }
    /// Map a join with default [JoinOptions]
    pub fn map_join<'a>(
        &'a mut self,
        toql_path: &str,
        joined_mapper: &str,
        join_type: JoinType,
        table_expression: SqlExpr,
        on_expression: SqlExpr,
    ) -> &'a mut Self {
        self.map_join_with_options(
            toql_path,
            joined_mapper,
            join_type,
            table_expression,
            on_expression,
            JoinOptions::new(),
        )
    }
    /// Map a join with options.
    pub fn map_join_with_options<'a, S>(
        &'a mut self,
        toql_path: S,
        joined_mapper: &str,
        join_type: JoinType,
        table_expression: SqlExpr,
        on_expression: SqlExpr,
        options: JoinOptions,
    ) -> &'a mut Self
    where
        S: Into<String> + Clone,
    {
        self.joins.insert(
            toql_path.clone().into(),
            Join {
                joined_mapper: joined_mapper.to_camel_case(),
                join_type,
                table_expression,
                on_expression,
                options,
            },
        );

        self.deserialize_order
            .push(DeserializeType::Join(toql_path.into()));

        self
    }

    /// Map a merge with default [MergeOptions].
    pub fn map_merge<S>(
        &mut self,
        toql_path: S,
        merged_mapper: &str,
        merge_join: SqlExpr,
        merge_predicate: SqlExpr,
    ) -> &mut Self
    where
        S: Into<String> + Clone,
    {
        self.map_merge_with_options(
            toql_path,
            merged_mapper,
            merge_join,
            merge_predicate,
            MergeOptions::new(),
        )
    }
    /// Map a merge with default [MergeOptions].
    pub fn map_merge_with_options<S>(
        &mut self,
        toql_path: S,
        merged_mapper: &str,
        merge_join: SqlExpr,
        merge_predicate: SqlExpr,
        options: MergeOptions,
    ) -> &mut Self
    where
        S: Into<String> + Clone,
    {
        self.deserialize_order
            .push(DeserializeType::Merge(toql_path.clone().into()));
        self.merges.insert(
            toql_path.into(),
            merge::Merge {
                merged_mapper: merged_mapper.to_camel_case(),
                merge_join,
                merge_predicate,
                options,
            },
        );
        self
    }
    /// Map a predicate handler with default [PredicateOptions].
    pub fn map_predicate_handler<H>(&mut self, name: &str, sql_expression: SqlExpr, handler: H)
    where
        H: 'static + PredicateHandler + Send + Sync,
    {
        self.map_predicate_handler_with_options(
            name,
            sql_expression,
            handler,
            PredicateOptions::new(),
        )
    }
    /// Map a predicate expression with default [PredicateOptions].
    pub fn map_predicate(&mut self, name: &str, sql_expression: SqlExpr) {
        self.map_predicate_with_options(name, sql_expression, PredicateOptions::new());
    }

    /// Map a predicate expression with a custom [PredicateHandler].
    pub fn map_predicate_handler_with_options<H>(
        &mut self,
        name: &str,
        sql_expression: SqlExpr,
        handler: H,
        options: PredicateOptions,
    ) where
        H: 'static + PredicateHandler + Send + Sync,
    {
        let predicate = Predicate {
            expression: sql_expression,
            handler: Arc::new(handler),
            options,
        };
        self.predicates.insert(name.to_string(), predicate);
    }
    /// Map a predicate expression.
    pub fn map_predicate_with_options(
        &mut self,
        name: &str,
        sql_expression: SqlExpr,
        options: PredicateOptions,
    ) {
        let predicate = Predicate {
            expression: sql_expression,
            handler: self.predicate_handler.clone(),
            options,
        };
        self.predicates.insert(name.to_string(), predicate);
    }

    /// Map a selection.
    pub fn map_selection(&mut self, name: &str, fields_or_paths: Vec<String>) {
        if cfg!(debug_assertion) && name.len() <= 3 {
            panic!(
                "Selection name `{}` is invalid: name must be longer than 3 characters.",
                name
            );
        }
        self.selections.insert(name.to_string(), fields_or_paths);
    }

    /// Restrict deleted this `struct` with a role expression.
    pub fn restrict_delete(&mut self, role_expr: RoleExpr) {
        self.delete_role_expr = Some(role_expr);
    }

    /// Restrict loading this `struct` with a role expression.  
    pub fn restrict_load(&mut self, role_expr: RoleExpr) {
        self.load_role_expr = Some(role_expr);
    }
}
