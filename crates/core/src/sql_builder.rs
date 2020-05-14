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
pub mod construct;
pub mod eval_query;
pub mod sql_builder_error;
pub mod build_result;
pub mod build_context;
pub mod sql_target_data;
pub mod wildcard_scope;



use crate::sql_builder::eval_query::eval_query;
use crate::sql_builder::construct::build_join_clause;
use crate::sql_builder::construct::combine_aux_params;
use crate::sql_builder::construct::build_count_select_clause;
use crate::sql_builder::construct::build_select_clause;
use crate::sql_builder::construct::build_ordering;
use crate::sql_builder::sql_target_data::SqlTargetData;
use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::error::ToqlError;
use crate::query::assert_roles;
use crate::query::concatenation::Concatenation;
use crate::query::field_order::FieldOrder;
use crate::query::Query;
use crate::query::{field_filter::FieldFilter, QueryToken};
use build_result::BuildResult;
use crate::sql_mapper::Join;
use crate::sql_mapper::JoinType;
use crate::sql_mapper::SqlMapper;
use crate::sql_mapper::SqlTarget;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

use crate::sql::{Sql, SqlArg};
use wildcard_scope::WildcardScope;

/// The Sql builder to build normal queries and count queries.
pub struct SqlBuilder<'a> {
   // count_query: bool,               // Build count query
    subpath: String,                 // Build only subpath
    joins: HashSet<String>,          // Use this joins
    ignored_paths: Vec<String>,      // Ignore paths, no errors are raised for them
    selected_paths: HashSet<String>, // Selected paths
    wildcard_scope: WildcardScope,   // Wildcard restriction
    aux_params: &'a HashMap<String, SqlArg> // Aux params used for all queries with this builder instance, contains typically config or auth data
}

#[derive(PartialEq)]
pub enum BuildMode {
    SelectQuery,          // Select Que
    CountFiltered,
    DeleteQuery,
    SelectMut,
    SelectAll
}

impl<'a> SqlBuilder<'a> {
    /// Create a new SQL Builder
    pub fn new(aux_params: &'a HashMap<String, SqlArg>) -> Self {
       
       SqlBuilder {
           // count_query: false,
            subpath: "".to_string(),
            joins: HashSet::new(),
            ignored_paths: Vec::new(),
            selected_paths: HashSet::new(),
            wildcard_scope: WildcardScope::All,
            aux_params,
        }
    }
   

    /// Add wildcard scope to the wildcard scopes
    pub fn scope_wildcard(mut self, scope: &WildcardScope) -> Self {
        match (&self.wildcard_scope, scope) {
            (WildcardScope::All, WildcardScope::Only(_)) => self.wildcard_scope = scope.to_owned(),
            (WildcardScope::Only(_), WildcardScope::All) => {
                self.wildcard_scope = WildcardScope::All
            }
            (WildcardScope::Only(old), WildcardScope::Only(new)) => {
                let mut combined = HashSet::new();
                old.iter().for_each(|n| {
                    combined.insert(n.to_owned());
                });
                new.iter().for_each(|n| {
                    combined.insert(n.to_owned());
                });
                self.wildcard_scope = WildcardScope::Only(combined)
            }
            (WildcardScope::All, WildcardScope::All) => {}
        };
        self
    }

    /// Add path to list of ignore paths.
    pub fn ignore_path<T: Into<String>>(mut self, path: T) -> Self {
        self.ignored_paths.push(path.into());
        self
    }
    /// Add path to list of selected paths.
    pub fn select_path<T: Into<String>>(mut self, path: T) -> Self {
        self.selected_paths.insert(path.into());
        self
    }
    /// TODO
    pub fn with_join<T: Into<String>>(mut self, join: T) -> Self {
        self.joins.insert(join.into());
        self
    }


    


