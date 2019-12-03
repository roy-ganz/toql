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
use crate::query::Concatenation;
use crate::query::FieldOrder;
use crate::query::Query;
use crate::query::QueryToken;
use crate::sql_builder_result::SqlBuilderResult;
use crate::sql_mapper::Join;
use crate::sql_mapper::JoinType;
use crate::sql_mapper::SqlMapper;
use crate::sql_mapper::SqlTarget;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fmt;

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

/// The Sql builder to build normal queries and count queries.
pub struct SqlBuilder {
    count_query: bool,          // Build count query
    subpath: String,            // Build only subpath
    joins: BTreeSet<String>,    // Use this joins
    ignored_paths: Vec<String>, // Ignore paths, no errors are raised for them
    selected_paths: BTreeSet<String>, // Selected paths
}

#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum SqlBuilderError {
    /// The field is not mapped to a column or SQL expression. Contains the field name.
    FieldMissing(String),
    /// The field requires a role that the query does not have. Contains the role.
    RoleRequired(String),
    /// The filter expects other arguments. Typically raised by custom functions (FN) if the number of arguments is wrong.
    FilterInvalid(String),
    /// A query expression requires a query parameter, that is not provided. Contains the parameter.
    QueryParamMissing(String),
    /// The query parameter that is required by the query expression is wrong. Contains the parameter and the details.
    QueryParamInvalid(String, String),

}

impl fmt::Display for SqlBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlBuilderError::FieldMissing(ref s) => write!(f, "field `{}` is missing", s),
            SqlBuilderError::RoleRequired(ref s) => write!(f, "role `{}` is required", s),
            SqlBuilderError::FilterInvalid(ref s) => write!(f, "filter `{}` is invalid ", s),
            SqlBuilderError::QueryParamMissing(ref s) => {
                write!(f, "query parameter `{}` is missing ", s)
            }
            SqlBuilderError::QueryParamInvalid(ref s, ref d) => {
                write!(f, "query parameter `{}` is invalid: {} ", s, d)
            },
        }
    }
}

impl SqlBuilder {
    /// Create a new SQL Builder
    pub fn new() -> Self {
        SqlBuilder {
            count_query: false,
            subpath: "".to_string(),
            joins: BTreeSet::new(),
            ignored_paths: Vec::new(),
            selected_paths: BTreeSet::new(),
        }
    }
    /// Add path to list of ignore paths.
    pub fn ignore_path<T: Into<String>>(mut self, path: T) -> Self {
        self.ignored_paths.push(path.into());
        self
    }
    pub fn select_path<T: Into<String>>(mut self, path: T) -> Self {
        self.selected_paths.insert(path.into());
        self
    }
    /* pub fn for_role<T: Into<String>>(mut self, role: T) -> Self {
        self.roles.insert(role.into());
        self
    } */
    /// TODO
    pub fn with_join<T: Into<String>>(mut self, join: T) -> Self {
        self.joins.insert(join.into());
        self
    }

    /// Build query for total count.
    pub fn build_count(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query,
    ) -> Result<SqlBuilderResult, SqlBuilderError> {
        self.count_query = true;
        self.build(sql_mapper, query)
    }

    // Build normal query for this path
    pub fn build_path<T: Into<String>>(
        &mut self,
        path: T,
        sql_mapper: &SqlMapper,
        query: &Query,
    ) -> Result<SqlBuilderResult, SqlBuilderError> {
        self.subpath = {
            let p = path.into();
            if p.ends_with("_") {
                p
            } else {
                format!("{}_", p)
            }
        };
        self.build(sql_mapper, query)
    }

    fn validate_roles(proposed: &BTreeSet<String>, required: &BTreeSet<String>) -> bool {
        if required.is_empty() {
            return true;
        } // Is valid, if no roles are required
        if proposed.is_empty() {
            return false;
        } // Is invalid, if roles are required, but no roles proposed

        for r in required {
            if !proposed.contains(r) {
                return false;
            }
        }
        true
    }

