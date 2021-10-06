//!
//! The SQL Mapper translates Toql fields into database columns or SQL expressions.
//!
//! It's needed by the  [SQL Builder](../sql_builder/struct.SqlBuilder.html) to turn a [Query](../query/struct.Query.html)
//! into a [SQL Builder Result](../sql_builder_result/SqlBuilderResult.html).
//! The result holds the different parts of an SQL query. It can be turned into SQL that can be sent to the database.
//!
//! ## Example
//! ``` ignore
//! let mapper = TableMapper::new("Bar b")
//!     .map_field("foo", "b.foo")
//!     .map_field("fuu_id", "u.foo")
//!     .map_field("faa", "(SELECT COUNT(*) FROM Faa WHERE Faa.bar_id = b.id)")
//!     .join("fuu", "LEFT JOIN Fuu u ON (b.foo_id = u.id)");
//! ```
//!
//! To map a full struct with [map()](struct.TableMapper.html#method.map), the struct must implement the [Mapped](trait.Mapped.html) trait.
//! The [Toql derive](https://docs.rs/toql_derive/0.1/index.html) implements that trait for any derived struct.
//!
//! ### Options
//! Field can have options. They can be hidden for example or require a certain role (permission).
//! Use [map_with_options()](struct.TableMapper.html#method.map_with_options).
//!
//! ### Filter operations
//! Beside fields and joins the SQL Mapper also registers all filter operations.
//! To add a custom operation you must define a new [Fieldhandler](trait.FieldHandler.html)
//! and add it either
//!  - to a single field with [map_handler()](struct.TableMapper.html#method.map_handler).
//!  - to all fields with [new_with_handler()](struct.TableMapper.html#method.new_with_handler).
//!
pub mod field_options;
pub mod merge_options;
pub mod join_options;
pub mod join_type;
pub mod mapped;
pub mod predicate_options;

pub(crate) mod field;
pub(crate) mod join;
pub(crate) mod merge;
pub(crate) mod predicate;

use heck::{CamelCase, MixedCase};

use crate::table_mapper::join_options::JoinOptions;
use crate::table_mapper::merge_options::MergeOptions;
use crate::table_mapper::predicate_options::PredicateOptions;

use crate::predicate_handler::{DefaultPredicateHandler, PredicateHandler};
use crate::result::Result;
use crate::table_mapper::field::Field;
use crate::table_mapper::field_options::FieldOptions;
use crate::table_mapper::join::Join;
use crate::table_mapper::mapped::Mapped;
use crate::table_mapper::merge::Merge;
use crate::table_mapper::predicate::Predicate;

use std::collections::HashMap;

use crate::field_handler::{BasicFieldHandler, FieldHandler};
use crate::{role_expr::RoleExpr, sql_expr::SqlExpr};
use join_type::JoinType;
use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
pub enum DeserializeType {
    Field(String), // Toql fieldname
    Join(String),  // Toql join path
    Merge(String), // Toql merge path
                   //Embedded
}

#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum TableMapperError {
    /// The requested canonical alias is not used. Contains the alias name.
    CanonicalAliasMissing(String),
    ColumnMissing(String, String), // table column
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

trait MapperFilter {
    fn build(field: crate::query::QueryToken) -> String;
}

/// Translates Toql fields into columns or SQL expressions.
#[derive(Debug)]
pub struct TableMapper {
    pub table_name: String,
    pub canonical_table_alias: String, // Calculated from table_name

    /* pub aliased_table: String,
    pub alias_format: AliasFormat,       // */
    pub(crate) field_handler: Arc<dyn FieldHandler + Send + Sync>, // Default field handler
    pub(crate) predicate_handler: Arc<dyn PredicateHandler + Send + Sync>, // Default predicate handler

    // pub(crate) field_order: Vec<String>,

    // Deserialization order for selects statements
    pub(crate) deserialize_order: Vec<DeserializeType>,

    /// Joined mappers
    pub(crate) joined_mappers: HashMap<String, String>, // Toql path, Mapper name

    /// Field information
    pub(crate) fields: HashMap<String, Field>,

    /// Predicate information
    pub(crate) predicates: HashMap<String, Predicate>,

    /// Join information
    pub(crate) joins: HashMap<String, Join>,

