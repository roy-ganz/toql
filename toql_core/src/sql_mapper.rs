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
//! ### Caching
//! If a struct contains merged fields (collections of structs) then the SQL Builder must build multiple SQL queries with different mappers.
//! To give high level functions all SQL Mappers, they must be put into a cache. This allows to
//! load the full dependency tree.
//!

use crate::alias::AliasFormat;
use crate::query::FieldFilter;
use crate::sql_builder::SqlBuilderError;
use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt;
use enquote::unquote;



#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum SqlMapperError {
    /// The requested canonical alias is not used. Contains the alias name.
    CanonicalAliasMissing(String),
    ColumnMissing(String, String),  // table column
}
impl fmt::Display for SqlMapperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlMapperError::CanonicalAliasMissing(ref s) => write!(f, "canonical sql alias `{}` is missing", s),
            SqlMapperError::ColumnMissing(ref t, ref c) => write!(f, "sql column `{}` is missing on database table `{}`", t, c),
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
    pub(crate) options: FieldOptions,                   // Options
    pub(crate) filter_type: FilterType,                  // Filter on where or having clause
    pub(crate) handler: Arc<dyn FieldHandler + Send + Sync>, // Handler to create clauses
    pub(crate) subfields: bool, // Target name has subfields separated by underscore
    pub(crate) expression: String, // Column name or SQL expression
    pub(crate) sql_query_params: Vec<String>, // Query_params for SQL expressions
}


impl SqlTarget {
    pub fn sql_query_param_values(&self, build_params: &HashMap<String, String>) -> Result<Vec<String>, SqlBuilderError> {
                let mut params : Vec<String> = Vec::with_capacity(self.sql_query_params.len());
                    for p in &self.sql_query_params {
                    let qp = 
                        build_params
                        .get(p)
                        .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
                        params.push(qp.to_owned());
                    }
                    Ok(params)

    }


}


/// Handles the standart filters as documented in the guide.
/// Returns [FilterInvalid](../sql_builder/enum.SqlBuilderError.html) for any attempt to use FN filters.
#[derive(Debug, Clone)]
pub struct BasicFieldHandler {}

impl BasicFieldHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
/// Options for a mapped field.
pub struct FieldOptions {
    pub(crate) preselect: bool, // Always select this field, regardless of query fields
    pub(crate) count_filter: bool, // Filter field on count query
    pub(crate) count_select: bool, // Select field on count query
    pub(crate) ignore_wildcard: bool, // Ignore field for wildcard selection
    pub(crate) roles: HashSet<String>, // Only for use by these roles
    pub(crate) filter_only: bool,   // This field cannot be loaded, its only used as a filter
}

impl FieldOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        FieldOptions {
            preselect: false,
            count_filter: false,
            count_select: false,
            ignore_wildcard: false,
            roles: HashSet::new(),
            filter_only: false
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
    pub fn ignore_wildcard(mut self, ignore_wildcard: bool) -> Self {
        self.ignore_wildcard = ignore_wildcard;
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

    /// The field can only be used to filter query results. 
    /// It is ommitted in the select statement and cannot be deserialized
    pub fn filter_only(mut self, filter_only: bool) -> Self {
        self.filter_only = filter_only;
        self
    }
}

/// Options for a mapped field.
#[derive(Debug)]
pub struct JoinOptions {
    pub(crate) preselect: bool, // Always select this join, regardless of query fields
    pub(crate) ignore_wildcard: bool, // Ignore field on this join for wildcard selection
    pub(crate) roles: HashSet<String>, // Only for use by these roles
}

impl JoinOptions {
    /// Create new mapper options
    pub fn new() -> Self {
        JoinOptions {
            preselect: false,
            ignore_wildcard: false,
            roles: HashSet::new(),
        }
    }

    /// Field is selected, regardless of the query.
    pub fn preselect(mut self, preselect: bool) -> Self {
        self.preselect = preselect;
        self
    }
 
