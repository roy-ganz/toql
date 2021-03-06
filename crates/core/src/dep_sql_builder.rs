//!
//! The SQL Builder turns a [Query](../query/struct.Query.html) with the help of a [SQL Mapper](../sql_mapper/struct.SqlMapper.html)
//! into a [SQL Builder Result](../sql_builder_result/SqlBuilderResult.html)
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
use crate::error::ToqlError;
use crate::query::assert_roles;
use crate::query::Concatenation;
use crate::query::FieldOrder;
use crate::query::Query;
use crate::query::{FieldFilter, QueryToken};
use crate::sql_builder_result::SqlBuilderResult;
use crate::sql_mapper::Join;
use crate::sql_mapper::JoinType;
use crate::sql_mapper::SqlMapper;
use crate::sql_mapper::SqlTarget;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

use crate::sql::{Sql, SqlArg};

struct SqlTargetData {
    selected: bool, // Target is selected
    used: bool,     // Target is either selected or filtered
}

impl Default for SqlTargetData {
    fn default() -> SqlTargetData {
        SqlTargetData {
            used: false,
            selected: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WildcardScope {
    All,
    Only(HashSet<String>),
}

impl WildcardScope {
    pub fn contains_field(&self, field: &str) -> bool {
        match self {
            WildcardScope::All => true,
            WildcardScope::Only(scopes) => scopes.contains(field),
        }
    }
    pub fn contains_all_fields_from_path(&self, path: &str) -> bool {
        match self {
            WildcardScope::All => true,
            WildcardScope::Only(scopes) => {
                let mut field = String::from(path);
                if !path.is_empty() && !path.ends_with('_') {
                    field.push('_');
                }
                field.push_str("*");
                scopes.contains(field.as_str())
            }
        }
    }
    pub fn contains(&self, field_with_path: &str) -> bool {
        match self {
            WildcardScope::All => true,
            WildcardScope::Only(scopes) => {
                scopes.contains(field_with_path)
                    || if !field_with_path.ends_with("*") {
                        // If field is provided check for all fields
                        let mut path = field_with_path.trim_end_matches(|c| c != '_').to_string();
                        path.push_str("*");
                        scopes.contains(path.as_str())
                    } else {
                        false
                    }
            }
        }
    }
    pub fn contains_path(&self, path: &str) -> bool {
        let path = path.trim_end_matches("_"); // Remove optional trailing underscore
        match self {
            WildcardScope::All => true,
            WildcardScope::Only(scopes) => {
                let mut wildcard_path = path.to_owned();
                wildcard_path.push_str("_*");
                // Check if path with wildcard exists or any field with that path
                scopes.contains(wildcard_path.as_str())
                    || scopes
                        .iter()
                        .any(|s| s.trim_end_matches(|c| c != '_').trim_end_matches("_") == path)
            }
        }
    }
}

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

#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum SqlBuilderError {
    /// The field is not mapped to a column or SQL expression. Contains the field name.
    FieldMissing(String),
     /// The field is not mapped to a column or SQL expression. Contains the field name.
    PredicateMissing(String),
    /// The field requires a role that the query does not have. Contains the role.
    RoleRequired(String),
    /// The filter expects other arguments. Typically raised by custom functions (FN) if the number of arguments is wrong.
    FilterInvalid(String),
    /// A query expression requires a query parameter, that is not provided. Contains the parameter.
    QueryParamMissing(String),
    /// The query parameter that is required by the query expression is wrong. Contains the parameter and the details.
    QueryParamInvalid(String, String),
    /// A predicate requires more arguments, than the toql query provided, contains the predicate.
    PredicateArgumentMissing(String),
}

impl fmt::Display for SqlBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlBuilderError::FieldMissing(ref s) => write!(f, "field `{}` is missing", s),
            SqlBuilderError::PredicateMissing(ref s) => write!(f, "predicate `@{}` is missing", s),
            SqlBuilderError::RoleRequired(ref s) => write!(f, "role `{}` is required", s),
            SqlBuilderError::FilterInvalid(ref s) => write!(f, "filter `{}` is invalid ", s),
            SqlBuilderError::QueryParamMissing(ref s) => {
                write!(f, "query parameter `{}` is missing ", s)
            }
            SqlBuilderError::QueryParamInvalid(ref s, ref d) => {
                write!(f, "query parameter `{}` is invalid: {} ", s, d)
            },
            SqlBuilderError::PredicateArgumentMissing(ref s) => {
                write!(f, "predicate `{}` requires more arguments. ", s)
            }
        }
    }
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

        let mut result = SqlBuilderResult::new();
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
    ) -> Result<SqlBuilderResult, SqlBuilderError> {
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
    ) -> Result<SqlBuilderResult, SqlBuilderError> {
        let mut ordinals: HashSet<u8> = HashSet::new();
        let mut ordering: HashMap<u8, Vec<(FieldOrder, String)>> = HashMap::new();

        let mut need_where_concatenation = false;
        let mut need_having_concatenation = false;
        let mut pending_where_parens_concatenation: Option<Concatenation> = None;
        let mut pending_having_parens_concatenation: Option<Concatenation> = None;
        let mut pending_where_parens: u8 = 0;
        let mut pending_having_parens: u8 = 0;

        let mut sql_target_data: HashMap<&str, SqlTargetData> = HashMap::new();
        let mut selected_paths: HashSet<String> = HashSet::new();

        let mut on_params : HashMap<String, SqlArg> = HashMap::new(); // 

        // aux params from query and SqlBuilder instance
        let mut build_aux_params : HashMap<String, SqlArg> = HashMap::new();
        Self::combine_aux_params(
                                        &mut build_aux_params,
                                        &query.aux_params,
                                        &self.aux_params,
                                    );

        let mut result = SqlBuilderResult::new();
        result.aliased_table =  sql_mapper.aliased_table.clone();
        result.distinct =  query.distinct;
            

        for t in &query.tokens {
            {
                match t {
                    QueryToken::LeftBracket(ref concatenation) => {
                        pending_where_parens += 1;
                        pending_having_parens += 1;
                        pending_having_parens_concatenation = Some(concatenation.clone());
                        pending_where_parens_concatenation = Some(concatenation.clone());
                    }
                    QueryToken::RightBracket => {
                        if pending_where_parens > 0 {
                            pending_where_parens -= 1;
                        } else {
                            result.where_clause.push_str(")");
                            need_where_concatenation = true;
                        }
                        if pending_having_parens > 0 {
                            pending_having_parens -= 1;
                        } else {
                            result.having_clause.push_str(")");
                            need_having_concatenation = true;
                        }
                    }

                    QueryToken::Wildcard(wildcard) => {
                        // Skip wildcard for count queries
                      /*   if self.count_query {
                            continue;
                        } */
                        // Wildcard is only evaluated for nomral queries
                        if mode != BuildMode::SelectQuery {
                            continue;
                        }
                        // Skip field from other path

                        if !(wildcard.path.starts_with(&self.subpath)
                            || self.subpath.starts_with(&wildcard.path))
                        {
                            continue;
                        }

                        let wildcard_path = wildcard
                            .path
                            .trim_start_matches(&self.subpath)
                            .trim_end_matches('_');

                        // Skip ignored path
                        if self
                            .ignored_paths
                            .iter()
                            .any(|p| wildcard_path.starts_with(p))
                        {
                            continue;
                        }
                        // Ensure user has load roles for path
                        let mut path = wildcard_path;
                        while !path.is_empty() {
                            if let Some(join) = sql_mapper.joins.get(path) {
                                assert_roles(&roles, &join.options.roles)
                                    .map_err(|role| SqlBuilderError::RoleRequired(role))?;
                            } else {
                                return Err(SqlBuilderError::FieldMissing(path.to_owned()));
                            }
                            path = path.trim_end_matches(|c| c != '_').trim_end_matches('_');
                        }

                        // Cache vars to speed up validation
                        let mut last_validated_path = ("", true); 
                        let mut last_validated_scope_wildcard = ("", "", false);

                        for (field_name, sql_target) in &sql_mapper.fields {
                            
                            if !sql_target.options.query_select {
                                continue;
                            }


                            let field_path = field_name
                                .trim_end_matches(|c| c != '_')
                                .trim_end_matches('_');

                            // Check if field is in wildcard scope
                            let wildcard_in_scope = if last_validated_scope_wildcard
                                == (&self.subpath, &field_path, true)
                            {
                                true
                            } else {
                                // check path
                                let mut temp_scope: String;
                                let path_for_scope_test = if self.subpath.is_empty() {
                                    field_path
                                } else {
                                    temp_scope = self.subpath.clone();
                                    if !temp_scope.ends_with('_') {
                                        temp_scope.push('_');
                                    }
                                    temp_scope.push_str(field_path);
                                    temp_scope.as_str()
                                };

                                if self
                                    .wildcard_scope
                                    .contains_all_fields_from_path(path_for_scope_test)
                                {
                                    last_validated_scope_wildcard =
                                        (&self.subpath, &field_path, true);
                                    true
                                } else {
                                    let field_for_scope_test = if self.subpath.is_empty() {
                                        field_name
                                    } else {
                                        temp_scope = self.subpath.clone();
                                        if !temp_scope.ends_with('_') {
                                            temp_scope.push('_');
                                        }
                                        temp_scope.push_str(field_name);
                                        temp_scope.as_str()
                                    };
                                    //println!("Test {} in {:?}", field_for_scope_test, self.wildcard_scope);
                                    self.wildcard_scope.contains_field(&field_for_scope_test)
                                }
                            };

                            if !wildcard_in_scope {
                               //    println!("Skipped {:?}", field_name);
                                continue;
                            } else {
                               // println!("Included {:?}", field_name);
                            }

                            // Skip field if it doesn't belong to wildcard path
                            if !field_path.starts_with(wildcard_path) {
                                continue;
                            }

                            // Skip ignored paths, they belong typically to merged fields and are handled by another build() call
                            if self.ignored_paths.iter().any(|p| field_name.starts_with(p)) {
                                continue;
                            }

                            if sql_target.options.skip_wildcard {
                                continue;
                            }

                            // Skip fields with missing role
                            if assert_roles(&roles, &sql_target.options.roles).is_err() {
                                continue;
                            }

                            // Skip field paths, that are marked with ignore wildcard or have missing roles
                            if !field_path.is_empty() {
                                if field_path != last_validated_path.0 {
                                    let mut path = field_path;
                                    // Remember successful validation to speed up next validation of the same path
                                    last_validated_path.0 = field_path;
                                    while !path.is_empty() {
                                        // Validate path only up to wildcard path
                                        if path == wildcard_path {
                                            last_validated_path = (path, true);
                                            break;
                                        }

                                        //if ignore wildcard, roles missing validated_path = (path, false)
                                        if let Some(join) = sql_mapper.joins.get(path) {
                                            if join.options.skip_wildcard {
                                                last_validated_path = (path, false);
                                                break;
                                            }
                                        }

                                        //println!("PATH={}", path);
                                        // Next path
                                        path = path
                                            .trim_end_matches(|c| c != '_')
                                            .trim_end_matches('_');
                                    }
                                }

                                // Skip any field on path with failed validation
                                if last_validated_path.1 != true {
                                    // println!("Path {} is invalid!", last_validated_path.0);
                                    continue;
                                }
                                /*  else {
                                    //println!("Path {} is valid!", last_validated_path.0);
                                } */
                            }

                            // Select all fields on wildcard path
                            // including joins with preselected fields only

                            //let select = (field_path == wildcard_path) || field_path.starts_with(wildcard_path) && sql_target.options.preselect;

                            // let select = field_path.starts_with(wildcard_path);
                            //println!( "field {}: field_path={}, wildcard_path ={}, select={}",&field_name, field_path, &wildcard.path, select);

                            /* if (wildcard.path.is_empty())  && ! sql_target.subfields
                            || (field_name.starts_with(&wildcard.path)
                                && field_name.rfind("_").unwrap_or(field_name.len())
                                    < wildcard.path.len()) */
                            /*   if select
                            { */
                            let f = sql_target_data.entry(field_name.as_str()).or_default();

                            f.selected = true; // Select field
                            f.used = true;      // Used field because selected

                            // Ensure all parent paths are selected
                            if sql_target.subfields {
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

                                /* for subfield in field_name.split('_').rev().skip(1) {
                                     let exists= selected_paths.insert(subfield);
                                if exists { break;}
                                } */
                            }
                            // }
                        }
                    }
                    QueryToken::Field(query_field) => {
                        // Ignore field if name does not start with path
                        // E.g "user_id" has path "user"
                        if !self.subpath.is_empty() && !query_field.name.starts_with(&self.subpath)
                        {
                            continue;
                        }
                        if self
                            .ignored_paths
                            .iter()
                            .any(|p| query_field.name.starts_with(p))
                        {
                            continue;
                        }

                        let field_name = if self.subpath.is_empty() {
                            &query_field.name
                        } else {
                            query_field
                                .name
                                .trim_start_matches(&self.subpath)
                                .trim_start_matches('_')
                        };

                        match sql_mapper.fields.get(field_name) {
                            Some(sql_target) => {
                                // Verify user role and skip field role mismatches
                                assert_roles(&roles, &sql_target.options.roles)
                                    .map_err(|role| SqlBuilderError::RoleRequired(role))?;

                                // Skip filtering and ordering in count queries for unfiltered fields
                                if mode == BuildMode::CountFiltered && !sql_target.options.count_filter {
                                    continue;
                                }

                                // Skip field that cannot neither be selected and filtered
                                if !sql_target.options.query_select {
                                    continue;
                                }

                                /* if self.count_query == true && !sql_target.options.count_filter {
                                    continue;
                                } */

                                // Select sql target if field is not hidden
                                let data = sql_target_data.entry(field_name).or_default();

                                // Add parent Joins
                                if sql_target.subfields {
                                    Self::insert_paths(field_name, &mut selected_paths);
                                    /* let mut path = field_name
                                        .trim_end_matches(|c| c != '_')
                                        .trim_end_matches('_');
                                    while !path.is_empty() {
                                        let exists = !selected_paths.insert(path.to_owned());
                                        if exists {
                                            break;
                                        }
                                        path = path
                                            .trim_end_matches(|c| c != '_')
                                            .trim_end_matches('_');
                                    } */
                                }
                                //println!("{:?}", selected_paths);

                                data.selected = match mode {
                                    BuildMode::CountFiltered => sql_target.options.count_select,
                                    BuildMode::DeleteQuery => false,
                                    BuildMode::SelectAll => true,
                                    BuildMode::SelectMut => sql_target.options.mut_select,
                                    BuildMode::SelectQuery => !query_field.hidden
                                };
                              /*   if self.count_query {
                                    sql_target.options.count_select
                                } else {
                                    !query_field.hidden
                                }; */

                                // Target is used if it's selected
                                data.used = data.selected;  

                               // data.used = !query_field.hidden;

                                // TODO fix bug
                                // Resolve query params in sql expression
                                /* for p in &sql_target.sql_query_params {
                                    let qp = query
                                        .params
                                        .get(p)
                                        .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;

                                    if query_field.aggregation == true {
                                        result.having_params.push(qp.to_string());
                                    } else {
                                        result.where_params.push(qp.to_string());
                                    }
                                } */

                                if let Some(f) = &query_field.filter {
                                    // Combine aux params from query and target
                                    let mut combined_aux_params: HashMap<String, SqlArg> =
                                        HashMap::new();
                                    let aux_params = Self::combine_aux_params(
                                        &mut combined_aux_params,
                                        &build_aux_params,
                                        &sql_target.options.aux_params,
                                    );

                                    let aux_param_values = Self::aux_param_values(&sql_target.sql_aux_param_names, aux_params)?;
                                        

                                    let expression = sql_target
                                        .handler
                                        .build_select(
                                            (sql_target.expression.to_owned(), aux_param_values),
                                            aux_params,
                                        )?
                                        .unwrap_or(("null".to_string(), vec![]));

                                    if let Some(f) = sql_target.handler.build_filter(
                                        expression, 
                                        &f, aux_params
                                    )? {

                                        // Valid filter on target makes it used
                                         data.used = data.used | true;

                                        if query_field.aggregation == true {
                                            if need_having_concatenation == true {
                                                if pending_having_parens > 0 {
                                                    SqlBuilderResult::push_concatenation(
                                                        &mut result.having_clause,
                                                        &pending_having_parens_concatenation,
                                                    );
                                                } else {
                                                    SqlBuilderResult::push_concatenation(
                                                        &mut result.having_clause,
                                                        &Some(query_field.concatenation.clone()), // OPTIMISE
                                                    );
                                                }
                                            }

                                            SqlBuilderResult::push_pending_parens(
                                                &mut result.having_clause,
                                                &pending_having_parens,
                                            );

                                            SqlBuilderResult::push_filter(
                                                &mut result.having_clause,
                                                &f.0,
                                            );
                                            if query_field.aggregation == true {
                                                result.having_params.extend_from_slice(&f.1);
                                            } else {
                                                result.where_params.extend_from_slice(&f.1);
                                            }

                                            need_having_concatenation = true;
                                            pending_having_parens = 0;
                                        } else {
                                            if need_where_concatenation == true {
                                                if pending_where_parens > 0 {
                                                    SqlBuilderResult::push_concatenation(
                                                        &mut result.where_clause,
                                                        &pending_where_parens_concatenation,
                                                    );
                                                } else {
                                                    SqlBuilderResult::push_concatenation(
                                                        &mut result.where_clause,
                                                        &Some(query_field.concatenation.clone()), // IMPROVE
                                                    );
                                                }
                                            }
                                            SqlBuilderResult::push_pending_parens(
                                                &mut result.where_clause,
                                                &pending_where_parens,
                                            );
                                            SqlBuilderResult::push_filter(
                                                &mut result.where_clause,
                                                &f.0,
                                            );
                                            if query_field.aggregation == true {
                                                result.having_params.extend_from_slice(&f.1);
                                            } else {
                                                result.where_params.extend_from_slice(&f.1);
                                            }

                                            pending_where_parens = 0;
                                            need_where_concatenation = true;
                                        }
                                    }

                                // Add Parameters for on clauses
                                if let FieldFilter::Eq(a) | FieldFilter::Ne(a) = f {
                                    for n in &sql_target.options.on_params {
                                        on_params.insert(n.to_string(), a.to_owned());
                                    }
                                }
                                
                                    // TODO Test correct aux_params provided
                                   // Sql Target aux params, join aux params?
                                   /*  if let Some((j, p)) =
                                        sql_target.handler.build_join(aux_params)?
                                    {
                                        result.join_clause.push_str(&j);
                                        result.join_clause.push_str(" ");
                                        result.join_params.extend_from_slice(&p);
                                    }  */
                                }
                                if let Some(o) = &query_field.order {
                                    let num = match o {
                                        FieldOrder::Asc(num) => num,
                                        FieldOrder::Desc(num) => num,
                                    };
                                    ordinals.insert(*num);
                                    let l = ordering.entry(*num).or_insert(Vec::new());
                                    l.push((o.clone(), query_field.name.clone()));
                                    // OPTIMISE
                                }
                            }
                            None => {
                                // If field has path, validate to known paths
                                if !query_field.name.contains("_")
                                    || !self.path_ignored(&query_field.name)
                                {
                                    return Err(SqlBuilderError::FieldMissing(
                                        query_field.name.clone(),
                                    ));
                                }
                            }
                        }
                    },
                     QueryToken::Predicate(query_predicate) => {
                         
                         // Predicates work only on base entity
                        if !self.subpath.is_empty()
                        {
                            continue;
                        }
                          

                         match sql_mapper.predicates.get(&query_predicate.name) {
                            Some(predicate) => {

                                if mode == BuildMode::CountFiltered && !predicate.options.count_filter {
                                    continue;
                                }

                                fn predicate_param_values(aux_param_names: &Vec<String>, aux_params: &HashMap<String, SqlArg>, predicate_args: &Vec<SqlArg>, predicate_name:&str ) -> Result<Vec<SqlArg>, SqlBuilderError>{
                                      let mut params: Vec<SqlArg> = Vec::with_capacity(aux_param_names.len());
                                      let mut i = 0usize;
                                    for p in aux_param_names {
                                       let value =  if p == "?" {
                                            (predicate_args.get(i).ok_or(SqlBuilderError::PredicateArgumentMissing(predicate_name.to_string())),
                                            i = i + 1).0
                                        } else {
                                        aux_params
                                            .get(p)
                                            .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))
                                        }?;
                                        params.push(value.to_owned());
                                    };
                                    Ok(params)
                                }

                                
                                
                                 let mut combined_aux_params: HashMap<String, SqlArg> =
                                        HashMap::new();
                                    let aux_params = Self::combine_aux_params(
                                        &mut combined_aux_params,
                                        &build_aux_params,
                                        &predicate.options.aux_params,
                                    );

                                let args = predicate_param_values(&predicate.sql_aux_param_names, &aux_params,  &query_predicate.args,&query_predicate.name )?;
                                if let Some((expr, args)) = predicate.handler.build_predicate(( predicate.expression.to_owned(), args),&query_predicate.args, aux_params)? {

                                   if need_where_concatenation == true {
                                        if pending_where_parens > 0 {
                                            SqlBuilderResult::push_concatenation(
                                                &mut result.where_clause,
                                                &pending_where_parens_concatenation,
                                            );
                                        } else {
                                            SqlBuilderResult::push_concatenation(
                                                &mut result.where_clause,
                                                &Some(query_predicate.concatenation.clone()), // IMPROVE
                                            );
                                        }
                                    }
                                    SqlBuilderResult::push_pending_parens(
                                        &mut result.where_clause,
                                        &pending_where_parens,
                                    );
                                    SqlBuilderResult::push_filter(
                                        &mut result.where_clause,
                                        &expr,
                                    );
                                   
                                    result.where_params.extend_from_slice(&args);
                                    
                                    pending_where_parens = 0;
                                    need_where_concatenation = true;
                                }

                                // Add On Parameters for on clauses 
                                for (i,n) in &predicate.options.on_params {
                                    let a = query_predicate.args.get(*i as usize).ok_or(SqlBuilderError::PredicateArgumentMissing(query_predicate.name.to_string()))?;
                                    on_params.insert(n.to_string(), a.to_owned());
                                }

                                
                            


                            },
                            None => {
                                 return Err(SqlBuilderError::PredicateMissing(
                                        query_predicate.name.clone(),
                                    ));
                            }
                         }
                     }
                }
            }
        }

        // Select all fields for count queries that are marked with count_select
        if mode == BuildMode::CountFiltered {
            for (field_name, mapper_field) in &sql_mapper.fields {
                if mapper_field.options.count_select {
                    let f = sql_target_data.entry(field_name.as_str()).or_default();
                    f.selected = true;
                    f.used= true;
                }
            }
        }
        // Select all fields for count queries that are marked with count_select
        else if mode == BuildMode::SelectMut {
            for (field_name, mapper_field) in &sql_mapper.fields {
                if mapper_field.options.mut_select {
                    let f = sql_target_data.entry(field_name.as_str()).or_default();
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
        }

        
        let mut combined_on_params: HashMap<String, SqlArg>= HashMap::new();
        let on_params = if on_params.is_empty() {
            &build_aux_params
        } else {
            Self::combine_aux_params(
                                        &mut combined_on_params,
                                        &build_aux_params,
                                        &on_params,
                                    )
        };
        // println!("Selected joins from query {:?}", selected_paths);
        Self::build_join_clause(
            &sql_mapper.joins_root,
            &sql_mapper.joins_tree,
            &mut selected_paths,
            &sql_mapper.joins,
            &on_params,
            &mut result,
        )?;

        // Add Auxiliary joins
        // Add auxiliary joins
        result.join_clause.push_str(&query.join_stmts.join(" "));
        result
            .join_params
            .extend_from_slice(&query.join_stmt_params);

        //println!("Selected joins including inner joins {:?}", selected_paths);

        if mode == BuildMode::CountFiltered {
            Self::build_count_select_clause(
                &mut result,
                &build_aux_params,
                &sql_mapper.fields,
                &sql_mapper.field_order,
            )?;
        } else {
            Self::build_ordering(
                &mut result,
                &build_aux_params,
                &sql_target_data,
                &sql_mapper.fields,
                &ordinals,
                &ordering,
            )?;
            Self::build_select_clause(
                &mut result,
                &build_aux_params,
                &sql_mapper.fields,
                &sql_target_data,
                &sql_mapper.field_order,
                // &used_paths,
                &selected_paths,
                //  &sql_mapper.joins,
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


    pub fn aux_param_values(
        aux_param_names: &Vec<String>,
        aux_params: &HashMap<String, SqlArg>,
    ) -> Result<Vec<SqlArg>, SqlBuilderError> {
        let mut params: Vec<SqlArg> = Vec::with_capacity(aux_param_names.len());
        for p in aux_param_names {
            let qp = aux_params
                .get(p)
                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
            params.push(qp.to_owned());
        }
        Ok(params)
    }

    pub fn resolve_query_params(
        expression: &str,
        aux_params: &HashMap<String, SqlArg>,
    ) -> Result<Sql, SqlBuilderError> {
        let (sql, params) = Self::extract_query_params(expression);

        let mut resolved: Vec<SqlArg> = Vec::new();
        for p in params {
            let v = aux_params
                .get(&p)
                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
            resolved.push(v.to_owned());
        }

        Ok((sql, resolved))
    }

    pub fn extract_query_params(expression: &str) -> (String, Vec<String>) {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new(r"<([\w_]+)>").unwrap();
        }

        let mut query_params = Vec::new();
        let sql = REGEX.replace(expression, |e: &regex::Captures| {
            let name = &e[1];
            query_params.push(name.to_string());
            "?"
        });
        (sql.to_string(), query_params)
    }

    fn build_ordering(
        result: &mut SqlBuilderResult,
        query_aux_params: &HashMap<String, SqlArg>,
        sql_target_data: &HashMap<&str, SqlTargetData>,
        sql_targets: &HashMap<String, SqlTarget>,
        ordinals: &HashSet<u8>,
        ordering: &HashMap<u8, Vec<(FieldOrder, String)>>,
    ) -> Result<(), SqlBuilderError> {
        // Build ordering clause
        for n in ordinals {
            if let Some(fields) = ordering.get(n) {
                for (ord, toql_field) in fields {
                    let o = match ord {
                        FieldOrder::Asc(_) => " ASC",
                        FieldOrder::Desc(_) => " DESC",
                    };
                    if let Some(_sql_target_data) = sql_target_data.get(toql_field.as_str()) {
                        if let Some(sql_target) = sql_targets.get(toql_field) {
                            let mut combined_aux_params: HashMap<String, SqlArg> = HashMap::new();
                            let aux_params = Self::combine_aux_params(
                                &mut combined_aux_params,
                                query_aux_params,
                                &sql_target.options.aux_params,
                            );
                            let aux_param_values =  Self::aux_param_values(&sql_target.sql_aux_param_names, aux_params)?;
                            if let Some(s) = sql_target.handler.build_select(
                                (sql_target.expression.to_owned(), aux_param_values),
                                aux_params,
                            )? {
                                result.order_clause.push_str(&s.0);
                                result.order_params.extend_from_slice(&s.1);
                            }
                        }
                    }
                    result.order_clause.push_str(o);
                    result.order_clause.push_str(", ");
                }
            }
        }
        result.order_clause = result.order_clause.trim_end_matches(", ").to_string();
        Ok(())
    }

    fn build_count_select_clause(
        result: &mut SqlBuilderResult,
        query_aux_params: &HashMap<String, SqlArg>,
        sql_targets: &HashMap<String, SqlTarget>,
        field_order: &Vec<String>,
    ) -> Result<(), SqlBuilderError> {
        let mut any_selected = false;
        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                // For selected fields there exists target data
                if sql_target.options.count_select {
                    let mut combined_aux_params: HashMap<String, SqlArg> = HashMap::new();
                    let aux_params = Self::combine_aux_params(
                        &mut combined_aux_params,
                        query_aux_params,
                        &sql_target.options.aux_params,
                    );

                    let aux_param_values =  Self::aux_param_values(&sql_target.sql_aux_param_names, aux_params)?;
                    if let Some(sql_field) = sql_target.handler.build_select(
                        (sql_target.expression.to_owned(), aux_param_values),
                        aux_params,
                    )? {
                        result.select_clause.push_str(&sql_field.0);
                        result.select_params.extend_from_slice(&sql_field.1);
                        result.select_clause.push_str(", ");
                        any_selected = true;
                    }
                }
            }
        }
        result.any_selected = any_selected;
        if any_selected {
            // Remove last ,
            result.select_clause = result.select_clause.trim_end_matches(", ").to_string();
        } else {
            result.select_clause = "1".to_string();
        }
        Ok(())
    }

    fn build_select_clause(
        result: &mut SqlBuilderResult,
        query_aux_params: &HashMap<String, SqlArg>,
        sql_targets: &HashMap<String, SqlTarget>,
        sql_target_data: &HashMap<&str, SqlTargetData>,
        field_order: &Vec<String>,
        //  used_paths: &HashSet<String>,
        selected_paths: &HashSet<String>,
        // joins: &HashMap<String, Join>,
    ) -> Result<(), SqlBuilderError> {
        // Build select clause
        let mut any_selected = false;

        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                // Skip fields that must not appear in select statement
                

                let path: &str = toql_field
                    .trim_end_matches(|c| c != '_')
                    .trim_end_matches('_');
                // For selected fields there exists target data
                // For always selected fields, check if path is used by query
                let selected = (/*join_selected
                ||*/sql_target.options.preselect
                        //&& (path.is_empty() || used_paths.contains(&path)))
                        && (path.is_empty() || selected_paths.contains(path)))
                    || sql_target_data
                        .get(toql_field.as_str())
                        .map_or(false, |d| d.selected);

                if selected {
                    let mut combined_aux_params: HashMap<String, SqlArg> = HashMap::new();
                    let aux_params = Self::combine_aux_params(
                        &mut combined_aux_params,
                        query_aux_params,
                        &sql_target.options.aux_params,
                    );

                    let params =  Self::aux_param_values(&sql_target.sql_aux_param_names, aux_params)?;
                    if let Some(sql_field) = sql_target
                        .handler
                        .build_select((sql_target.expression.to_owned(), params), aux_params)?
                    {
                        result.select_clause.push_str(&sql_field.0);
                        result.select_params.extend_from_slice(&sql_field.1);

                        /*  // Replace query params with actual values
                        for p in &sql_target.sql_query_params {
                            let qp = query_params
                                .get(p)
                                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
                            result.select_params.push(qp.to_string());
                        } */

                        any_selected = true;
                    } else {
                        result.select_clause.push_str("null");
                    }
                } else {
                    result.select_clause.push_str("null");
                }
                result.select_clause.push_str(", ");
            }
        }
        result.any_selected = any_selected;
        // Remove last ,
        result.select_clause = result.select_clause.trim_end_matches(", ").to_string();
        Ok(())
    }
    fn build_join_clause(
        join_root: &Vec<String>,
        join_tree: &HashMap<String, Vec<String>>,
        selected_paths: &mut HashSet<String>,
        sql_joins: &HashMap<String, Join>,
        aux_params: &HashMap<String, SqlArg>,
        result: &mut SqlBuilderResult,
    ) -> Result<(), SqlBuilderError> {
        fn build_join_start(join: &Join) -> String {
            let mut result = String::from(match join.join_type {
                JoinType::Inner => "JOIN (",
                JoinType::Left => "LEFT JOIN (",
            });
            result.push_str(&join.aliased_table);
            result
        }
        /* fn build_join_end(join: &Join) -> String {
            let mut result = String::from(") ON (");
            result.push_str(&join.on_predicate);
            result.push_str(") ");
            result
        } */
        fn build_joins(
            joins: &Vec<String>,
            selected_paths: &mut HashSet<String>,
            sql_joins: &HashMap<String, Join>,
            aux_params: &HashMap<String, SqlArg>,
            result: &mut SqlBuilderResult,
            join_tree: &HashMap<String, Vec<String>>,
        ) -> Result<(), SqlBuilderError>{
            for join in joins {
                // Construct join if
                // - join is left join and selected
                // - join is inner join (must always be selected)
                let join_data = sql_joins.get(join.as_str());
                if let Some(join_data) = join_data {
                    // If join is used in query

                    // Add path for preselected join
                    if join_data.options.preselect {
                        selected_paths.insert(join.to_owned());
                    }
                    // Construction rules for joins:
                    // - Preselected and Inner Joins always
                    // - Left Joins only on demand
                    let construct = join_data.options.preselect
                        || match join_data.join_type {
                            JoinType::Inner => true,
                            JoinType::Left => selected_paths.contains(join.as_str()),
                        };
                    if construct {
                        if let Some(t) = sql_joins.get(join) {
                            result.join_clause.push_str(build_join_start(&t).as_str());
                            result.join_clause.push(' ');
                            // Ressolve nested joins
                            resolve_nested(&join, selected_paths, sql_joins, aux_params,result, join_tree)?;
                            result.join_clause.pop(); // remove last whitespace

                           result.join_clause.push_str(") ON (");
                            
            
                          
                            
                             // Combine aux params from query and local join params
                             let mut combined_aux_params: HashMap<String, SqlArg> =
                                        HashMap::new();
                                    let temp_aux_params = SqlBuilder::combine_aux_params(
                                        &mut combined_aux_params,
                                        &aux_params,
                                        &join_data.options.aux_params,
                                    );
                                                       
                            let params = SqlBuilder::aux_param_values(&join_data.sql_aux_param_names, &temp_aux_params)?;
                            match &t.options.join_handler{
                                Some(h) => {
                                    let (on, pa) = h.build_on_predicate((t.on_predicate.to_owned(),params), aux_params)?;
                                    result.join_clause.push_str(&on);
                                    result.join_clause.push_str(") ");
                                    result.join_params.extend_from_slice(&pa);
                                }
                                None => {
                                    result.join_clause.push_str(&t.on_predicate);
                                    result.join_clause.push_str(") ");
                                    result.join_params.extend_from_slice(&params);
                                }
                            }; 
                                
                            
                            
                        }
                    }
                }
            }
            Ok(())
        }
        fn resolve_nested(
            path: &str,
            selected_paths: &mut HashSet<String>,
            sql_joins: &HashMap<String, Join>,
            aux_params: &HashMap<String, SqlArg>,
            result: &mut SqlBuilderResult,
            join_tree: &HashMap<String, Vec<String>>,
        ) -> Result<(), SqlBuilderError>{
            if join_tree.contains_key(path) {
                let joins = join_tree.get(path).unwrap();
                build_joins(&joins, selected_paths, sql_joins, aux_params,result, join_tree)?;
            }
            Ok(())
        }

        //println!("Selected joins {:?}", selected_paths);
        // Process top level joins
        build_joins(join_root, selected_paths, sql_joins, aux_params, result, join_tree)?;

        // Process all fields with subpaths from the query
        /*for (k, v) in sql_join_data  {
            // If not yet joined, check if subpath should be optionally joined
            if !v.joined {
                // For every subpath, check if there is JOIN data available
                if let Some(t) = sql_joins.get(*k) {
                    // If there is JOIN data available, use it to construct join
                    // Join data can be missing for directly typed join

                    result.join_clause.push_str(&t.join_clause);
                    result.join_clause.push(' ');
                }
                v.joined = true; // Mark join as processed
            }
        } */

        // Remove last whitespace
        if result
            .join_clause
            .chars()
            .rev()
            .next()
            .unwrap_or('A')
            .is_whitespace()
        {
            result.join_clause = result.join_clause.trim_end().to_string();
        }
        Ok(())
    }

    fn path_ignored(&self, field_name: &str) -> bool {
        for path in &self.ignored_paths {
            if field_name.starts_with(path) {
                return true;
            }
        }
        false
    }

    fn combine_aux_params<'b>(
        combined_aux_params: &'b mut HashMap<String, SqlArg>,
        query_aux_params: &'b HashMap<String, SqlArg>,
        sql_target_aux_params: &'b HashMap<String, SqlArg>,
    ) -> &'b HashMap<String, SqlArg> {
        if sql_target_aux_params.is_empty() {
            query_aux_params
        } else {
            for (k, v) in query_aux_params {
                combined_aux_params.insert(k.clone(), v.clone());
            }
            for (k, v) in sql_target_aux_params {
                combined_aux_params.insert(k.clone(), v.clone());
            }
            combined_aux_params
        }
    }

    fn insert_paths(field_with_path:&str, paths: &mut HashSet<String>) {
            let mut path = field_with_path
                                    .trim_end_matches(|c| c != '_')
                                    .trim_end_matches('_');
                while !path.is_empty() {
                    let exists = !paths.insert(path.to_owned());
                    if exists {
                        break;
                    }
                    path =
                        path.trim_end_matches(|c| c != '_').trim_end_matches('_');
                }

    }
}
