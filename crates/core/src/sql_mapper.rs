//!
//! The SQL Mapper translates Toql fields into database columns or SQL expressions.
//!
//! It's needed by the  [SQL Builder](../sql_builder/struct.SqlBuilder.html) to turn a [Query](../query/struct.Query.html)
//! into a [SQL Builder Result](../sql_builder_result/SqlBuilderResult.html).
//! The result holds the different parts of an SQL query. It can be turned into SQL that can be sent to the database.
//!
//! ## Example
//! ``` ignore
//! let mapper = SqlMapper::new("Bar b")
//!     .map_field("foo", "b.foo")
//!     .map_field("fuu_id", "u.foo")
//!     .map_field("faa", "(SELECT COUNT(*) FROM Faa WHERE Faa.bar_id = b.id)")
//!     .join("fuu", "LEFT JOIN Fuu u ON (b.foo_id = u.id)");
//! ```
//!
//! To map a full struct with [map()](struct.SqlMapper.html#method.map), the struct must implement the [Mapped](trait.Mapped.html) trait.
//! The [Toql derive](https://docs.rs/toql_derive/0.1/index.html) implements that trait for any derived struct.
//!
//! ### Options
//! Field can have options. They can be hidden for example or require a certain role (permission).
//! Use [map_with_options()](struct.SqlMapper.html#method.map_with_options).
//!
//! ### Filter operations
//! Beside fields and joins the SQL Mapper also registers all filter operations.
//! To add a custom operation you must define a new [Fieldhandler](trait.FieldHandler.html)
//! and add it either
//!  - to a single field with [map_handler()](struct.SqlMapper.html#method.map_handler).
//!  - to all fields with [new_with_handler()](struct.SqlMapper.html#method.new_with_handler).
//!


use crate::predicate_handler::{PredicateHandler, DefaultPredicateHandler};
use crate::alias::AliasFormat;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use crate::field_handler::{FieldHandler, BasicFieldHandler};
use crate::join_handler::{JoinHandler, DefaultJoinHandler};
use crate::sql::SqlArg;




#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum SqlMapperError {
    /// The requested canonical alias is not used. Contains the alias name.
    CanonicalAliasMissing(String),
    ColumnMissing(String, String), // table column
}
impl fmt::Display for SqlMapperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlMapperError::CanonicalAliasMissing(ref s) => {
                write!(f, "canonical sql alias `{}` is missing", s)
            }
            SqlMapperError::ColumnMissing(ref t, ref c) => {
                write!(f, "database table `{}` is missing column `{}`", t, c)
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)] // IMPROVE Having AND None are considered unused
pub(crate) enum FilterType {
    Where,
    Having,
    None,
}

#[derive(Debug)]
pub(crate) struct SqlTarget {
    pub(crate) options: FieldOptions,                        // Options
    pub(crate) filter_type: FilterType,                      // Filter on where or having clause
    pub(crate) handler: Arc<dyn FieldHandler + Send + Sync>, // Handler to create clauses
    pub(crate) subfields: bool, // Target name has subfields separated by underscore
    pub(crate) expression: String, // Column name or SQL expression
    pub(crate) sql_aux_param_names: Vec<String>, //  Extracted from <aux_param>
}
/* 
impl SqlTarget {
   /*  pub fn sql_aux_param_values(
        &self,
        aux_params: &HashMap<String, String>,
    ) -> Result<Vec<String>, SqlBuilderError> {
        let mut params: Vec<String> = Vec::with_capacity(self.sql_aux_param_names.len());
        for p in &self.sql_aux_param_names {
            let qp = aux_params
                .get(p)
                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
            params.push(qp.to_owned());
        }
        Ok(params)
    } */
} */


#[derive(Debug, Clone)]
/// Options for a mapped field.
pub struct FieldOptions {
    pub(crate) preselect: bool, // Always select this field, regardless of query fields
    pub(crate) count_filter: bool, // Filter field on count query
    pub(crate) count_select: bool, // Select field on count query
    pub(crate) mut_select: bool, // Select field on mut select
    pub(crate) skip_wildcard: bool, // Skip field for wildcard selection
    pub(crate) roles: HashSet<String>, // Only for use by these roles
    pub(crate) aux_params: HashMap<String, SqlArg>, // Auxiliary params
    pub(crate) on_params: Vec<String>,  // Identity params for on clauses
}