    /// Merge information
    pub(crate) merges: HashMap<String, Merge>,

    /// Load role
    pub(crate) load_role_expr: Option<RoleExpr>,

    /// Delete role
    pub(crate) delete_role_expr: Option<RoleExpr>,

    /// Selections
    /// Automatic created selection are
    /// #cnt - Fields for count query
    /// #mut - Fields for insert
    /// #all - All mapped fields
    pub(crate) selections: HashMap<String, Vec<String>>, // name, toql fields or paths

                                                         //    pub(crate) joins_root: Vec<String>, // Top joins
                                                         //    pub(crate) joins_tree: HashMap<String, Vec<String>>, // Subjoins
}

impl TableMapper {
    /// Create new mapper for _table_ or _table alias_.
    /// Example: `::new("Book")` or `new("Book b")`.
    /// If you use an alias you must map all
    /// SQL columns with the alias too.
    pub fn new<T>(sql_table_name: &str) -> Self
where {
        let f = BasicFieldHandler {};
        Self::new_with_handler(sql_table_name, f)
    }
    /// Create new mapper for _table_ or _table alias_.
    /// The alias format defines how aliases are look like, if
    /// the Sql Mapper is called to build them.
    /*  pub fn with_alias_format<T>(table: T, alias_format: AliasFormat) -> Self
    where
        T: Into<String>,
    {
        let f = BasicFieldHandler {};
        Self::new_with_handler(table, alias_format, f)
    } */
    /// Creates new mapper with a custom handler.
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
            joined_mappers: HashMap::new(),
            selections: HashMap::new(),
            load_role_expr: None,
            delete_role_expr: None,
        }
    }
    /// Create a new mapper from a struct that implements the Mapped trait.
    /// The Toql derive does that for every attributed struct.
    pub fn from_mapped<M: Mapped>() -> Result<TableMapper> {
        Self::from_mapped_with_handler::<M, _>(BasicFieldHandler::new())
    }

    pub fn from_mapped_with_handler<M: Mapped, H>(handler: H) -> Result<TableMapper>
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let mut m = TableMapper::new_with_handler(&M::table_name(), handler);

        M::map(&mut m)?;
        Ok(m)
    }
    pub fn joined_mapper(&self, name: &str) -> Option<String> {
        self.join(name).map(|j| j.joined_mapper.to_owned())
    }
      pub fn is_partial_join(&self, name: &str) -> bool {
          self.join(name).filter(|j| j.options.partial_table).is_some()
      }
    pub fn merged_mapper(&self, name: &str) -> Option<String> {
        self.merge(name).map(|m| m.merged_mapper.to_owned())
    }
    

    pub(crate) fn join(&self, name: &str) -> Option<&Join> {
        self.joins.get(name)
    }
    pub(crate) fn merge(&self, name: &str) -> Option<&Merge> {
        self.merges.get(name)
    }
    pub(crate) fn field(&self, name: &str) -> Option<&Field> {
        self.fields.get(name)
    }

    pub(crate) fn joined_partial_mappers(&self) -> Vec<(String,String)> {
        self.joins.iter().filter_map(|(n, j)| if j.options.partial_table{ Some((n.to_string(), j.joined_mapper.to_string()))} else {None}).collect()
    }



    

    /* /// Maps all fields from a struct as a joined dependency.
       /// Example: To map for a user an `Address` struct that implements `Mapped`
       /// ``` ignore
       /// user_mapper.map_join<Address>("address", "a");
       /// ```
       pub fn map_join<'a, T: Mapped>(&'a mut self, toql_path: &str, sql_alias: &str) -> &'a mut Self {
           //T::map(self, toql_path, sql_alias);

           self
       }
    */

    /// Maps a Toql field to a field handler.
    /// This allows most freedom, you can define in the [FieldHandler](trait.FieldHandler.html)
    /// how to generate SQL for your field.
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
    // Maps a Toql field with options to a field handler.
    /// This allows most freedom, you can define in the [FieldHandler](trait.FieldHandler.html)
    /// how to generate SQL for your field.
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
        // TODO put into function
        /*  let query_param_regex = regex::Regex::new(r"<([\w_]+)>").unwrap();
        let sql_expression = sql_expression;
        let mut sql_aux_param_names = Vec::new();
        let sql_expression = query_param_regex.replace(&sql_expression, |e: &regex::Captures| {
            let name = &e[1];
            sql_aux_param_names.push(name.to_string());
            "?"
        }); */

        let t = Field {
            options,
            handler: Arc::new(handler),
            expression,
            // sql_aux_param_names: sql_aux_param_names,
        };
        self.deserialize_order
            .push(DeserializeType::Field(toql_field.to_string()));
        self.fields.insert(toql_field.to_string(), t);
        self
    }
    /// Changes the handler of a field.
    /// This will panic if the field does not exist
    /// Use it to make changes, it prevents typing errors of field names.
    /*  pub fn alter_handler<H>(&mut self, toql_field: &str, handler: H) -> &mut Self
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let sql_target = self.fields.get_mut(toql_field).expect(&format!(
            "Cannot alter \"{}\": Field is not mapped.",
            toql_field
        ));

        sql_target.handler = Arc::new(handler);
        self
    } */
    /// Changes the handler and options of a field.
    /// This will panic if the field does not exist
    /// Use it to make changes, it prevents typing errors of field names.
    /* pub fn alter_handler_with_options(
        &mut self,
        toql_field: &str,
        handler: Arc<dyn FieldHandler + Sync + Send>,
        options: FieldOptions,
    ) -> &mut Self {
        let sql_target = self.fields.get_mut(toql_field).expect(&format!(
            "Cannot alter \"{}\": Field is not mapped.",
            toql_field
        ));
        sql_target.options = options;
        sql_target.handler = handler;
        self
    } */
    /// Changes the database column or SQL expression of a field.
    /// This will panic if the field does not exist
    /// Use it to make changes, it prevents typing errors of field names.
    /* pub fn alter_field(
        &mut self,
        toql_field: &str,
        sql_expression: SqlExpr,
        options: FieldOptions,
    ) -> &mut Self {
        let sql_target = self.fields.get_mut(toql_field).expect(&format!(
            "Cannot alter \"{}\": Field is not mapped.",
            toql_field
        ));
        sql_target.expression = sql_expression;
        sql_target.options = options;
        self
    }

     pub fn get_options(&self, toql_field: &str) -> Option<FieldOptions> {
        match self.fields.get(toql_field) {
            Some(f) => Some(f.options.clone()),
            None => None
        }
    }

    pub fn set_options(&mut self, toql_field: &str, options: FieldOptions )  {
        let f =  self.fields.get_mut(toql_field).expect(&format!(
            "Cannot alter \"{}\": Field is not mapped.",
            toql_field
        ));
        f.options = options;
    }  */

    /// Adds a new field - or updates an existing field - to the mapper.
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

    /// Adds a new field - or updates an existing field - to the mapper.
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

    /// Adds a new field - or updates an existing field - to the mapper.
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
    /// Adds a join for a given path to the mapper.
    /// Example: `map.join("foo", "LEFT JOIN Foo f ON (foo_id = f.id)")`
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
       self.map_merge_with_options(toql_path, merged_mapper, merge_join, merge_predicate, MergeOptions::new())
    }
    pub fn map_merge_with_options<S>(
        &mut self,
        toql_path: S,
        merged_mapper: &str,
        merge_join: SqlExpr,
        merge_predicate: SqlExpr,
        options: MergeOptions
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
                options
            },
        );
        self
    }
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
    pub fn map_predicate(&mut self, name: &str, sql_expression: SqlExpr) {
        self.map_predicate_with_options(name, sql_expression, PredicateOptions::new());
    }
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

    pub fn map_selection(&mut self, name: &str, fields_or_paths: Vec<String>) {
        if cfg!(debug_assertion) && name.len() <= 3 {
            panic!(
                "Selection name `{}` is invalid: name must be longer than 3 characters.",
                name
            );
        }
        self.selections.insert(name.to_string(), fields_or_paths);
    }

    pub fn restrict_delete(&mut self, role_expr: RoleExpr) {
        self.delete_role_expr = Some(role_expr);
    }

    pub fn restrict_load(&mut self, role_expr: RoleExpr) {
        self.load_role_expr = Some(role_expr);
    }
}