    /// Field is ignored by the wildcard.
    pub fn ignore_wildcard(mut self, ignore_wildcard: bool) -> Self {
        self.ignore_wildcard = ignore_wildcard;
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
}

trait MapperFilter {
    fn build(field: crate::query::QueryToken) -> String;
}
/// A FieldHandler maps a Toql field onto an SQL.
/// Use it to
/// - define your own custom function (through FN)
/// - map the standart filters differently
/// - disallow standart filters
/// - handle fields that do not exist in the struct
/// - handle fields that match multiple columns (full text index)
///
/// ## Example (see full working example in tests)
/// ``` ignore
/// use toql::query::FieldFilter;
/// use toql::sql_mapper::FieldHandler;
/// use toql::sql_builder::SqlBuilderError;
/// struct MyHandler {};
///
/// impl FieldHandler for MyHandler {
///     fn build_filter(&self, sql: &str, _filter: &FieldFilter)
///     ->Result<Option<String>, SqlBuilderError> {
///        --snip--
///     }
///     fn build_param(&self, _filter: &FieldFilter) -> Vec<String> {
///         --snip--
///     }
/// }
/// let my_handler = MyHandler {};
/// let mapper = SqlMapper::new_with_handler(my_handler);
///
pub trait FieldHandler {
    /// Return sql and params if you want to select it.
    fn build_select(
        &self,
        select: (String,Vec<String>),
        _build_params: &HashMap<String, String>,
    ) -> Result<Option<(String, Vec<String>)>, crate::sql_builder::SqlBuilderError> {
        Ok(Some(select))
    }

    /// Match filter and return SQL expression.
    /// Do not insert parameters in the SQL expression, use `?` instead.
    /// If you miss some arguments, raise an error, typically `SqlBuilderError::FilterInvalid`
    fn build_filter(
        &self,
        _select: (String, Vec<String>),
        _filter: &FieldFilter,
        build_params: &HashMap<String, String>,
    ) -> Result<Option<(String, Vec<String>)>, crate::sql_builder::SqlBuilderError>;
    /* /// Return the parameters for your `?`
    fn build_param(
        &self,
        _filter: &FieldFilter,
        _query_params: &HashMap<String, String>,
    ) -> Vec<String>; */
    /// Return addition SQL join clause for this field or None
    fn build_join(
        &self,
        _build_params: &HashMap<String, String>,
    ) -> Result<Option<String>, crate::sql_builder::SqlBuilderError> {
        Ok(None)
    }
}

impl std::fmt::Debug for (dyn FieldHandler + std::marker::Send + std::marker::Sync + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FieldHandler()")
    }
}

pub fn sql_param(s: String) -> String {
    if s.chars().next().unwrap_or(' ') == '\'' {
        return unquote(&s).expect("Argument invalid"); // Must be valid, because Pest rule
    }
    s
}

impl FieldHandler for BasicFieldHandler {
    /* fn build_param(
        &self,
        filter: &FieldFilter,
        _query_params: &HashMap<String, String>,
    ) -> Vec<String> {
        match filter {
            FieldFilter::Eq(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Eqn => vec![],
            FieldFilter::Ne(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Nen => vec![],
            FieldFilter::Ge(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Gt(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Le(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Lt(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Bw(lower, upper) => {
                vec![sql_param(lower.clone()), sql_param(upper.clone())]
            }
            FieldFilter::Re(criteria) => vec![sql_param(criteria.clone())],
            //     FieldFilter::Sc(criteria) => vec![criteria.clone()],
            FieldFilter::In(args) => args.iter().map(|a| sql_param(a.to_string())).collect(),
            FieldFilter::Out(args) => args.iter().map(|a| sql_param(a.to_string())).collect(), //args.clone(),
            FieldFilter::Lk(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Fn(_name, _args) => vec![], // must be implemented by user
        }
    } */