impl FieldOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        FieldOptions {
            preselect: false,
            count_filter: false,
            count_select: false,
            mut_select: false,
            skip_wildcard: false,
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            on_params: Vec::new()
        }
    }

    /// Field is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }
    /// Any filter on the field is considered when creating a count query.
    /// Typically applied to fields that represent permissions and foreign keys.
    /// Assumme a user wants to see all books. You will restrict the user query
    /// with a permission filter, so that the user sees all of *his* books.
    /// The count query must also use the filter.
    pub fn count_filter(mut self, count_filter: bool) -> Self {
        self.count_filter = count_filter;
        self
    }
    /// Any selected field is also used for the count query.
    /// Only used in rare cases where you fiddle with distinct results.
    pub fn count_select(mut self, count_select: bool) -> Self {
        self.count_select = count_select;
        self
    }
    /// Field is ignored by the wildcard.
    pub fn skip_wildcard(mut self, skip_wildcard: bool) -> Self {
        self.skip_wildcard = skip_wildcard;
        self
    }
    /// The field can only be selected and filtered by queries that have
    /// these roles.
    /// Example: The email address is only visible to users with
    /// the _admin_ role.
    pub fn restrict_roles(mut self, roles: HashSet<String>) -> Self {
        self.roles = roles;
        self
    }

    
     /// Additional build param. This is used by the query builder together with
     /// its build params. Build params can be used in SQL expressions (`SELECT <param_name>` )
     /// and field handlers.
    pub fn aux_param<S, T>(mut self, name: S, value: T) -> Self 
    where S: Into<String>, T: Into<SqlArg>
    {
        self.aux_params.insert(name.into(), value.into());
        self
    }
}

/// Options for a mapped field.
#[derive(Debug)]
pub struct JoinOptions {
    pub(crate) preselect: bool, // Always select this join, regardless of query fields
    pub(crate) skip_wildcard: bool, // Ignore field on this join for wildcard selection
    pub(crate) roles: HashSet<String>, // Only for use by these roles
    pub(crate) aux_params: HashMap<String, SqlArg>, // Additional build params
    pub(crate) join_handler: Option<Arc<dyn JoinHandler + Send + Sync>> // Optional join handler
        
}

impl JoinOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        JoinOptions {
            preselect: false,
            skip_wildcard: false,
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            join_handler:None
        }
    }

    /// Field is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }

    /// Field is ignored by the wildcard.
    pub fn skip_wildcard(mut self, skip_wildcard: bool) -> Self {
        self.skip_wildcard = skip_wildcard;
        self
    }
    /// The field can only be selected and filtered by queries that have
    /// these roles.
    /// Example: The email address is only visible to users with
    /// the _admin_ role.
    pub fn restrict_roles(mut self, roles: HashSet<String>) -> Self {
        self.roles = roles;
        self
    }

    /// Additional build param. This is used by the query builder together with
    /// its build params. Build params can be used in SQL expressions (`SELECT <param_name>` )
    /// and field handlers.
    pub fn aux_param<S, T>(mut self, name: S, value: T) -> Self 
    where S: Into<String>, T:Into<SqlArg>
    {
        self.aux_params.insert(name.into(), value.into());
        self
    }
}

#[derive(Debug)]
pub struct PredicateOptions {
     pub(crate) aux_params: HashMap<String, SqlArg>,
     pub(crate) on_params: Vec<(u8,String)>,  // Argument params for on clauses (index, name)
     pub(crate) count_filter: bool
}

impl PredicateOptions {

    pub fn new() -> Self {
        PredicateOptions { aux_params: HashMap::new(), on_params: Vec::new(), count_filter: false}
    }

 /// Additional build param. This is used by the query builder together with
     /// its build params. Build params can be used in SQL expressions (`SELECT <param_name>` )
     /// and field handlers.
    pub fn aux_param<S, T>(mut self, name: S, value: T) -> Self 
    where S: Into<String>, T:Into<SqlArg>
    {
        self.aux_params.insert(name.into(), value.into());
        self
    }

    /// Additional build param. This is used by the query builder together with
     /// its build params. Build params can be used in SQL expressions (`SELECT <param_name>` )
     /// and field handlers.
    pub fn on_param(mut self, index: u8, name: String) -> Self {
        self.on_params.push((index, name));
        self
    }
    /// By default predicates are considered when creating a count query.
    /// However the predicate can be ignored by setting the count filter to false
    pub fn count_filter(mut self, count_filter: bool) -> Self {
        self.count_filter = count_filter;
        self
    }
}

trait MapperFilter {
    fn build(field: crate::query::QueryToken) -> String;
}

