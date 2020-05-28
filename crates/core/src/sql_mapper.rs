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

pub(crate) mod field_options;
pub(crate) mod field;
pub(crate) mod join_options;
pub(crate) mod join;
pub(crate) mod predicate_options;
pub(crate) mod predicate;
pub(crate) mod mapped;


use heck::MixedCase;
use crate::sql_mapper::predicate_options::PredicateOptions;
use crate::sql_mapper::join_options::JoinOptions;
use crate::sql_mapper::join::JoinType;
use crate::sql_mapper::field_options::FieldOptions;
use crate::sql_mapper::mapped::Mapped;
use crate::sql_mapper::join::Join;
use crate::sql_mapper::predicate::Predicate;
use crate::sql_mapper::field::{Field, FilterType};
use crate::predicate_handler::{PredicateHandler, DefaultPredicateHandler};
use crate::alias::AliasFormat;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use crate::field_handler::{FieldHandler, BasicFieldHandler};
use crate::join_handler::{JoinHandler, DefaultJoinHandler};
use crate::sql::SqlArg;
use crate::sql_expr::SqlExpr;
use crate::error::{Result, ToqlError};


#[derive(Debug)]
pub enum DeserializeType {
    Field(String), // Toql fieldname
    Join(String),  // Toql join path
    //Embedded
}


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




trait MapperFilter {
    fn build(field: crate::query::QueryToken) -> String;
}

/// Translates Toql fields into columns or SQL expressions.
#[derive(Debug)]
pub struct SqlMapper {
    pub table_name: String,
    pub canonical_table_alias: String, // Calculated form table_name

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

    /// Selections
    /// Automatic created selection are
    /// #count - Fields for count query
    /// #mut - Fields for insert
    /// #all - All mapped fields
    pub(crate) selections: HashMap<String, Vec<String>>, // name, toql fields or paths


//    pub(crate) joins_root: Vec<String>, // Top joins
//    pub(crate) joins_tree: HashMap<String, Vec<String>>, // Subjoins

   
   
}


