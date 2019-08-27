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

struct SqlJoinData {
    joined: bool, // Join has been added to join clause
}
impl Default for SqlJoinData {
    fn default() -> SqlJoinData {
        SqlJoinData { joined: false }
    }
}
/// The Sql builder to build normal queries and count queries.
pub struct SqlBuilder {
    count_query: bool,       // Build count query
    subpath: String,         // Build only subpath
    joins: BTreeSet<String>, // Use this joins
    ignored_paths: Vec<String>, // Ignore paths, no errors are raised for them
    selected_paths: BTreeSet<String> // Selected paths
                             // alias: String,           // Alias all fields with this
}

#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum SqlBuilderError {
    /// The field is not mapped to a column or SQL expression. Contains the field name.
    FieldMissing(String),
    /// The field requires a role that the query does not have. Contains the role.
    RoleRequired(String),
    /// The filter expects other arguments. Typically raised by custom functions (FN) if the number of arguments is wrong.
    FilterInvalid(String)
}

impl fmt::Display for SqlBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlBuilderError::FieldMissing(ref s) =>
                write!(f, "field `{}` is missing", s),
            SqlBuilderError::RoleRequired(ref s) =>
                write!(f, "role `{}` is required", s),
            SqlBuilderError::FilterInvalid(ref s) =>
                write!(f, "filter `{}` is invalid ", s),
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
    ) {
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
                            if let Some(s) = sql_target.handler.build_select(&sql_target.expression, query_parameters)
                            {
                                result.order_by_clause.push_str(&s);
                            }
                        }
                    }
                    result.order_by_clause.push_str(o);
                    result.order_by_clause.push_str(", ");
                }
            }
        }
        result.order_by_clause = result.order_by_clause.trim_end_matches(", ").to_string();
    }

    fn build_count_select_clause(
        result: &mut SqlBuilderResult,
          query_params: &HashMap<String, String>,
        sql_targets: &HashMap<String, SqlTarget>,
        field_order: &Vec<String>,
    ) {
        let mut any_selected = false;
        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                // For selected fields there exists target data
                if sql_target.options.count_select {
                    if let Some(sql_field) = sql_target.handler.build_select(&sql_target.expression, query_params)
                    {
                        result.select_clause.push_str(&sql_field);
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
    }

    fn build_select_clause(
        result: &mut SqlBuilderResult,
        query_params: &HashMap<String, String>,
        sql_targets: &HashMap<String, SqlTarget>,
        sql_target_data: &HashMap<&str, SqlTargetData>,
        field_order: &Vec<String>,
        used_paths: &BTreeSet<String>,
        joins: &HashMap<String, Join>
    ) {
        // Build select clause
        let mut any_selected = false;
        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
            let path :String = toql_field.split('_').rev().skip(1).collect();

             let join_selected = if sql_target.options.always_selected  {
                                     if let Some(sql_join) = joins.get(path.as_str()) {
                                        sql_join.selected 
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    };
            


                // For selected fields there exists target data
                // For always selected fields, check if path is used by query
                let selected = ( join_selected || sql_target.options.always_selected &&  used_paths.contains(&path ))  
                    || sql_target_data
                        .get(toql_field.as_str())
                        .map_or(false, |d| d.selected);

                if selected {
                    if let Some(sql_field) = sql_target.handler.build_select(&sql_target.expression, query_params)
                    {
                        result.select_clause.push_str(&sql_field);
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
    }
    fn build_join_clause(
        sql_join_data: &mut HashMap<&str, SqlJoinData>,
        sql_joins: &HashMap<String, Join>,
        result: &mut SqlBuilderResult,
    ) {
        // Process all fields with subpaths from the query
        for (k, v) in sql_join_data {
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
        }
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
        let mut sql_join_data: HashMap<&str, SqlJoinData> = HashMap::new();

        let mut used_paths: BTreeSet<String> = BTreeSet::new();

        let mut result = SqlBuilderResult {
            table: sql_mapper.table.clone(),
            any_selected: false,
            distinct: query.distinct,
            join_clause: String::from(""),
            select_clause: String::from(""),
            where_clause: String::from(""),
            order_by_clause: String::from(""),
            having_clause: String::from(""),
            where_params: vec![],
            having_params: vec![],
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
                                for subfield in field_name.split('_').rev().skip(1) {
                                    if !sql_join_data.contains_key(subfield) {
                                        sql_join_data.insert(subfield, SqlJoinData::default());
                                    }
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
                        if !self.subpath.is_empty() && ! wildcard.path.starts_with(&self.subpath)
                        {
                            continue;
                        }

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

                            // Select all top fields, that are top fields or are in the right path level
                            if  (wildcard.path.is_empty() && !sql_target.subfields)
                            || (field_name.starts_with(&wildcard.path) 
                                && field_name.rfind("_").unwrap_or(field_name.len()) < wildcard.path.len()) {
                                let f = sql_target_data.entry(field_name.as_str()).or_default();
                                f.selected = true; // Select field

                                //println!("PATH= {}", path);
                                //println!("FIELDNAME= {}", field_name);
                                //println!("MATCH = {}",field_name.starts_with(path) && field_name.rfind("_").unwrap_or(field_name.len()) < field_name.len() );
                                // Add JOIN information
                                if sql_target.subfields {
                                    for subfield in field_name.split('_').rev().skip(1) {
                                        if !sql_join_data.contains_key(subfield) {
                                            sql_join_data.insert(subfield, SqlJoinData::default());
                                        }
                                    }
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
                        if self.ignored_paths.iter().any(|p| query_field.name.starts_with(p)) {
                            continue;
                        }

                        let fieldname = if self.subpath.is_empty() {
                            &query_field.name
                        } else {
                            query_field
                                .name
                                .trim_start_matches(&self.subpath)
                                .trim_start_matches('_')
                        };

                        match sql_mapper.fields.get(fieldname) {
                            Some(sql_target) => {
                                // Verify user role and skip field role mismatches
                                let role_valid =
                                    Self::validate_roles(&query.roles, &sql_target.options.roles);
                                if role_valid == false {
                                    return Err(SqlBuilderError::RoleRequired(format!(
                                        "Field requires a user role: '{}'. ",
                                        fieldname
                                    )));
                                }
                                // Skip filtering and ordering in count queries for unfiltered fields
                                if self.count_query == true && !sql_target.options.count_filter {
                                    continue;
                                }

                                // Select sql target if field is not hidden
                                let data = sql_target_data.entry(fieldname).or_default();

                                // Add Join data for all sub fields
                                if sql_target.subfields {
                                    for subfield in fieldname.split('_').rev().skip(1) {
                                        if !sql_join_data.contains_key(subfield) {
                                            sql_join_data.insert(subfield, SqlJoinData::default());
                                        }
                                    }
                                }

                                // Add path to used path list
                                let path :String = fieldname.split('_').rev().skip(1).collect();
                                if !used_paths.contains(&path) {
                                    used_paths.insert(path);
                                }

                                data.selected = if self.count_query {
                                    sql_target.options.count_select
                                } else {
                                    !query_field.hidden
                                };

                                data.used = !query_field.hidden;

                                if let Some(f) = &query_field.filter {
                                    
                                    if let Some(f) = sql_target.handler.build_filter(&sql_target.expression, &f, &query.params)?
                                        
                                    {
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
                                                &f,
                                            );

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
                                                &f,
                                            );

                                            pending_where_parens = 0;
                                            need_where_concatenation = true;
                                        }
                                    }
                                    let mut p = sql_target.handler.build_param(&f, &query.params);
                                    if query_field.aggregation == true {
                                        result.having_params.append(&mut p);
                                    } else {
                                        result.where_params.append(&mut p);
                                    }

                                    if let Some(j) = sql_target.handler.build_join(&query.params) {
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
                                    return Err(SqlBuilderError::FieldMissing(query_field.name.clone()));
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

        // Build select
        // Ensure implicitly selected subfields are joined
        for toql_field in &sql_mapper.field_order {
            if let Some(sql_target) = sql_mapper.fields.get(toql_field.as_str()) {
                
                let path :String = toql_field.split('_').rev().skip(1).collect();
              
              
                // Fields that are markd `always selected` are selected, if either
                // their path is in use or their path belongs to a join that is always selected
                let join_selected =  if path.is_empty() { 
                        false 
                    } else { 
                        if let Some(sql_join) = sql_mapper.joins.get(path.as_str()) {
                           sql_join.selected 
                        } else {
                            false
                        }
                    };
            
               
                
                


                if sql_target.options.always_selected
                &&  (join_selected || used_paths.contains(&path ) )  
                 && sql_target.subfields {
                    for subfield in toql_field.split('_').rev().skip(1) {
                        if !sql_join_data.contains_key(subfield) {
                            sql_join_data.insert(subfield, SqlJoinData::default());
                        }
                    }
                }
            }
        }

        if self.count_query {
            Self::build_count_select_clause(
                &mut result,
                &query.params,
                &sql_mapper.fields,
                &sql_mapper.field_order,
            );
        } else {
            Self::build_ordering(
                &mut result,
                &query.params,
                &sql_target_data,
                &sql_mapper.fields,
                &ordinals,
                &ordering,
            );
            Self::build_select_clause(
                &mut result,
                &query.params,
                &sql_mapper.fields,
                &sql_target_data,
                &sql_mapper.field_order,
                &used_paths,
                &sql_mapper.joins
            );
        }

        Self::build_join_clause(&mut sql_join_data, &sql_mapper.joins, &mut result);

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
            .order_by_clause
            .chars()
            .rev()
            .next()
            .unwrap_or('_')
            .is_whitespace()
        {
            result.order_by_clause = result.order_by_clause.trim_end().to_owned();
        }

        // Create combined params if needed
        if !result.having_params.is_empty() && !result.where_params.is_empty() {
            result
                .combined_params
                .extend_from_slice(&result.where_params);
            result
                .combined_params
                .extend_from_slice(&result.having_params);
        }

        Ok(result)
    }

    fn path_ignored(&self, fieldname: &str) -> bool {
        for path in &self.ignored_paths {
            if fieldname.starts_with(path) {
                return true;
            }
        }
        false
    }
}