/// Translates Toql fields into columns or SQL expressions.
#[derive(Debug)]
pub struct SqlMapper {
    pub aliased_table: String,
    pub alias_format: AliasFormat,       //
    pub(crate) field_handler: Arc<dyn FieldHandler + Send + Sync>, // Default field handler
    pub(crate) predicate_handler: Arc<dyn PredicateHandler + Send + Sync>, // Default predicate handler
    pub(crate) field_order: Vec<String>,
    pub(crate) fields: HashMap<String, SqlTarget>,
    pub(crate) predicates: HashMap<String, Predicate>,
    pub(crate) joins: HashMap<String, Join>,
    pub(crate) joins_root: Vec<String>, // Top joins
    pub(crate) joins_tree: HashMap<String, Vec<String>>, // Subjoins

    pub(crate) alias_translation: HashMap<String, String>,

    pub(crate) table_index: u16, //table index for aliases
}

#[derive(Debug, PartialEq)]
pub enum JoinType {
    Left,
    Inner,
}

#[derive(Debug)]
pub(crate) struct Predicate {
    pub(crate) expression: String,
    pub(crate) handler: Arc<dyn PredicateHandler + Send + Sync>, // Handler to create clauses
    pub(crate) sql_aux_param_names: Vec<String>, // aux params in predicate statement or ? in correct order
    pub(crate) options: PredicateOptions,
}



#[derive(Debug)]
pub(crate) struct Join {
    pub(crate) join_type: JoinType,   // LEFT JOIN ...
    pub(crate) aliased_table: String, // Table t0
    pub(crate) on_predicate: String,  // ON ..
    pub(crate) options: JoinOptions,
    pub(crate) sql_aux_param_names: Vec<String>, // aux params in ON clause
}
/// Structs that implement `Mapped` can be added to the mapper with [map()](struct.SqlMapper.html#method.map).
///
/// The Toql derive implements this trait for derived structs.
pub trait Mapped {
    fn table_name() -> String;
    fn table_alias() -> String;
    fn type_name() -> String;
    fn map(mapper: &mut SqlMapper, toql_path: &str, sql_alias: &str); // Map entity fields
}

impl SqlMapper {
     /// Create new mapper for _table_ or _table alias_.
    /// Example: `::new("Book")` or `new("Book b")`.
    /// If you use an alias you must map all
    /// SQL columns with the alias too.
     pub fn new<T>(table: T) -> Self
    where
        T: Into<String>,
    {
        Self::with_alias_format(table, AliasFormat::Canonical)
    }
    /// Create new mapper for _table_ or _table alias_.
    /// The alias format defines how aliases are look like, if
    /// the Sql Mapper is called to build them.
    pub fn with_alias_format<T>(table: T, alias_format: AliasFormat) -> Self
    where
        T: Into<String>,
    {
        let f = BasicFieldHandler {};
        Self::new_with_handler(table, alias_format, f)
    }
    /// Creates new mapper with a custom handler.
    /// Use this to provide custom filter functions for all fields.
    pub fn new_with_handler<T, H>(aliased_table: T, alias_format: AliasFormat, handler: H) -> Self
    where
        T: Into<String>,
        H: 'static + FieldHandler + Send + Sync, // TODO improve lifetime
    {
        SqlMapper {
            field_handler: Arc::new(handler),
            predicate_handler: Arc::new(DefaultPredicateHandler::new()),
            aliased_table: aliased_table.into(),
            joins: HashMap::new(),
            fields: HashMap::new(),
            field_order: Vec::new(),
            predicates: HashMap::new(),
            joins_root: Vec::new(),
            joins_tree: HashMap::new(),
            
            table_index: 0, // will be incremented before use
            alias_format: alias_format,
            alias_translation: HashMap::new(),
        }
    }
    /// Create a new mapper from a struct that implements the Mapped trait.
    /// The Toql derive does that for every attributed struct.
    pub fn from_mapped<M: Mapped>() -> SqlMapper 
    {
        Self::from_mapped_with_alias::<M>(&M::table_alias(), AliasFormat::Canonical)
    }
     /// Create a new mapper from a struct that implements the Mapped trait.
    /// The alias format defines what the table aliases in Sql look like.
    pub fn from_mapped_with_alias<M: Mapped>(
        sql_alias: &str,
        alias_format: AliasFormat,
    ) -> SqlMapper // Create new SQL Mapper and map entity fields
    {
        let s = format!("{} {}", M::table_name(), sql_alias);
        let mut m = Self::with_alias_format(
            if sql_alias.is_empty() {
                M::table_name()
            } else {
                s
            },
            alias_format,
        );
        M::map(&mut m, "", &M::table_alias());
        m
    }
    pub fn from_mapped_with_handler<M: Mapped, H>(
        alias_format: AliasFormat,
        handler: H,
    ) -> SqlMapper
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let s = format!("{} {}", M::table_name(), M::table_alias());
        let mut m = SqlMapper::new_with_handler(
            if M::table_alias().is_empty() {
                M::table_name()
            } else {
                s
            },
            alias_format,
            handler,
        );