impl SqlMapper {
     /// Create new mapper for _table_ or _table alias_.
    /// Example: `::new("Book")` or `new("Book b")`.
    /// If you use an alias you must map all
    /// SQL columns with the alias too.
     pub fn new<T>(sql_table_name: &str) -> Self
    where
    {
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
        SqlMapper {
            field_handler: Arc::new(handler),
            predicate_handler: Arc::new(DefaultPredicateHandler::new()),
            table_name: sql_table_name.to_string(),
            canonical_table_alias: sql_table_name.to_mixed_case(),
            joins: HashMap::new(),
            fields: HashMap::new(),
            predicates: HashMap::new(),
            deserialize_order: Vec::new(),
            joined_mappers: HashMap::new(),
            selections: HashMap::new(),
        }
    }
    /// Create a new mapper from a struct that implements the Mapped trait.
    /// The Toql derive does that for every attributed struct.
    pub fn from_mapped<M: Mapped>() -> SqlMapper 
    {
         Self::from_mapped_with_handler::<M, _>(BasicFieldHandler::new())

        //Self::from_mapped_with_::<M>(&M::table_alias(), AliasFormat::Canonical)
    }
     /// Create a new mapper from a struct that implements the Mapped trait.
    /// The alias format defines what the table aliases in Sql look like.
    /* pub fn from_mapped_with_alias<M: Mapped>(
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
           
        );
        M::map(&mut m, "", &M::table_alias());
        m
    } */
    pub fn from_mapped_with_handler<M: Mapped, H>(
        handler: H,
    ) -> SqlMapper
    where
        H: 'static + FieldHandler + Send + Sync,
    {
     
        let mut m = SqlMapper::new_with_handler(
            &M::table_name(),
            handler,
        );

        M::map(&mut m, "", &M::table_alias());
        m
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
    pub fn map_field_with_handler<'a, H>(
        &'a mut self,
        toql_field: &str,
        expression: SqlExpr,
        handler: H,
    ) -> &'a mut Self
    where
        H: 'static + FieldHandler + Send + Sync,
    {
        self.map_field_with_handler_and_options(toql_field, expression, handler, FieldOptions::new())
    }
    // Maps a Toql field with options to a field handler.
    /// This allows most freedom, you can define in the [FieldHandler](trait.FieldHandler.html)
    /// how to generate SQL for your field.
    pub fn map_field_with_handler_and_options<'a, H>(
        &'a mut self,
        toql_field: &str,
        sql_expression: SqlExpr,
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
            options: options,
            handler: Arc::new(handler),
            expression: sql_expression,
           // sql_aux_param_names: sql_aux_param_names,
        };
        self.deserialize_order.push(MapperType::Field(toql_field.to_string()));
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
    where T: Into<String>
    {
        self.map_expr_with_options(toql_field, SqlExpr::aliased_column(column_name.into()), FieldOptions::new())
    }

    /// Adds a new field - or updates an existing field - to the mapper.
    pub fn map_column_with_options<'a, T>(&'a mut self, toql_field: &str, column_name: T, options: FieldOptions,) -> &'a mut Self 
    where T: Into<String>
    {
        self.map_expr_with_options(toql_field, SqlExpr::aliased_column(column_name.into()), options)
    }
   
    /// Adds a new field - or updates an existing field - to the mapper.
    pub fn map_expr_with_options<'a>(
        &'a mut self,
        toql_field: &str,
        sql_expression: SqlExpr,
        options: FieldOptions,
    ) -> &'a mut Self {

        // Add count field to selection for quicker lookup
        if options.count_filter || options.count_select {
            let s = self.selections.entry("count".to_string()).or_insert(Vec::new());
            s.push(toql_field.to_string());
        }

        // Add mut field to selection for quicker lookup
        if options.mut_select  {
            let s = self.selections.entry("mut".to_string()).or_insert(Vec::new());
            s.push(toql_field.to_string());
        } 

     
        let t = Field {
            expression: sql_expression,
            options: options,
            handler: Arc::clone(&self.field_handler),
          //  sql_aux_param_names,
        };

        self.deserialize_order.push(MapperType::Field(toql_field.to_string()));
        self.fields.insert(toql_field.to_string(), t);

        self
    }
    /// Adds a join for a given path to the mapper.
    /// Example: `map.join("foo", "LEFT JOIN Foo f ON (foo_id = f.id)")`
    pub fn map_join<'a>(
        &'a mut self,
        toql_path: &str,
         joined_mapper: &str,
        join_expression: SqlExpr
    ) -> &'a mut Self {
        self.map_join_with_options(
            toql_path,
            joined_mapper,
            join_expression,
            JoinOptions::new(),
        )
    }
    pub fn map_join_with_options<'a, S, T>(
        &'a mut self,
        toql_path: S,
        joined_mapper: T,
        join_expression: SqlExpr,
        options: JoinOptions,
    ) -> &'a mut Self 
    where S: Into<String> + Clone, T: Into<String>
    {
      
             self.joins.insert(
            toql_path.clone().into(),
            Join {
               joined_mapper: joined_mapper.into(),
               join_expression,
                options,
            },
        );

        self.deserialize_order.push(MapperType::Join( toql_path.into()));

        /* // Precalculate tree information for quicker join construction
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
 */
        // Find targets that use join and set join field

        self
    }
    /// Changes an already added join.
    /// This will panic if the join does not exist
    /// Use it to make changes, it prevents typing errors of path names.
  /*   pub fn alter_join<'a>(
        &'a mut self,
        toql_path: &str,
        join_expression: SqlExpr,
    ) -> Result<&'a mut Self> {
        let j = self.joins.get_mut(toql_path).ok_or(ToqlError::MapperMissing)?;
        j.expression = join_expression;
        Ok(self)
    } */

    pub fn map_predicate_handler<H>(&mut self, name: &str, sql_expression :SqlExpr, handler: H) 
      where  H: 'static + PredicateHandler + Send + Sync,
    {

       self.map_predicate_handler_with_options(name, sql_expression, handler, PredicateOptions::new())

    }
    pub fn map_predicate(&mut self, name: &str, sql_expression :SqlExpr) {
         
        self.map_predicate_with_options(name, sql_expression, PredicateOptions::new());
    }
    pub fn map_predicate_handler_with_options<H>(&mut self, name: &str, sql_expression :SqlExpr, handler: H, options: PredicateOptions) 
      where  H: 'static + PredicateHandler + Send + Sync,
    {

                
        let predicate = Predicate {
            expression:sql_expression,
            handler:  Arc::new(handler),
            options,
        };
        self.predicates.insert(name.to_string(), predicate);

    }
    pub fn map_predicate_with_options(&mut self, name: &str, sql_expression :SqlExpr, options: PredicateOptions) {
                
        let predicate = Predicate {
            expression: sql_expression,
            handler: self.predicate_handler.clone(),
            options,
        };
        self.predicates.insert(name.to_string(), predicate);
        
    }
  

    /* /// Translates a canonical sql alias into a shorter alias
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
         } */
}