    /// Build query for total count.
    pub fn build_query_sql<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
        modifier : &str,
        extra : &str,
    ) -> Result<Sql, ToqlError> {
       
        let result = self.build(sql_mapper, query, roles, BuildMode::SelectQuery)?;
        Ok((result.query_stmt(modifier, extra), result.combined_params))
    }

    /// Build query to count all records that match the query predicate.
    /// Thi uses only fields marked as count filter.
    pub fn build_filtered_count_sql<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
    ) -> Result<Sql, ToqlError> {
       
        let result = self.build(sql_mapper, query, roles, BuildMode::CountFiltered)?;
        Ok((result.count_stmt(), result.combined_params))
    }

    /// Build query that counts the rows instead of selecting them.
    pub fn build_count_sql<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
    ) -> Result<Sql, ToqlError> {
       
        let result = self.build(sql_mapper, query, roles, BuildMode::SelectQuery)?; // Normal select (This can be optimised with another mode )
        Ok((result.count_stmt(), result.combined_params)) // Count statement
    }

    pub fn build_query_and_count_sql<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
        query_modifier : &str,
        query_extra : &str,
       
    ) -> Result<(Sql, Sql), ToqlError> {
       
        let result = self.build(sql_mapper, query, roles, BuildMode::SelectQuery)?;
        Ok((
            (result.query_stmt(query_modifier, query_extra), result.combined_params.clone()),
            (result.count_stmt(), result.combined_params)
        ))
    }

     /// Build query for total count.
    pub fn build_delete_sql<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
    ) -> Result<Sql, ToqlError> {
       
        let result = self.build(sql_mapper, query, roles, BuildMode::DeleteQuery)?;
        Ok((result.delete_stmt(), result.combined_params))
    }
     /// Build query for total count.
    pub fn build_select_all_sql<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
        modifier : &str,
        extra : &str,
    ) -> Result<Sql, ToqlError> {
       
        let result = self.build(sql_mapper, query, roles, BuildMode::SelectAll)?;
        Ok((result.query_stmt(modifier, extra), result.combined_params))
    }

    /// Build query for a given path.
    /// Return none of no column is selected
    pub fn build_query_sql_for_path<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
        modifier : &str,
        extra : &str,
        path : &str
    ) -> Result<Option<Sql>, ToqlError> {
       
        let result = self.build_path(path, sql_mapper, query, roles, BuildMode::SelectQuery)?;

        Ok( match result.any_selected() {
            true => Some((result.query_stmt(modifier, extra), result.combined_params)),
            false => None
        })
    }
    /// Build query for a given path.
    /// Return none of no column is selected
    pub fn build_query_sql_for_path_with_additional_columns<M, S>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
        modifier : &str,
        extra : &str,
        path : &str,
        columns: &[S]
    ) -> Result<Option<Sql>, ToqlError>
     where S: AsRef<str>
     {
        let mut result = self.build_path(path, sql_mapper, query, roles, BuildMode::SelectQuery)?;

        Ok( match result.any_selected() {

            true => {
                columns.iter().for_each(|s|result.push_select(s.as_ref()));
                Some((result.query_stmt(modifier, extra), result.combined_params))},
            false => None
        })
    }

  /// Build query for total count.
    pub fn build_query_sql_with_additional_columns<M, S> (
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
        modifier : &str,
        extra : &str,
        selects: &[S]
    ) -> Result<Sql, ToqlError> 
     where S: AsRef<str>
    {
       
        let mut result = self.build(sql_mapper, query, roles, BuildMode::SelectQuery)?;
        selects.iter().for_each(|s|result.push_select(s.as_ref()));

        Ok((result.query_stmt(modifier, extra), result.combined_params))
    }

     /// Build query for total count.
    pub fn build_select_mut_sql<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
        modifier : &str,
        extra : &str,
    ) -> Result<Sql, ToqlError> {
       
       /*  let mut sql_target_data: HashMap<&str, SqlTargetData> = HashMap::new();
        let mut selected_paths: HashSet<String> = HashSet::new();

        let mut result = BuildResult::new();
        result.aliased_table =  sql_mapper.aliased_table.clone();
        
       for field_name in &sql_mapper.mut_fields {
                let f = sql_target_data.entry(field_name.as_str()).or_default();
                f.selected = true; // Select field
                f.used = true;      // Used field because selected
                Self::insert_paths(&field_name, &mut selected_paths);
       }

        let aux_params = HashMap::new();
        Self::build_join_clause(
            &sql_mapper.joins_root,
            &sql_mapper.joins_tree,
            &mut selected_paths,
            &sql_mapper.joins,
            &aux_params,
            &mut result,
        )?;

         Self::build_select_clause(
                &mut result,
                &aux_params,
                &sql_mapper.fields,
                &sql_target_data,
                &sql_mapper.field_order,
                &selected_paths,
            )?;
 */


        let result = self.build(sql_mapper, query, roles, BuildMode::SelectMut)?;
        Ok((result.query_stmt(modifier, extra), result.combined_params))
    }

    /// Build normal query for this path
    pub fn build_path<M, T: Into<String>>(
        &mut self,
        path: T,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,mode: BuildMode
    ) -> Result<BuildResult, SqlBuilderError> {
        self.subpath = {
            let p = path.into();
            if p.ends_with("_") {
                p
            } else {
                format!("{}_", p)
            }
        };
        self.build(sql_mapper, query, roles, mode)
    }

    

    /// Build normal query.
    pub fn build<M>(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query<M>,
        roles: &HashSet<String>,
        mode: BuildMode
    ) -> Result<BuildResult, SqlBuilderError> {
        let mut ordinals: HashSet<u8> = HashSet::new();
        let mut ordering: HashMap<u8, Vec<(FieldOrder, String)>> = HashMap::new();

      

        let mut sql_target_data: HashMap<String, SqlTargetData> = HashMap::new();
        let mut selected_paths: HashSet<String> = HashSet::new();

        let mut on_params : HashMap<String, SqlArg> = HashMap::new(); // 

        // combine aux params from query and SqlBuilder instance
        let mut build_aux_params : HashMap<String, SqlArg> = HashMap::new();
        combine_aux_params(
                                        &mut build_aux_params,
                                        &query.aux_params,
                                        &self.aux_params,
                                    );

        let mut result = BuildResult::new(&sql_mapper.aliased_table);
        //result.aliased_table =  sql_mapper.aliased_table.clone();
        result.distinct =  query.distinct;
            
        eval_query(
            &build_aux_params,
            &mut on_params,
            roles, 
            &self.wildcard_scope,
            &sql_mapper, 
            &self.ignored_paths, 
            &self.subpath, 
            &mode, 
            &query,
            &mut sql_target_data, 
            &mut selected_paths, 
            &mut ordinals,
            &mut ordering,
            &mut result 
                )?;

        // Select all fields for count queries that are marked with count_select
        if mode == BuildMode::CountFiltered {
            for (field_name, mapper_field) in &sql_mapper.fields {
                if mapper_field.options.count_select {
                    let f = sql_target_data.entry(field_name.to_string()).or_default();
                    f.selected = true;
                    f.used= true;
                }
            }
        }
        // Select all fields for count queries that are marked with count_select
        else if mode == BuildMode::SelectMut {
            for field_name in &sql_mapper.mut_fields {
               
                    let f = sql_target_data.entry(field_name.to_string()).or_default();
                    f.selected = true;
                    f.used = true;

                    // Add path for join
                     let mut path = field_name
                    .trim_end_matches(|c| c != '_')
                    .trim_end_matches('_');
                    while !path.is_empty() {
                        let exists = !selected_paths.insert(path.to_owned());
                        if exists {
                            break;
                        }
                        path =
                            path.trim_end_matches(|c| c != '_').trim_end_matches('_');
                    }
            }
        }

        
        let mut combined_on_params: HashMap<String, SqlArg>= HashMap::new();
        let on_params = if on_params.is_empty() {
            &build_aux_params
        } else {
            combine_aux_params(
                                        &mut combined_on_params,
                                        &build_aux_params,
                                        &on_params,
                                    )
        };
        // println!("Selected joins from query {:?}", selected_paths);
        build_join_clause(
            &sql_mapper.joins_root,
            &sql_mapper.joins_tree,
            &mut selected_paths,
            &sql_mapper.joins,
            &on_params,
            &mut result,
        )?;

      
        // Add auxiliary joins
        result.join_clause.push_str(&query.join_stmts.join(" "));
        result
            .join_params
            .extend_from_slice(&query.join_stmt_params);

        //println!("Selected joins including inner joins {:?}", selected_paths);

        if mode == BuildMode::CountFiltered {
            build_count_select_clause(
                &mut result,
                &build_aux_params,
                &sql_mapper.fields,
                &sql_mapper.field_order,
            )?;
        } else {
            build_ordering(
                &mut result,
                &build_aux_params,
                &sql_target_data,
                &sql_mapper.fields,
                &ordinals,
                &ordering,
            )?;
            build_select_clause(
                &mut result,
                &build_aux_params,
                &sql_mapper.fields,
                &sql_target_data,
                &sql_mapper.field_order,
               
                &selected_paths,
           )?;
        }

        // Remove trailing whitespace on JOIN and ORDER clause
        if result
            .join_clause
            .chars()
            .rev()
            .next()
            .unwrap_or(' ')
            .is_whitespace()
        {
            result.join_clause = result.join_clause.trim_end().to_owned();
        }
        if result
            .order_clause
            .chars()
            .rev()
            .next()
            .unwrap_or('_')
            .is_whitespace()
        {
            result.order_clause = result.order_clause.trim_end().to_owned();
        }

        // Add additional where predicates if provided
        if query.where_predicates.len() > 0 {
            let concatenate = !result.where_clause.is_empty();
            if concatenate {
                result.where_clause.push_str(" AND (");
            }
            result
                .where_clause
                .push_str(&query.where_predicates.join(" AND "));
            if concatenate {
                result.where_clause.push(')');
            }
            result
                .where_params
                .extend_from_slice(&query.where_predicate_params);
        }

        result
            .combined_params
            .extend_from_slice(&result.select_params);
        result
            .combined_params
            .extend_from_slice(&result.join_params);
        result
            .combined_params
            .extend_from_slice(&result.where_params);
        result
            .combined_params
            .extend_from_slice(&result.having_params);
        result
            .combined_params
            .extend_from_slice(&result.order_params);

        Ok(result)
    }
   
}