        M::map(&mut m, "", &M::table_alias());
        m
    }

    
    /// Maps all fields from a struct as a joined dependency.
    /// Example: To map for a user an `Address` struct that implements `Mapped`
    /// ``` ignore
    /// user_mapper.map_join<Address>("address", "a");
    /// ```
    pub fn map_join<'a, T: Mapped>(&'a mut self, toql_path: &str, sql_alias: &str) -> &'a mut Self {
        T::map(self, toql_path, sql_alias);
        self
    }
    /// Maps a Toql field to a field handler.
    /// This allows most freedom, you can define in the [FieldHandler](trait.FieldHandler.html)
    /// how to generate SQL for your field.
    pub fn map_handler<'a, H>(
        &'a mut self,
        toql_field: &str,
        expression: &str,
        handler: H,
    ) -> &'a mut Self
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        self.map_handler_with_options(toql_field, expression, handler, FieldOptions::new())
    }
    // Maps a Toql field with options to a field handler.
    /// This allows most freedom, you can define in the [FieldHandler](trait.FieldHandler.html)
    /// how to generate SQL for your field.
    pub fn map_handler_with_options<'a, H>(
        &'a mut self,
        toql_field: &str,
        sql_expression: &str,
        handler: H,
        options: FieldOptions,
    ) -> &'a mut Self
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        // TODO put into function
        let query_param_regex = regex::Regex::new(r"<([\w_]+)>").unwrap();
        let sql_expression = sql_expression.to_string();
        let mut sql_aux_param_names = Vec::new();
        let sql_expression = query_param_regex.replace(&sql_expression, |e: &regex::Captures| {
            let name = &e[1];
            sql_aux_param_names.push(name.to_string());
            "?"
        });

        let t = SqlTarget {
            options: options,
            filter_type: FilterType::Where, // Filter on where clause
            subfields: toql_field.find('_').is_some(),
            handler: Arc::new(handler),
            expression: sql_expression.to_string(),
            sql_aux_param_names: sql_aux_param_names,
        };
        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }
    /// Changes the handler of a field.
    /// This will panic if the field does not exist
    /// Use it to make changes, it prevents typing errors of field names.
    pub fn alter_handler<H>(&mut self, toql_field: &str, handler: H) -> &mut Self
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let sql_target = self.fields.get_mut(toql_field).expect(&format!(
            "Cannot alter \"{}\": Field is not mapped.",
            toql_field
        ));

        sql_target.handler = Arc::new(handler);
        self
    }
    /// Changes the handler and options of a field.
    /// This will panic if the field does not exist
    /// Use it to make changes, it prevents typing errors of field names.
    pub fn alter_handler_with_options(
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
    }
    /// Changes the database column or SQL expression of a field.
    /// This will panic if the field does not exist
    /// Use it to make changes, it prevents typing errors of field names.
    pub fn alter_field(
        &mut self,
        toql_field: &str,
        sql_expression: &str,
        options: FieldOptions,
    ) -> &mut Self {
        let sql_target = self.fields.get_mut(toql_field).expect(&format!(
            "Cannot alter \"{}\": Field is not mapped.",
            toql_field
        ));
        sql_target.expression = sql_expression.to_string();
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
    } 

    /// Adds a new field - or updates an existing field - to the mapper.
    pub fn map_field<'a>(&'a mut self, toql_field: &str, sql_field: &str) -> &'a mut Self {
        self.map_field_with_options(toql_field, sql_field, FieldOptions::new())
    }

    /// Adds a new field - or updates an existing field - to the mapper.
    pub fn map_field_with_options<'a>(
        &'a mut self,
        toql_field: &str,
        sql_expression: &str,
        options: FieldOptions,
    ) -> &'a mut Self {
        // If sql_expression contains query params replace them with ?
        let query_param_regex = regex::Regex::new(r"<([\w_]+)>").unwrap();
        let sql_expression = sql_expression.to_string();
        let mut sql_aux_param_names = Vec::new();
        let sql_expression = query_param_regex.replace(&sql_expression, |e: &regex::Captures| {
            let name = &e[1];
            sql_aux_param_names.push(name.to_string());
            "?"
        });

        let t = SqlTarget {
            expression: sql_expression.to_string(),
            options: options,
            filter_type: FilterType::Where, // Filter on where clause
            subfields: toql_field.find('_').is_some(),
            handler: Arc::clone(&self.field_handler),
            sql_aux_param_names,
        };

        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }
    /// Adds a join for a given path to the mapper.
    /// Example: `map.join("foo", "LEFT JOIN Foo f ON (foo_id = f.id)")`
    pub fn join<'a>(
        &'a mut self,
        toql_path: &str,
        join_type: JoinType,
        aliased_table: &str,
        on_predicate: &str,
    ) -> &'a mut Self {
        self.join_with_options(
            toql_path,
            join_type,
            aliased_table,
            on_predicate,
            JoinOptions::new(),
        )
    }
    pub fn join_with_options<'a>(
        &'a mut self,
        toql_path: &str,
        join_type: JoinType,
        aliased_table: &str,
        on_predicate: &str,
        options: JoinOptions,
    ) -> &'a mut Self {
        let query_param_regex = regex::Regex::new(r"<([\w_]+)>").unwrap();
        let on_predicate = on_predicate.to_string();
        let mut sql_aux_param_names = Vec::new();
        let on_predicate = query_param_regex.replace_all(&on_predicate, |e: &regex::Captures| {
            let name = &e[1];
            sql_aux_param_names.push(name.to_string());
            "?"
        });
        self.joins.insert(
            toql_path.to_string(),
            Join {
                join_type,
                aliased_table: aliased_table.to_string(),
                on_predicate: on_predicate.to_string(),
                options,
                sql_aux_param_names,
            },
        );

        // Precalculate tree information for quicker join construction
        // Build root joins and store child joins to parent joins
        // Eg. [user] = [user_country, user_address, user_info]

        let c = toql_path.matches('_').count();
        if c == 0 {
            self.joins_root.push(toql_path.to_string());
        } else {
            // Add path to base path
            let head: &str = toql_path
                .trim_end_matches(|c| c != '_')
                .trim_end_matches('_');

            let j = self
                .joins_tree
                .entry(head.to_string())
                .or_insert(Vec::new());
            j.push(toql_path.to_string());
        }

        // Find targets that use join and set join field

        self
    }
    /// Changes an already added join.
    /// This will panic if the join does not exist
    /// Use it to make changes, it prevents typing errors of path names.
    pub fn alter_join<'a>(
        &'a mut self,
        toql_path: &str,
        join_type: JoinType,
        aliased_table: &str,
        on_predicate: &str,
    ) -> &'a mut Self {
        let j = self.joins.get_mut(toql_path).expect("Join is missing.");
        j.join_type = join_type;
        j.aliased_table = aliased_table.to_string();
        j.on_predicate = on_predicate.to_string();
        self
    }

    pub fn map_predicate_handler<H>(&mut self, name: &str, sql_expression :&str, handler: H) 
      where  H: 'static + PredicateHandler + Send + Sync,
    {

       self.map_predicate_handler_with_options(name, sql_expression, handler, PredicateOptions::new())

    }
    pub fn map_predicate(&mut self, name: &str, sql_expression :&str) {
         
        self.map_predicate_with_options(name, sql_expression, PredicateOptions::new());
    }
    pub fn map_predicate_handler_with_options<H>(&mut self, name: &str, sql_expression :&str, handler: H, options: PredicateOptions) 
      where  H: 'static + PredicateHandler + Send + Sync,
    {

       let (expression, sql_aux_param_names) = Self::predicate_argument_names(sql_expression);
                
        let predicate = Predicate {
            expression,
            sql_aux_param_names,
            handler:  Arc::new(handler),
            options,
        };
        self.predicates.insert(name.to_string(), predicate);

    }
    pub fn map_predicate_with_options(&mut self, name: &str, sql_expression :&str, options: PredicateOptions) {
         
        let (expression, sql_aux_param_names) = Self::predicate_argument_names(sql_expression);
                
        let predicate = Predicate {
            expression,
            sql_aux_param_names,
            handler: self.predicate_handler.clone(),
            options,
        };
        self.predicates.insert(name.to_string(), predicate);
        

    }
  

    /// Translates a canonical sql alias into a shorter alias
    pub fn translate_alias(&mut self, canonical_alias: &str) -> String {
        use std::collections::hash_map::Entry;

        let a = match self.alias_translation.entry(canonical_alias.to_owned()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let alias = match self.alias_format {
                    AliasFormat::TinyIndex => {
                        self.table_index = self.table_index + 1;
                        AliasFormat::tiny_index(self.table_index)
                    }
                    AliasFormat::ShortIndex => {
                        self.table_index = self.table_index + 1;
                        AliasFormat::short_index(&canonical_alias, self.table_index)
                    }
                    AliasFormat::MediumIndex => {
                        self.table_index = self.table_index + 1;
                        AliasFormat::medium_index(&canonical_alias, self.table_index)
                    }
                    _ => canonical_alias.to_owned(),
                };
                v.insert(alias)
            }
        }
        .to_owned();

        //println!("{} -> {}", canonical_alias, &a);
        a
    }
    /// Returns a translated alias or the canonical alias if it's not been translated
    pub fn translated_alias(&self, canonical_alias: &str) -> String {
        // println!("{:?}", self.alias_translation);
        self.alias_translation
            .get(canonical_alias)
            .unwrap_or(&canonical_alias.to_owned())
            .to_owned()
    }
    /// Helper method to build an aliased column with a canonical sql alias
    /// Example for `AliasFormat::TinyIndex`: user_address_country.id translates into t1.id
    pub fn translate_aliased_column(&mut self, canonical_alias: &str, column: &str) -> String {
        let translated_alias = self.translate_alias(canonical_alias);
        format!(
            "{}{}{}",
            &translated_alias,
            if canonical_alias.is_empty() { "" } else { "." },
            column
        )
    }
    /// Helper method to build an aliased column with a canonical sql alias
    /// Example for `AliasFormat::TinyIndex`: user_address_country.id translates into t1.id
    pub fn aliased_column(&self, canonical_alias: &str, column: &str) -> String {
        let translated_alias = self.translated_alias(canonical_alias);
        format!(
            "{}{}{}",
            &translated_alias,
            if canonical_alias.is_empty() { "" } else { "." },
            column
        )
    }
    /// Helper method to build an aliased table with a canonical SQL alias
    /// Example for `AliasFormat::TinyIndex`:  Country user_address_country translates into Country t1
    pub fn translate_aliased_table(&mut self, table: &str, canonical_alias: &str) -> String {
        let translated_alias = self.translate_alias(canonical_alias);
        format!(
            "{}{}{}",
            table,
            if canonical_alias.is_empty() { "" } else { " " },
            &translated_alias
        )
    }

    /// Helper method to replace alias placeholders in SQL expression
    /// '[canonical_sql_alias]' is replaced with translated alias
    pub fn replace_aliases(&mut self, sql_with_aliases: &str) -> Result<String, SqlMapperError> {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new(r"\[([\w_]+)\]").unwrap();
        }

        if cfg!(debug_assertions) {
            for m in REGEX.find_iter(sql_with_aliases) {
                if !self.alias_translation.contains_key(m.as_str()) {
                    return Err(SqlMapperError::CanonicalAliasMissing(m.as_str().to_owned()));
                }
            }
        }

        let sql = REGEX.replace(sql_with_aliases, |e: &regex::Captures| {
            let canonical_alias = &e[1];
            let alias = self.alias_translation.get(canonical_alias);
            if let Some(a) = alias {
                a.to_owned()
            } else {
                String::from(canonical_alias)
            }
        });
        Ok(sql.to_string())
    }

    /// Extract aux parameter names and arguments from predicate expression 
    /// Example: `SELECT 1 WHERE <a> and ? and <b>` yields  the tuple 
    /// ("SELECT 1 WHERE ? and ? and ?" , ["a", "?", "b"] )
    fn predicate_argument_names(sql_expression: &str) -> (String, Vec<String>) {
                lazy_static! {
                    static ref REGEX: regex::Regex = regex::Regex::new(r"<([\w_]+)>|\?").unwrap();
                }
                let mut sql_aux_param_names :Vec<String> = Vec::new();
                
                for cap in REGEX.captures_iter(sql_expression) {
                    if cap[0] == *"?" {
                        sql_aux_param_names.push("?".to_owned());
                    } else {
                        sql_aux_param_names.push(cap[1].to_owned());
                    }
                }

                let replaced_sql_expression = REGEX.replace_all(sql_expression, "?");

                ( replaced_sql_expression.to_owned().to_string(), sql_aux_param_names)
         }
}
