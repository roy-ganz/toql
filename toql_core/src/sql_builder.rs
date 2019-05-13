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

pub struct SqlBuilder {
    count_query: bool,       // Build count query
    subpath: String,         // Build only subpath
    joins: BTreeSet<String>, // Use this joins
    ignored_paths: Vec<String>, // Ignore paths, no errors are raised for them
                             // alias: String,           // Alias all fields with this
}

#[derive(Debug)]
pub enum SqlBuilderError {
    FieldMissing(String),
    RoleRequired(String),
}

impl fmt::Display for SqlBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlBuilderError::FieldMissing(ref s) =>
                write!(f, "field `{}` is missing", s),
            SqlBuilderError::RoleRequired(ref s) =>
                write!(f, "role `{}` is required", s),
            
        }
    }
}


impl SqlBuilder {
    pub fn new() -> Self {
        SqlBuilder {
            count_query: false,
            subpath: "".to_string(),
            joins: BTreeSet::new(),
            ignored_paths: Vec::new(),
        }
    }
    pub fn ignore_path<T: Into<String>>(mut self, path: T) -> Self {
        self.ignored_paths.push(path.into());
        self
    }
    /* pub fn for_role<T: Into<String>>(mut self, role: T) -> Self {
        self.roles.insert(role.into());
        self
    } */
    pub fn with_join<T: Into<String>>(mut self, join: T) -> Self {
        self.joins.insert(join.into());
        self
    }

    pub fn build_count(
        &mut self,
        sql_mapper: &SqlMapper,
        query: &Query,
    ) -> Result<SqlBuilderResult, SqlBuilderError> {
        self.count_query = true;
        self.build(sql_mapper, query)
    }

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
                            if let Some(s) = sql_target.handler.build_select(&sql_target.expression)
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
        sql_targets: &HashMap<String, SqlTarget>,
        field_order: &Vec<String>,
    ) {
        let mut any_selected = false;
        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                // For selected fields there exists target data
                if sql_target.options.count_select {
                    if let Some(sql_field) = sql_target.handler.build_select(&sql_target.expression)
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
        sql_targets: &HashMap<String, SqlTarget>,
        sql_target_data: &HashMap<&str, SqlTargetData>,
        field_order: &Vec<String>,
    ) {
        // Build select clause
        let mut any_selected = false;
        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                // For selected fields there exists target data
                let selected = sql_target.options.always_selected
                    || sql_target_data
                        .get(toql_field.as_str())
                        .map_or(false, |d| d.selected);

                if selected {
                    if let Some(sql_field) = sql_target.handler.build_select(&sql_target.expression)
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

                                data.selected = if self.count_query {
                                    sql_target.options.count_select
                                } else {
                                    !query_field.hidden
                                };

                                data.used = !query_field.hidden;

                                if let Some(f) = &query_field.filter {
                                    if let Some(f) =
                                        sql_target.handler.build_filter(&sql_target.expression, &f)
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
                                    let mut p = sql_target.handler.build_param(&f);
                                    if query_field.aggregation == true {
                                        result.having_params.append(&mut p);
                                    } else {
                                        result.where_params.append(&mut p);
                                    }

                                    if let Some(j) = sql_target.handler.build_join() {
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
                                    return Err(SqlBuilderError::FieldMissing(format!(
                                        "no field mapped for `{}`",
                                        query_field.name
                                    )));
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
                if sql_target.options.always_selected && sql_target.subfields {
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
                &sql_mapper.fields,
                &sql_mapper.field_order,
            );
        } else {
            Self::build_ordering(
                &mut result,
                &sql_target_data,
                &sql_mapper.fields,
                &ordinals,
                &ordering,
            );
            Self::build_select_clause(
                &mut result,
                &sql_mapper.fields,
                &sql_target_data,
                &sql_mapper.field_order,
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

// Generic merge function
pub fn merge<T, O, K, F, X, Y>(
    this: &mut std::vec::Vec<T>,
    mut other: Vec<O>,
    tkey: X,
    okey: Y,
    assign: F,
) where
    O: Clone,
    K: Eq + std::hash::Hash,
    F: Fn(&mut T, O),
    X: Fn(&T) -> Option<K>,
    Y: Fn(&O) -> Option<K>,
{
    // Build index to lookup all books of an author by author id
    let mut index: HashMap<K, Vec<usize>> = HashMap::new();

    for (i, b) in this.iter().enumerate() {
        match tkey(&b) {
            Some(k) => {
                let v = index.entry(k).or_insert(Vec::new());
                v.push(i);
            }
            None => {}
        }
    }

    // Consume all authors and distribute
    for a in other.drain(..) {
        // Get all books for author id
        match &okey(&a) {
            Some(ok) => {
                let vbi = index.get(ok).unwrap();

                // Clone author for second to last books
                for bi in vbi.iter().skip(1) {
                    if let Some(mut b) = this.get_mut(*bi) {
                        assign(&mut b, a.clone());
                    }
                }

                // Assign drained author for first book
                let fbi = vbi.get(0).unwrap();
                if let Some(mut b) = this.get_mut(*fbi) {
                    assign(&mut b, a.clone());
                }
            }
            None => {}
        }
    }
}