    fn build_filter(
        &self,
        mut select: (String, Vec<String>),
        filter: &FieldFilter,
        _build_params: &HashMap<String, String>,
    ) -> Result<Option<(String, Vec<String>)>, crate::sql_builder::SqlBuilderError> {
        match filter {
            FieldFilter::Eq(criteria) => Ok(Some((format!("{} = ?", select.0), {
                select.1.push(sql_param(criteria.clone()));
                select.1
            }))),
            FieldFilter::Eqn => Ok(Some((format!("{} IS NULL", select.0), select.1))),
            FieldFilter::Ne(criteria) => Ok(Some((format!("{} <> ?", select.0), {
                select.1.push(sql_param(criteria.clone()));
                select.1
            }))),
            FieldFilter::Nen => Ok(Some((format!("{} IS NOT NULL", select.0), select.1))),
            FieldFilter::Ge(criteria) => Ok(Some((format!("{} >= ?", select.0), {
                select.1.push(sql_param(criteria.clone()));
                select.1
            }))),
            FieldFilter::Gt(criteria) => Ok(Some((format!("{} > ?", select.0), {
                select.1.push(sql_param(criteria.clone()));
                select.1
            }))),
            FieldFilter::Le(criteria) => Ok(Some((format!("{} <= ?", select.0), {
                select.1.push(sql_param(criteria.clone()));
                select.1
            }))),
            FieldFilter::Lt(criteria) => Ok(Some((format!("{} < ?", select.0), {
                select.1.push(sql_param(criteria.clone()));
                select.1
            }))),
            FieldFilter::Bw(lower, upper) => Ok(Some((format!("{} BETWEEN ? AND ?", select.0), {
                select.1.push(sql_param(lower.clone()));
                select.1.push(sql_param(upper.clone()));
                select.1
            }))),
            FieldFilter::Re(criteria) => Ok(Some((format!("{} RLIKE ?", select.0), {
                select.1.push(sql_param(criteria.clone()));
                select.1
            }))),
            FieldFilter::In(args) => Ok(Some((
                format!(
                    "{} IN ({})",
                    select.0,
                    std::iter::repeat("?")
                        .take(args.len())
                        .collect::<Vec<&str>>()
                        .join(",")
                ),
                {
                    let a: Vec<String> = args.iter().map(|a| sql_param(a.to_string())).collect();
                    select.1.extend_from_slice(&a);
                    select.1
                },
            ))),
            FieldFilter::Out(args) => Ok(Some((
                format!(
                    "{} NOT IN ({})",
                    select.0,
                    std::iter::repeat("?")
                        .take(args.len())
                        .collect::<Vec<&str>>()
                        .join(",")
                ),
                {
                    let a: Vec<String> = args.iter().map(|a| sql_param(a.to_string())).collect();
                    select.1.extend_from_slice(&a);
                    select.1
                },
            ))),
            //      FieldFilter::Sc(_) => Ok(Some(format!("FIND_IN_SET (?, {})", expression))),
            FieldFilter::Lk(criteria) => Ok(Some((format!("{} LIKE ?", select.0), {
                select.1.push(sql_param(criteria.clone()));
                select.1
            }))),
            FieldFilter::Fn(name, _) => Err(SqlBuilderError::FilterInvalid(name.to_owned())), // Must be implemented by user
        }
    }
}

/// A cache that holds mappers.
//pub type SqlMapperCache = HashMap<String, SqlMapper>;

#[derive(Debug)]
pub struct SqlMapperCache {
    pub mappers: HashMap<String, SqlMapper>,
    pub alias_format: AliasFormat,                         // 
}
impl SqlMapperCache {
    pub fn new(alias_format: AliasFormat) -> SqlMapperCache {
        SqlMapperCache {
            mappers: HashMap::new(),
            alias_format,
        }
    }
    pub fn insert_new_mapper<M: Mapped>(&mut self) -> String {
        let mut m = SqlMapper::from_mapped::<M>( self.alias_format.clone());
        //m.aliased_table = m.translate_aliased_table(&M::table_name(), &M::table_alias());
        self.mappers.insert(String::from(M::type_name()), m);
        M::type_name()
    }
    pub fn insert_new_mapper_with_handler<M: Mapped, H>(&mut self, handler: H) -> String
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        let mut m = SqlMapper::from_mapped_with_handler::<M, _>(self.alias_format.clone(), handler);
       // m.aliased_table = m.translate_aliased_table(&M::table_name(), &M::table_alias());
        self.mappers.insert(String::from(M::type_name()), m);
        M::type_name()
    }
}

/// Translates Toql fields into columns or SQL expressions.
#[derive(Debug)]
pub struct SqlMapper {
    pub aliased_table: String,
    pub alias_format: AliasFormat,                         // 
    pub params: HashMap<String, String>,                  // builds params
    
