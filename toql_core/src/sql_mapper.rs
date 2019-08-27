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

use crate::query::FieldFilter;
use crate::sql_builder::SqlBuilderError;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;

use enquote::unquote;

#[derive(Debug)]
#[allow(dead_code)] // IMPROVE Having AND None are considered unused
pub(crate) enum FilterType {
    Where,
    Having,
    None,
}

#[derive(Debug)]
pub(crate) struct SqlTarget {
    pub(crate) options: MapperOptions,                   // Options
    pub(crate) filter_type: FilterType,                  // Filter on where or having clause
    pub(crate) handler: Arc<FieldHandler + Send + Sync>, // Handler to create clauses
    pub(crate) subfields: bool,                          // Target name has subfields separated by underscore
    pub(crate) expression: String,                       // Column name or SQL expression
}

/// Handles the standart filters as documented in the guide.
/// Returns [FilterInvalid](../sql_builder/enum.SqlBuilderError.html) for any attempt to use FN filters.
#[derive(Debug, Clone)]
pub struct BasicFieldHandler {}

#[derive(Debug)]
/// Options for a mapped field.
pub struct MapperOptions {
    pub(crate) always_selected: bool,   // Always select this field, regardless of query fields
    pub(crate) count_filter: bool,      // Filter field on count query
    pub(crate) count_select: bool,      // Select field on count query
    pub(crate) ignore_wildcard: bool,   // Ignore field for wildcard selection
    pub(crate) roles: BTreeSet<String>, // Only for use by these roles
}


impl MapperOptions {

    /// Create new mapper options
    pub fn new() -> Self {
        MapperOptions {
            always_selected: false,
            count_filter: false,
            count_select: false,
            ignore_wildcard: false,
            roles: BTreeSet::new(),
        }
    }
   


    /// Field is always selected, regardless of the query.
    pub fn select_always(mut self, always_selected: bool) -> Self {
        self.always_selected = always_selected;
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
    pub fn restrict_roles(mut self, roles: BTreeSet<String>) -> Self {
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
    /// Return sql if you want to select it.
    fn build_select(&self, sql: &str, _query_params: &HashMap<String, String>) -> Option<String> {
       Some(format!("{}", sql))
    }
    
    /// Match filter and return SQL expression.
    /// Do not insert parameters in the SQL expression, use `?` instead.
    /// If you miss some arguments, raise an error, typically `SqlBuilderError::FilterInvalid`
    fn build_filter(&self, sql: &str, _filter: &FieldFilter, query_params: &HashMap<String, String>) ->Result<Option<String>, crate::sql_builder::SqlBuilderError>;
    /// Return the parameters for your `?`
    fn build_param(&self, _filter: &FieldFilter, _query_params: &HashMap<String, String>) -> Vec<String>;
    /// Return addition SQL join clause for this field or None
     fn build_join(&self,  _query_params: &HashMap<String, String>) -> Option<String> {
        None
    } 
}

impl std::fmt::Debug for (dyn FieldHandler + std::marker::Send + std::marker::Sync + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FieldHandler()")
    }
}

pub fn sql_param(s: String) -> String {
    if s.chars().next().unwrap_or(' ')== '\'' {
        return unquote(&s).expect("Argument invalid")    // Must be valid, because Pest rule
    } 
        s
    
}

impl FieldHandler for BasicFieldHandler {
    
