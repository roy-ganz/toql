//!
//! The SQL Builder turns a [Query](../query/struct.Query.html) with the help of a [SQL Mapper](../sql_mapper/struct.SqlMapper.html)
//! into a [SQL Builder Result](../sql_builder_result/BuildResult.html)
//! The result hold the different parts of an SQL query and can be turned into an SQL query that can be sent to the database.
//!
//! ## Example
//!
//! ``` ignore
//!
//! let  query = Query::wildcard().and(Field::from("foo").eq(5));
//! let mapper::new("Bar b").map_field("foo", "b.foo");
//! let builder_result = QueryBuilder::new().build_query(&mapper, &query);
//! assert_eq!("SELECT b.foo FROM Bar b WHERE b.foo = ?", builder_result.to_sql());
//! assert_eq!(["5"], builder_result.params());
//! ```
//!
//! The SQL Builder can also add joins if needed. Joins must be registered on the SQL Mapper for this.
//!
//! ### Count queries
//! Besides normal queries the SQL Builder can als build count queries.
//!
//! Let's assume you have a grid view with books and the user enters a search term to filter your grid.
//! The normal query will get 50 books, but you will only display 10 books. Toql calls those 50 _the filtered count_.
//! To get the unfilted count, Toql must issue another query with different filter settings. Typically to get
//! the number of all books only that user has access to. Toql calls this _the total count_.
//!
//! ### Paths
//! The SQL Builder can also ignore paths to skip paths in the query that are not mapped in the mapper.
//! This is needed for structs that contain collections, as these collections must be querried with a different mapper.
//!
//! Let's assume a struct *user* had a collection of *phones*.
//! The Toql query may look like:  `username, phones_number`.
//! The SQL Builder needs 2 passes to resolve that query:
//!  - The first pass will query all users with the user mapper and will ignore the path *phones_*.
//!  - The second pass will only build the query for the path *phones_* with the help of the phone mapper.
//!

pub(crate) mod build_context;
pub(crate) mod build_result;
pub(crate) mod sql_builder_error;

/* use crate::sql_builder::eval_query::eval_query;
use crate::sql_builder::construct::build_join_clause;
use crate::sql_builder::construct::combine_aux_params;
use crate::sql_builder::construct::build_count_select_clause;
use crate::sql_builder::construct::build_select_clause;
use crate::sql_builder::construct::build_ordering;
use crate::sql_builder::sql_target_data::SqlTargetData; */
use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::error::ToqlError;
use crate::query::assert_roles;
use crate::query::concatenation::Concatenation;
use crate::query::field_order::FieldOrder;
use crate::query::Query;
use crate::query::{field_filter::FieldFilter, QueryToken};
use super::sql_builder::build_result::BuildResult;
use super::sql_builder::build_context::BuildContext;
use crate::sql_mapper::join::Join;
use crate::sql_mapper::join::JoinType;
use crate::sql_mapper::SqlMapper;
use crate::sql_mapper::field::Field as MapperField;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use crate::query::field::Field;
use super::parameter::Parameters;

use crate::sql_mapper_registry::SqlMapperRegistry;

use crate::sql::{Sql, SqlArg};
use crate::alias::AliasFormat;
use crate::alias_translator::AliasTranslator;
//use wildcard_scope::WildcardScope;

/// The Sql builder to build normal queries and count queries.
pub struct SqlBuilder<'a> {

    root_mapper: String,
    sql_mapper_registry: &'a SqlMapperRegistry,
    roles: HashSet<String>,
    aux_params: HashMap<String, SqlArg>, // Aux params used for all queries with this builder instance, contains typically config or auth data
     
    extra_joins: HashSet<String>,          // Use this joins
  //  alias_translator: &'a mut AliasTranslator
      
}


impl<'a> SqlBuilder<'a> {
    /// Create a new SQL Builder
    pub fn new(root_mapper: String, sql_mapper_registry: &'a SqlMapperRegistry) -> Self {
      
       SqlBuilder {
           root_mapper,
           sql_mapper_registry,
           roles :HashSet::new(),
           aux_params: HashMap::new(),
           extra_joins:HashSet::new(),
        }
    }