    pub(crate) handler: Arc<dyn FieldHandler + Send + Sync>,
    pub(crate) field_order: Vec<String>,
    pub(crate) fields: HashMap<String, SqlTarget>,
    pub(crate) joins: HashMap<String, Join>,
    pub(crate) joins_root: Vec<String>,                  // Top joins
    pub(crate) joins_tree: HashMap<String,  Vec<String>>, // Subjoins
    
    pub(crate) alias_translation: HashMap<String, String>,
   
    pub(crate) table_index: u16,                          //table index for aliases
}

#[derive(Debug, PartialEq)]
pub enum JoinType {
    Left,
    Inner
}

#[derive(Debug)]
pub(crate) struct Join {
    pub(crate) join_type : JoinType, // LEFT JOIN ... 
    pub(crate) aliased_table: String, // Table t0
    pub(crate) on_predicate: String, // ON ..
    pub (crate) options: JoinOptions,
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
    pub fn new<T>(table: T, alias_format: AliasFormat) -> Self
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
            handler: Arc::new(handler),
            aliased_table: aliased_table.into(),
            joins: HashMap::new(),
            fields: HashMap::new(),
            field_order: Vec::new(),
            joins_root: Vec::new(),
            joins_tree: HashMap::new(),
            params: HashMap::new(),
            table_index: 0,   // will be incremented before use
            alias_format: alias_format,
            alias_translation: HashMap::new()
        }
    }
    pub fn from_mapped<M: Mapped>(alias_format: AliasFormat) -> SqlMapper // Create new SQL Mapper and map entity fields
    {
        Self::from_mapped_with_alias::<M>(&M::table_alias(), alias_format)
    }
    pub fn from_mapped_with_alias<M: Mapped>(sql_alias: &str, alias_format: AliasFormat) -> SqlMapper // Create new SQL Mapper and map entity fields
    {
        let s = format!("{} {}", M::table_name(), sql_alias);
         let mut m = Self::new(if sql_alias.is_empty() {
            M::table_name()
        } else {
            s
        }, alias_format); 
        M::map(&mut m, "", &M::table_alias());
        m
    }
    pub fn from_mapped_with_handler<M: Mapped, H>(alias_format: AliasFormat, handler: H) -> SqlMapper
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

    /*  /// Creates and inserts a new mapper into a cache.
    /// Returns a mutable reference to the created mapper. Use it for configuration.
    pub fn insert_new_mapper<T: Mapped>(cache: &mut SqlMapperCache) -> &mut SqlMapper {
        T::insert_new_mapper(cache)
    }
    /// Creates a new mapper with a custom field handler and insert it into a cache.
    /// Returns a mutable reference to the created mapper. Use it for configuration.
     pub fn insert_new_mapper_with_handler<T, H>(cache: &mut SqlMapperCache, handler: H) -> &mut SqlMapper
     where T: Mapped,
            H: 'static + FieldHandler + Send + Sync // TODO improve lifetime
     {
        T::insert_new_mapper_with_handler(cache, handler)
    }
    /// Maps all fields from a struct.
    /// This trait is implemented by the Toql derive for derived structs.
    pub fn map<T: Mapped>(sql_alias: &str) -> Self {
        // Mappable must create mapper for top level table
        T::new_mapper(sql_alias)
    } */
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
        let mut sql_query_params = Vec::new();
        let sql_expression = query_param_regex.replace(&sql_expression, |e: &regex::Captures| {
            let name = &e[1];
            sql_query_params.push(name.to_string());
            "?"
        });

        let t = SqlTarget {
            options: options,
            filter_type: FilterType::Where, // Filter on where clause
            subfields: toql_field.find('_').is_some(),
            handler: Arc::new(handler),
            expression: sql_expression.to_string(),
            sql_query_params: sql_query_params,
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
        let mut sql_query_params = Vec::new();
        let sql_expression = query_param_regex.replace(&sql_expression, |e: &regex::Captures| {
            let name = &e[1];
            sql_query_params.push(name.to_string());
            "?"
        });

        let t = SqlTarget {
            expression: sql_expression.to_string(),
            options: options,
            filter_type: FilterType::Where, // Filter on where clause
            subfields: toql_field.find('_').is_some(),
            handler: Arc::clone(&self.handler),
            sql_query_params: sql_query_params,
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
        self.join_with_options(toql_path, join_type, aliased_table, on_predicate, JoinOptions::new())

    }
    pub fn join_with_options<'a>(
        &'a mut self,
        toql_path: &str,
        join_type: JoinType,
        aliased_table: &str,
        on_predicate: &str,
        options: JoinOptions
       
    ) -> &'a mut Self {
        self.joins.insert(
            toql_path.to_string(),
            Join {
                join_type,
                aliased_table: aliased_table.to_string(),
                on_predicate: on_predicate.to_string(),
                options,
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
           let head :&str = toql_path.trim_end_matches(|c| c != '_').trim_end_matches('_');

            let j = self.joins_tree.entry(head.to_string()).or_insert(Vec::new());
            j.push(toql_path.to_string());
        }
        

        // Find targets that use join and set join field

        self
    }
    /// Changes an already added join.
    /// This will panic if the join does not exist
    /// Use it to make changes, it prevents typing errors of path names.
    pub fn alter_join<'a>(&'a mut self, toql_path: &str, join_type:JoinType, aliased_table: &str, on_predicate:&str) -> &'a mut Self {
        let j = self.joins.get_mut(toql_path).expect("Join is missing.");
        j.join_type = join_type;
        j.aliased_table = aliased_table.to_string();
        j.on_predicate = on_predicate.to_string();
        self
    }
    /// Translates a canonical sql alias into a shorter alias
    pub fn translate_alias(&mut self, canonical_alias : &str) -> String {
        use std::collections::hash_map::Entry;
        
            
          let a =   match self.alias_translation.entry(canonical_alias.to_owned()) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => {
                    let alias  =match self.alias_format {
                        AliasFormat::TinyIndex => {self.table_index = self.table_index + 1; AliasFormat::tiny_index(self.table_index)},
                        AliasFormat::ShortIndex => {self.table_index = self.table_index + 1; AliasFormat::short_index(&canonical_alias, self.table_index)},
                        AliasFormat::MediumIndex => {self.table_index = self.table_index + 1; AliasFormat::medium_index(&canonical_alias, self.table_index)},
                        _ => canonical_alias.to_owned()
                        };
                    v.insert(alias)}
            }.to_owned();

            //println!("{} -> {}", canonical_alias, &a);
            a
    }
     /// Returns a translated alias or the canonical alias if it's not been translated
    pub fn translated_alias(&self, canonical_alias : &str) -> String {
       
       // println!("{:?}", self.alias_translation);
        self.alias_translation.get(canonical_alias).unwrap_or(&canonical_alias.to_owned()).to_owned()
          
    }
    /// Helper method to build an aliased column with a canonical sql alias
    /// Example for `AliasFormat::TinyIndex`: user_address_country.id translates into t1.id 
    pub fn translate_aliased_column(&mut self,canonical_alias: &str, column: &str) -> String{
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
    pub fn aliased_column(&self,canonical_alias: &str, column: &str) -> String{
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
    pub fn translate_aliased_table(&mut self,table: &str, canonical_alias: &str ) -> String{
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
    pub fn replace_aliases(&mut self,sql_with_aliases: &str) -> Result<String,  SqlMapperError>{
              lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new(r"\[([\w_]+)\]").unwrap();
        }

        if cfg!(debug_assertions) {
            for m in  REGEX.find_iter(sql_with_aliases) {
                if !self.alias_translation.contains_key(m.as_str()) {
                    return Err(SqlMapperError::CanonicalAliasMissing(m.as_str().to_owned()));
                }
            }
        }
        
        let sql = REGEX.replace(sql_with_aliases, |e: &regex::Captures| {
            let canonical_alias = &e[1];
            let alias = self.alias_translation.get(canonical_alias);
            if  let Some(a) = alias {
                a.to_owned()
            }else {
                String::from(canonical_alias)
            }
        });
        Ok(sql.to_string())
    }

}