    fn build_ordering(
        result: &mut SqlBuilderResult,
        query_parameters: &HashMap<String, String>,
        sql_target_data: &HashMap<&str, SqlTargetData>,
        sql_targets: &HashMap<String, SqlTarget>,
        ordinals: &BTreeSet<u8>,
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
                            if let Some(s) = sql_target
                                .handler
                                .build_select(&sql_target.expression, query_parameters)?
                            {
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
        query_params: &HashMap<String, String>,
        sql_targets: &HashMap<String, SqlTarget>,
        field_order: &Vec<String>,
    ) -> Result<(), SqlBuilderError> {
        let mut any_selected = false;
        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                // For selected fields there exists target data
                if sql_target.options.count_select {
                    if let Some(sql_field) = sql_target
                        .handler
                        .build_select(&sql_target.expression, query_params)?
                    {
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
        query_params: &HashMap<String, String>,
        sql_targets: &HashMap<String, SqlTarget>,
        sql_target_data: &HashMap<&str, SqlTargetData>,
        field_order: &Vec<String>,
      //  used_paths: &BTreeSet<String>,
        selected_paths: &BTreeSet<String>,
       // joins: &HashMap<String, Join>,
    ) -> Result<(), SqlBuilderError> {
        // Build select clause
        let mut any_selected = false;

       

        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                let path: &str = toql_field.trim_end_matches(|c | c != '_').trim_end_matches('_');

                

              /*   let join_selected = if sql_target.options.preselect {
                    if let Some(sql_join) = joins.get(path.as_str()) {
                        selected_paths.contains(path.as_str())
                    } else {
                        false
                    }
                } else {
                    false
                }; */

                 // For selected fields there exists target data
                // For always selected fields, check if path is used by query
                let selected = (/*join_selected
                    ||*/ sql_target.options.preselect
                        //&& (path.is_empty() || used_paths.contains(&path)))
                        && (path.is_empty() || selected_paths.contains(path)))
                    || sql_target_data
                        .get(toql_field.as_str())
                        .map_or(false, |d| d.selected);

                if selected {
                    if let Some(sql_field) = sql_target
                        .handler
                        .build_select(&sql_target.expression, query_params)?
                    {
                        result.select_clause.push_str(&sql_field.0);
                        result.select_params.extend_from_slice(&sql_field.1);

                        // Replace query params with actual values
                        for p in &sql_target.sql_query_params {
                            let qp = query_params
                                .get(p)
                                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
                            result.select_params.push(qp.to_string());
                        }

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
        selected_paths: &mut BTreeSet<String>,
        sql_joins: &HashMap<String, Join>,
        result: &mut SqlBuilderResult,
    ) {

        fn build_join_start(join: &Join) -> String {
            let mut result = String::from(match join.join_type {
                             JoinType::Inner => "JOIN ",
                             JoinType::Left => "LEFT JOIN ",
                         });
                        result.push_str(&join.aliased_table);
            result
        }
         fn build_join_end (join: &Join) -> String {
            let mut result = String::from("ON (");
                        result.push_str(&join.on_predicate);
                        result.push_str(") ");
                        result

         }
        fn build_joins(joins:&Vec<String>,selected_paths: &mut BTreeSet<String>, sql_joins: &HashMap<String, Join>, result: &mut SqlBuilderResult,  join_tree:&HashMap<String, Vec<String>>){
            
            for join in joins {

                // Construct join if
                // - join is left join and selected
                // - join is inner join (must always be selected)
                let join_data = sql_joins.get(join.as_str());
                if let Some(join_data) = join_data {
                    // If join is used in query
                    
                    // Add path for preselected join
                    if join_data.preselect  {
                        selected_paths.insert(join.to_owned());
                    }
                    // Construction rules for joins:
                    // - Preselected and Inner Joins always
                    // - Left Joins only on demand
                    let construct = join_data.preselect || match join_data.join_type {
                        JoinType::Inner  => {
                            true},
                        JoinType::Left  => selected_paths.contains(join.as_str())
                    };
                    if construct  {
                        if let Some(t) = sql_joins.get(join) {
                            result.join_clause.push_str(build_join_start(&t).as_str());
                            result.join_clause.push(' ');
                            // Ressolve nested joins
                            resolve_nested(&join, selected_paths, sql_joins, result,  join_tree);
                            result.join_clause.push_str(build_join_end(&t).as_str());
                        }
                    }
                }
            }

        }
        fn resolve_nested(path:&str, selected_paths: &mut BTreeSet<String>, sql_joins: &HashMap<String, Join>, result: &mut SqlBuilderResult,  join_tree:&HashMap<String, Vec<String>>) {

             if join_tree.contains_key(path) {
                 let joins = join_tree.get(path).unwrap();
                build_joins(&joins,selected_paths, sql_joins, result, join_tree);
            } 
        }

        println!("Selected joins {:?}", selected_paths);
        // Process top level joins
        build_joins(join_root, selected_paths, sql_joins, result,  join_tree);


        


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
    }

   

    /// Build normal query.
    pub fn build(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query,
    ) -> Result<SqlBuilderResult, SqlBuilderError> {
        let mut ordinals: BTreeSet<u8> = BTreeSet::new();
        let mut ordering: HashMap<u8, Vec<(FieldOrder, String)>> = HashMap::new();

        let mut need_where_concatenation = false;
        let mut need_having_concatenation = false;
        let mut pending_where_parens_concatenation: Option<Concatenation> = None;
        let mut pending_having_parens_concatenation: Option<Concatenation> = None;
        let mut pending_where_parens: u8 = 0;
        let mut pending_having_parens: u8 = 0;

        let mut sql_target_data: HashMap<&str, SqlTargetData> = HashMap::new();
        let mut selected_paths: BTreeSet<String> = BTreeSet::new();

       // let mut used_paths: BTreeSet<String> = BTreeSet::new();

        let mut result = SqlBuilderResult {
            table: sql_mapper.table.clone(),
            any_selected: false,
            distinct: query.distinct,
            join_clause: String::from(""),
            select_clause: String::from(""),
            where_clause: String::from(""),
            order_clause: String::from(""),
            having_clause: String::from(""),
            select_params: vec![], // query parameters in select clause, due to sql expr with <param>
            where_params: vec![],
            having_params: vec![],
            order_params: vec![],
            combined_params: vec![],
        };


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
                    QueryToken::DoubleWildcard(..) => {
                        // Skip wildcard for count queries
                        if self.count_query {
                            continue;
                        }
                        for (field_name, sql_target) in &sql_mapper.fields {
                            // Skip fields, that ignore wildcard
                            if sql_target.options.ignore_wildcard {
                                continue;
                            }
                            if self.ignored_paths.iter().any(|p| field_name.starts_with(p)) {
                                continue;
                            }

                            // Skip fields with missing role
                            let role_valid =
                                Self::validate_roles(&query.roles, &sql_target.options.roles);
                            if role_valid == false {
                                continue;
                            }
                            let f = sql_target_data.entry(field_name.as_str()).or_default();
                            f.selected = true; // Select field
                                               // Add JOIN information for subfields
                            if sql_target.subfields {
                                for path in field_name.split('_').rev().skip(1) {
                                    let exists= selected_paths.insert(path.to_owned());
                                    if exists { break;}
                                }
                            }
                        }
                    }

                    QueryToken::Wildcard(wildcard) => {
                        // Skip wildcard for count queries
                        if self.count_query {
                            continue;
                        }
                        // Skip field from other path
                        if !self.subpath.is_empty() && !wildcard.path.starts_with(&self.subpath) {
                            continue;
                        }
                        
                        let wildcard_path = wildcard.path.trim_start_matches(&self.subpath).trim_end_matches('_');
                        for (field_name, sql_target) in &sql_mapper.fields {
                            if sql_target.options.ignore_wildcard {
                                continue;
                            }
                            if self.ignored_paths.iter().any(|p| field_name.starts_with(p)) {
                                continue;
                            }
                            // Skip fields with missing role
                            let role_valid =
                                Self::validate_roles(&query.roles, &sql_target.options.roles);
                            if role_valid == false {
                                continue;
                            }

                            let field_path = field_name.trim_end_matches(|c| c != '_').trim_end_matches('_');

                            // Select all fields on wildcard path
                            // including joins with preselected fields only
                            
                            let select = (field_path == wildcard_path) || field_path.starts_with(wildcard_path) && sql_target.options.preselect;
                            //println!( "field {}: field_path={}, wildcard_path ={}, select={}",&field_name, field_path, &wildcard.path, select);

                            /* if (wildcard.path.is_empty())  && ! sql_target.subfields
                                || (field_name.starts_with(&wildcard.path) 
                                    && field_name.rfind("_").unwrap_or(field_name.len())
                                        < wildcard.path.len()) */
                            if select
                            {
                                let f = sql_target_data.entry(field_name.as_str()).or_default();

                                f.selected = true; // Select field

                              
                                // Ensure all parent paths are selected
                                if sql_target.subfields {
                                    let mut path = field_name.trim_end_matches(|c| c!='_').trim_end_matches('_');
                                    while !path.is_empty() {
                                        let exists = !selected_paths.insert(path.to_owned());
                                        if exists { break;}
                                        path = path.trim_end_matches(|c| c!='_').trim_end_matches('_');
                                    }

                                    /* for subfield in field_name.split('_').rev().skip(1) {
                                         let exists= selected_paths.insert(subfield);
                                    if exists { break;}
                                    } */
                                }
                            }
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
                                let role_valid =
                                    Self::validate_roles(&query.roles, &sql_target.options.roles);
                                if role_valid == false {
                                    return Err(SqlBuilderError::RoleRequired(format!(
                                        "Field requires a user role: '{}'. ",
                                        field_name
                                    )));
                                }
                                // Skip filtering and ordering in count queries for unfiltered fields
                                if self.count_query == true && !sql_target.options.count_filter {
                                    continue;
                                }

                                // Select sql target if field is not hidden
                                let data = sql_target_data.entry(field_name).or_default();

                                // Add parent Joins  
                                if sql_target.subfields {
                                    
                                    
                                    let mut path = field_name.trim_end_matches(|c| c!='_').trim_end_matches('_');
                                    while !path.is_empty() {

                                        let exists = !selected_paths.insert(path.to_owned());
                                        if exists { break;}
                                        path = path.trim_end_matches(|c| c!='_').trim_end_matches('_');
                                    }
                                }
                                //println!("{:?}", selected_paths);

                              

                                data.selected = if self.count_query {
                                    sql_target.options.count_select
                                } else {
                                    !query_field.hidden
                                };

                                data.used = !query_field.hidden;

                                // Resolve query params in sql expression
                                for p in &sql_target.sql_query_params {
                                    let qp = query
                                        .params
                                        .get(p)
                                        .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;

                                    if query_field.aggregation == true {
                                        result.having_params.push(qp.to_string());
                                    } else {
                                        result.where_params.push(qp.to_string());
                                    }
                                }

                                if let Some(f) = &query_field.filter {
                                    // Get actual expression
                                    let expression = sql_target
                                        .handler
                                        .build_select(&sql_target.expression, &query.params)?
                                        .unwrap_or(("null".to_string(), vec![]));

                                    if let Some(f) = sql_target.handler.build_filter(
                                        expression, // todo change build_filter signature to take tuple
                                        &f,
                                        &query.params,
                                    )? {
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

                                    // Add filter param to result
                                    /* let mut p = sql_target.handler.build_param(&f, &query.params);
                                                                       if query_field.aggregation == true {
                                                                           result.having_params.append(&mut p);
                                                                       } else {
                                                                           result.where_params.append(&mut p);
                                                                       }
                                    */
                                    if let Some(j) = sql_target.handler.build_join(&query.params)? {
                                        result.join_clause.push_str(&j);
                                        result.join_clause.push_str(" ");
                                    }
                                }
                                if let Some(o) = &query_field.order {
                                    let num = match o {
                                        FieldOrder::Asc(num) => num,
                                        FieldOrder::Desc(num) => num,
                                    };
                                    ordinals.insert(*num);
                                    let l = ordering.entry(*num).or_insert(Vec::new());
                                    l.push((o.clone(), query_field.name.clone())); // OPTIMISE
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
                    }
                }
            }
        }

        // Select all fields for count queries that are marked with count_select
        if self.count_query {
            for (field_name, mapper_field) in &sql_mapper.fields {
                if mapper_field.options.count_select {
                    let f = sql_target_data.entry(field_name.as_str()).or_default();
                    f.selected = true;
                }
            }
        }

        /* // Build select
        // Ensure selected subfields are joined
        for toql_field in &sql_mapper.field_order {
            if let Some(sql_target) = sql_mapper.fields.get(toql_field.as_str()) {
                let path: String = toql_field.split('_').rev().skip(1).collect();

                // Fields that are marked `preselect` are selected, if either
                // their path is in use or
                // their path belongs to a join that is always selected (Inner Join)
                 let join_selected = if path.is_empty() {
                    false
                } else {
                    if let Some(sql_join) = sql_mapper.joins.get(path.as_str()) {
                        sql_join.join_type == JoinType::Inner && selected_paths.contains(path.as_str())
                    } else {
                        false
                    }
                }; 

                if sql_target.options.preselect
                    && (join_selected || used_paths.contains(&path))
                    //&& used_paths.contains(&path)
                    && sql_target.subfields
                {
                    for subfield in toql_field.split('_').rev().skip(1) {
                        let exists= selected_paths.insert(subfield);
                                    if exists { break;}
                    }
                }
            }
        } */

       // println!("Selected joins from query {:?}", selected_paths);
         Self::build_join_clause(&sql_mapper.joins_root, &sql_mapper.joins_tree, &mut selected_paths, &sql_mapper.joins, &mut result);
    
        //println!("Selected joins including inner joins {:?}", selected_paths);
        
        if self.count_query {
            Self::build_count_select_clause(
                &mut result,
                &query.params,
                &sql_mapper.fields,
                &sql_mapper.field_order,
            )?;
        } else {
            Self::build_ordering(
                &mut result,
                &query.params,
                &sql_target_data,
                &sql_mapper.fields,
                &ordinals,
                &ordering,
            )?;
            Self::build_select_clause(
                &mut result,
                &query.params,
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
            .extend_from_slice(&result.where_params);
        result
            .combined_params
            .extend_from_slice(&result.having_params);
        result
            .combined_params
            .extend_from_slice(&result.order_params);

        // Create combined params if needed
        /* if !result.having_params.is_empty() && !result.where_params.is_empty() {
            result
                .combined_params
                .extend_from_slice(&result.where_params);
            result
                .combined_params
                .extend_from_slice(&result.having_params);
        } */

        Ok(result)
    }

    pub fn resolve_query_params(
        expression: &str,
        query_params: &HashMap<String, String>,
    ) -> Result<(String, Vec<String>), SqlBuilderError> {
        let (sql, params) = Self::extract_query_params(expression);

        let mut resolved: Vec<String> = Vec::new();
        for p in params {
            let v = query_params
                .get(&p)
                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
            resolved.push(v.to_string());
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

    fn path_ignored(&self, field_name: &str) -> bool {
        for path in &self.ignored_paths {
            if field_name.starts_with(path) {
                return true;
            }
        }
        false
    }
}