    /*  pub fn with_alias_translator(mut self, alia: &'a mut AliasTranslator) ->  Self {
        self.alias_translator = alia;
        self
    } */
   /*  pub fn with_alias_format(mut self, alias_format: AliasFormat) ->  Self {
        self.alias_translator = &mut AliasTranslator::new(alias_format);
        self
    } */
   
    pub fn with_roles(mut self, roles: HashSet<String>) ->  Self {
        self.roles = roles;
        self
    }
   
        
    pub fn with_extra_join<T: Into<String>>(mut self, join: T) -> Self {
        self.extra_joins.insert(join.into());
        self
    }

    


    pub fn build_delete_sql<M>(&mut self,  query: &Query<M>, modified: &str, extra: &str, alias_translator: &mut AliasTranslator) -> Result<Sql, ToqlError> {

        
        
        let context = BuildContext::new();
        let root_mapper = self.sql_mapper_registry.mappers.get(&self.root_mapper)
                .ok_or(ToqlError::MapperMissing(self.root_mapper.to_owned()))?;
        let alias_table = alias_translator.translate(&root_mapper.canonical_table_alias);
        let aliased_table = format!("{} {}",alias_table, root_mapper.table_name);
        let mut result = BuildResult::new(&aliased_table);
        //result.aliased_table = root_mapper.aliased_table;
                
        self.build_where_clause(&query, &context, &mut result, alias_translator)?;
        //self.build_join_clause(&context, &mut result)?;

        Ok((result.delete_stmt(), result.combined_params))
    }


    fn build_where_clause<M>(&mut self, query: &Query<M>, build_context: &BuildContext, result: &mut BuildResult, alias_translator: &mut AliasTranslator) -> Result<(), ToqlError>{

        let mapper = self.sql_mapper_registry.mappers.get(&self.root_mapper)
            .ok_or(ToqlError::MapperMissing(self.root_mapper.to_owned()))?; // from query type
        // put selected_paths: &mut HashSet<FieldPath> in result
        let p= [&self.aux_params, &query.aux_params];
        let aux_params = Parameters::new(&p);

        for token in &query.tokens{

            match token {
                QueryToken::Field(field) => {
                    
                    if self.on_path(field) {
                        let basename= field.basename();
                        let sql_target =  mapper.fields.get(basename)
                            .ok_or(SqlBuilderError::FieldMissing(basename.to_string()))?;

                        // Continue if field is not filtered
                        if field.filter.is_none() {
                            continue;
                        }

                        if let Some(role) = self.missing_role( &sql_target.options.roles/*mapper.load_roles(field)*/) {
                                return Err (SqlBuilderError::RoleRequired(role.to_string()).into());
                        }
                        let canonical_alias = field.canonical_alias(&self.root_mapper);
                        let alias = alias_translator.translate(&canonical_alias);
                        let sql = sql_target.expression.resolve(&alias, None, &aux_params)?;

                        let select_sql = sql_target
                                            .handler
                                            .build_select(
                                                sql,
                                                &aux_params,
                                            )?
                                            .unwrap_or(("NULL".to_string(), vec![]));

                        // Does filter apply
                        if let Some(sql) = sql_target.handler
                                                .build_filter(select_sql, field.filter.as_ref().unwrap(), &aux_params)?
                        {
                            result.where_clause.push_str(&sql.0);
                            result.where_params.extend_from_slice(&sql.1);
                        }
                    }
                },
                QueryToken::Predicate(predicate) => {

                },
                _ => {}
            }

        }
        Ok(())

    }

    fn missing_role(&self, roles: &HashSet<String>) ->Option<&str>{

        None
    }

    fn on_path(&self, field: &Field) -> bool{

        false

    }

    fn canonical_alias<'b> (build_context: &BuildContext,  field_path: &'b str)-> &'b str {
        field_path.trim_start_matches(&build_context.root_path)

    }
}