    fn build_param(&self, filter: &FieldFilter,  _query_params: &HashMap<String, String>) -> Vec<String> {
        match filter {
            FieldFilter::Eq(criteria) => vec![sql_param(criteria.clone()) ],
            FieldFilter::Eqn => vec![],
            FieldFilter::Ne(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Nen => vec![],
            FieldFilter::Ge(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Gt(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Le(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Lt(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Bw(lower, upper) => vec![sql_param(lower.clone()), sql_param(upper.clone())],
            FieldFilter::Re(criteria) => vec![sql_param(criteria.clone())],
       //     FieldFilter::Sc(criteria) => vec![criteria.clone()],
            FieldFilter::In(args) => args.iter().map(|a| sql_param(a.to_string())).collect(),
            FieldFilter::Out(args) => args.iter().map(|a| sql_param(a.to_string())).collect(), //args.clone(),
            FieldFilter::Lk(criteria) => vec![sql_param(criteria.clone())],
            FieldFilter::Fn(_name, _args) => vec![], // must be implemented by user
        }
    }

    fn build_filter(&self, expression: &str, filter: &FieldFilter, _query_params: &HashMap<String, String>) ->Result<Option<String>,  crate::sql_builder::SqlBuilderError> {
        match filter {
            FieldFilter::Eq(_) => Ok(Some(format!("{} = ?", expression))),
            FieldFilter::Eqn => Ok(Some(format!("{} IS NULL", expression))),
            FieldFilter::Ne(_) => Ok(Some(format!("{} <> ?", expression))),
            FieldFilter::Nen => Ok(Some(format!("{} IS NOT NULL", expression))),
            FieldFilter::Ge(_) => Ok(Some(format!("{} >= ?", expression))),
            FieldFilter::Gt(_) => Ok(Some(format!("{} > ?", expression))),
            FieldFilter::Le(_) => Ok(Some(format!("{} <= ?", expression))),
            FieldFilter::Lt(_) => Ok(Some(format!("{} < ?", expression))),
            FieldFilter::Bw(_, _) => Ok(Some(format!("{} BETWEEN ? AND ?", expression))),
            FieldFilter::Re(_) => Ok(Some(format!("{} RLIKE ?", expression))),
            FieldFilter::In(values) => Ok(Some(format!(
                "{} IN ({})",
                expression,
                std::iter::repeat("?")
                    .take(values.len())
                    .collect::<Vec<&str>>()
                    .join(",")
            ))),
            FieldFilter::Out(values) => Ok(Some(format!(
                "{} NOT IN ({})",
                expression,
                std::iter::repeat("?")
                    .take(values.len())
                    .collect::<Vec<&str>>()
                    .join(",")
            ))),
      //      FieldFilter::Sc(_) => Ok(Some(format!("FIND_IN_SET (?, {})", expression))),
            FieldFilter::Lk(_) => Ok(Some(format!("{} LIKE ?", expression))),
            FieldFilter::Fn(name, _) => Err(SqlBuilderError::FilterInvalid(format!("no filter `{}` found.", name))), // Must be implemented by user
        }
    }
}

/// A cache that holds mappers.
//pub type SqlMapperCache = HashMap<String, SqlMapper>;

#[derive(Debug)]
pub struct SqlMapperCache {
    pub mappers: HashMap<String, SqlMapper>,
    
}
impl SqlMapperCache {
    pub fn new() -> SqlMapperCache {
        SqlMapperCache {
            mappers: HashMap::new()
        }
    }   
    pub fn insert_new_mapper<M:Mapped>(
        &mut self,
        ) -> String {
            let  m = SqlMapper::from_mapped::<M>();
            self.mappers.insert(String::from(M::table_name()), m);
            M::table_name()
            //self.cache.get_mut(&M::table_name()).unwrap()
        }
        pub fn insert_new_mapper_with_handler<M:Mapped,H>(
            &mut self,
            
            handler: H,
        ) -> String
        where
            H: 'static + FieldHandler + Send + Sync,
        {
            let m = SqlMapper::from_mapped_with_handler::<M, _>( handler);
            self.mappers.insert(String::from(M::table_name()), m);
            M::table_name()
        }

}


/// Translates Toql fields into columns or SQL expressions.
#[derive(Debug)]
pub struct SqlMapper {
    pub(crate) handler: Arc<FieldHandler + Send + Sync>,
    pub(crate) table: String,
    pub(crate) field_order: Vec<String>,
    pub(crate) fields: HashMap<String, SqlTarget>,
    pub(crate) joins: HashMap<String, Join>,
}

#[derive(Debug)]
pub(crate) struct Join {
    pub(crate) join_clause: String,    // LEFT JOIN ... ON ..
    pub(crate) selected: bool          // This join will always appear in query and fields should be selected
}
/// Structs that implement `Mapped` can be added to the mapper with [map()](struct.SqlMapper.html#method.map).
/// 
/// The Toql derive implements this trait for derived structs.
pub trait Mapped {
    fn table_name() -> String;
    fn table_alias() -> String;
    fn map(mapper: &mut SqlMapper, toql_path: &str, sql_alias: &str);       // Map entity fields
}

impl SqlMapper {

     /// Create new mapper for _table_ or _table alias_.
     /// Example: `::new("Book")` or `new("Book b")`.
     /// If you use an alias you must map all
     /// SQL columns with the alias too.
     pub fn new<T>(table: T)  -> Self
      where  T: Into<String>
     {
         let f = BasicFieldHandler {};
         Self::new_with_handler(table,f)
     }
     /// Creates new mapper with a custom handler.
     /// Use this to provide custom filter functions for all fields.
    pub fn new_with_handler<T, H>(table: T, handler: H) -> Self
    where
        T: Into<String>,
        H: 'static + FieldHandler + Send + Sync // TODO improve lifetime
    {
        SqlMapper {
            handler: Arc::new(handler),
            table: table.into(),
            joins: HashMap::new(),
            fields: HashMap::new(),
            field_order: Vec::new(),
        }
    }
    pub fn from_mapped<M:Mapped>() -> SqlMapper                           // Create new SQL Mapper and map entity fields
    {
        Self::from_mapped_with_alias::<M>(&M::table_alias())
    }
    pub fn from_mapped_with_alias<M:Mapped>(sql_alias: &str) -> SqlMapper                           // Create new SQL Mapper and map entity fields
    {
         let s = format!("{} {}", M::table_name(), sql_alias);
        let mut m =
            Self::new(if sql_alias.is_empty() { M::table_name() } else { s });
        M::map(&mut m, "", sql_alias);
        m
    }
    pub fn from_mapped_with_handler<M: Mapped,H>(handler: H) -> SqlMapper
      where
        H: 'static + FieldHandler + Send + Sync,
    {
              let s = format!("{} {}",M::table_name(), M::table_alias());
        let mut m = SqlMapper::new_with_handler(
            if M::table_alias().is_empty() { M::table_name() } else { s },
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
    pub fn map_join<'a, T: Mapped>(
        &'a mut self,
        toql_path: &str,
        sql_alias: &str,
    ) -> &'a mut Self {
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
        options: MapperOptions,
    ) -> &'a mut Self 
     where H: 'static + FieldHandler + Send + Sync
    {
        let t = SqlTarget {
            options: options,
            filter_type: FilterType::Where, // Filter on where clause
            subfields: toql_field.find('_').is_some(),
            handler: Arc::new(handler),
            expression: expression.to_string(),
        };
        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }
    /// Changes the handler of a field.
    /// This will panic if the field does not exist
    /// Use it to make changes, it prevents typing errors of field names.
    pub fn alter_handler<H>(
        &mut self,
        toql_field: &str,
        handler: H,
    ) -> &mut Self 
     where H: 'static + FieldHandler + Send + Sync
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
        handler: Arc<FieldHandler + Sync + Send>,
        options: MapperOptions,
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
        options: MapperOptions,
    ) -> &mut Self {
        let sql_target = self.fields.get_mut(toql_field).expect(&format!(
            "Cannot alter \"{}\": Field is not mapped.",
            toql_field
        ));
        sql_target.expression = sql_expression.to_string();
        sql_target.options = options;
        self
    }
    /// Adds a new field -or updates an existing field- to the mapper.
    pub fn map_field<'a>(&'a mut self, toql_field: &str, sql_field: &str) -> &'a mut Self {
        self.map_field_with_options(toql_field, sql_field, MapperOptions::new())
    }

    /// Adds a new field -or updates an existing field- to the mapper.
    pub fn map_field_with_options<'a>(
        &'a mut self,
        toql_field: &str,
        sql_expression: &str,
        options: MapperOptions,
    ) -> &'a mut Self {
       
        let t = SqlTarget {
            expression: sql_expression.to_string(),
            options: options,
            filter_type: FilterType::Where, // Filter on where clause
            subfields: toql_field.find('_').is_some(),
            handler: Arc::clone(&self.handler)
        };

        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }
    /// Adds a join for a given path to the mapper. 
    /// Example: `map.join("foo", "LEFT JOIN Foo f ON (foo_id = f.id)")`
    pub fn join<'a>(&'a mut self, toql_path: &str, join_clause: &str, selected: bool) -> &'a mut Self {
        self.joins.insert(
            toql_path.to_string(),
            Join {
                join_clause: join_clause.to_string(),
                selected: selected
            },
        );

        // Find targets that use join and set join field

        self
    }
    /// Changes an already added join.
    /// This will panic if the join does not exist
    /// Use it to make changes, it prevents typing errors of path names.
    pub fn alter_join<'a>(&'a mut self, toql_path: &str, join_clause: &str) -> &'a mut Self {
        let j = self.joins.get_mut(toql_path).expect("Join is missing.");
        j.join_clause = join_clause.to_string();
        self
    }
}
