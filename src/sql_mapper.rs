use crate::query::Field;

use crate::query::GroupSearch;

use crate::query::Concatenation;
use crate::query::FieldFilter;
use crate::query::FieldOrder;
use crate::query::Query;
use crate::query::QueryToken;
use std::collections::BTreeSet;
use std::collections::HashMap;

enum FilterType {
    Where,
    Having,
    None,
}

pub struct SqlTarget {
    selected: bool,             // Target is selected
    alias: String,              // Calculated alias for field
    used: bool,                 // Target is either selected or filtered
    joined_target: bool,        // Target must be joined
    joined: bool,               // Join clause set for target
    options: MapperOptions,     // Options
    filter_type: FilterType,    // Filter on where or having clause
    handler: Box<FieldHandler>, // Handler to create clauses
}

pub struct SqlField {
    name: String
}
pub struct MapperOptions {
    always_selected: bool, // Always select this field, regardless of query fields
    alias: bool,           // This field must not be aliased
    count_query: bool,     // Use this field also in count query
    ignore_wildcard: bool, // Ignore field for wildcard selection
    role: Option<String>,  // Only for use by this role
}

// OPT use references
impl MapperOptions {
    pub fn new() -> Self {
        MapperOptions {
            always_selected: false,
            alias: true,
            count_query: false,
            ignore_wildcard: false,
            role: None
        }
    }
    pub fn select_always(mut self, always_selected: bool) -> Self {
        self.always_selected = always_selected;
        self
    }
    pub fn alias(mut self, alias: bool) -> Self {
        self.alias = alias;
        self
    }
    pub fn use_for_count_query(mut self, count_query: bool) -> Self {
        self.count_query = count_query;
        self
    }
    pub fn ignore_for_wildcard(mut self, ignore_wildcard: bool) -> Self {
        self.ignore_wildcard = ignore_wildcard;
        self
    }
    pub fn restrict_to_role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }
    
}

pub struct BuildOptions {
    count_query: bool,    // Build count query
    subpath: String,      // Build only subpath
    role: Option<String>, // Build for role
   
}

impl BuildOptions {
    pub fn new() -> Self {
        BuildOptions {
            count_query: false,
            subpath: "".to_string(),
            role: None,
            
            
        }
    }
    pub fn build_count_query(mut self, count_query: bool) -> Self {
        self.count_query = count_query;
        self
    }
    pub fn build_subpath(mut self, subpath: &str) -> Self {
        self.subpath = subpath.to_string();
        self
    }
    pub fn build_role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }
   
}

trait MapperFilter {
    fn build(field: crate::query::QueryToken) -> String;
}
pub trait FieldHandler {
    fn validate_query(&self) -> bool {
        true
    }
    fn build_select(&self,  alias: &str) -> Option<String> {
        None
    }
    fn build_filter(&self,  alias: &str,filter: &FieldFilter) -> Option<String> {
        None
    }
    fn build_param(&self, filter: &FieldFilter) -> Option<String> {
        None
    }
    fn build_join(&self) -> Option<String> {
        None
    }
}

// Test trait to develop derive
pub trait ToqlQuery {
    fn hello_macro();
}

impl FieldHandler for SqlField {
    fn build_select(&self, alias: &str) -> Option<String> {
        Some(format!("{}{}", alias,self.name))
    }
    fn build_param(&self, filter: &FieldFilter) -> Option<String> {
        match filter {
            FieldFilter::Eq(criteria) => Some(criteria.clone()),
            FieldFilter::Ne(criteria) => Some(criteria.clone()),
            FieldFilter::Ge(criteria) => Some(criteria.clone()),
            FieldFilter::Gt(criteria) => Some(criteria.clone()),
            FieldFilter::Le(criteria) => Some(criteria.clone()),
            FieldFilter::Lt(criteria) => Some(criteria.clone()),
            FieldFilter::In(_) => None,
            FieldFilter::Out(_) => None,
            FieldFilter::Lk(criteria) => Some(criteria.clone()),
            FieldFilter::Other(_) => None,
            _ => None,
        }
    }
    fn build_filter(&self, alias: &str, filter: &FieldFilter) -> Option<String> {
        match filter {
            FieldFilter::Eq(_) => Some(format!("{}{} = ?",alias, self.name)),
            FieldFilter::Ne(_) => Some(format!("{}{} <> ?",alias, self.name)),
            FieldFilter::Ge(_) => Some(format!("{}{} >= ?",alias, self.name)),
            FieldFilter::Gt(_) => Some(format!("{}{} > ?",alias, self.name)),
            FieldFilter::Le(_) => Some(format!("{}{} <= ?",alias, self.name)),
            FieldFilter::Lt(_) => Some(format!("{}{} < ?",alias, self.name)),
            FieldFilter::In(values) => Some(format!("{}{} IN ({})", alias, self.name, values.join(","))),
            FieldFilter::Out(values) => Some(format!("{}{} NOT IN ({})", alias, self.name, values.join(","))),
            FieldFilter::Lk(_) => Some(format!("{}{} Lk ?",alias, self.name)),
            FieldFilter::Other(_) => None,
            _ => None,
        }
    }
    /*  fn build_join(&self) ->Option<String> {
        self.join_clause.clone()
    } */
}

pub struct SqlMapper {
    dirty: bool,
    alias: String,
    field_order: Vec<String>,
    fields: HashMap<String, SqlTarget>,
    joins: HashMap<String, Join>,
}

pub struct Join {
    join_clause: String,
    alias: String,
    joined: bool
}

pub struct MapperResult {
    pending_having_parens: u8,
    pending_where_parens: u8,
    need_having_concatenation: bool,
    need_where_concatenation: bool,
    pending_where_parens_concatenation: Option<Concatenation>,
    pending_having_parens_concatenation: Option<Concatenation>,
    pub join_clause: String,
    pub select_clause: String,
    pub where_clause: String,
    pub order_by_clause: String,
    pub having_clause: String,
    pub count_where_clause: String,
    pub count_having_clause: String,
    pub unused_fields: Vec<String>,
    pub where_params: Vec<String>,
    pub having_params: Vec<String>,
}

impl MapperResult {
    fn push_pending_parens(clause: &mut String, pending_parens: &u8) {
        for _n in 0..*pending_parens {
            clause.push_str("(");
        }
    }
    fn push_concatenation(clause: &mut String, pending_concatenation: &Option<Concatenation>) {
        if let Some(c) = pending_concatenation {
            match c {
                Concatenation::And => clause.push_str(" AND "),
                Concatenation::Or => clause.push_str(" OR "),
            }
        }
    }
    fn push_filter(clause: &mut String, filter: &str) {
        clause.push_str(filter);
    }
}

pub trait Mappable {
    fn map(mapper: &mut SqlMapper, prefix: &str);
}

impl SqlMapper {
    pub fn new() -> SqlMapper {
        SqlMapper {
            dirty: false,
            alias: "".to_string(),
            joins: HashMap::new(),
            fields: HashMap::new(),
            field_order: Vec::new(),
        }
    }
    pub fn map<T: Mappable>() -> SqlMapper {
        let mut m = Self::new();
        T::map(&mut m, "");
        m
    }
    pub fn map_with_prefix<T: Mappable>(prefix: &str) -> SqlMapper {
        let mut m = Self::new();
        T::map(&mut m, prefix);
        m
    }

    pub fn map_aliased<T: Mappable>(alias: &str) -> SqlMapper {
        let mut m = Self::map::<T>();
        m.alias = alias.to_string();
        m
    }

    pub fn map_handler<'a>(
        &'a mut self,
        toql_field: &str,
        handler: Box<FieldHandler>,
        options: MapperOptions,
    ) {
        // Check if toql field has underscore
        let x = toql_field.split('_').rev().next().is_some();

        let t = SqlTarget {
            options: options,
            filter_type: FilterType::Where, // filter on where clause
            selected: false,
            used: false,
            alias: String::from(""),
            joined: false,
            joined_target: x,
            handler: handler,
        };
        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
    }

    pub fn alter_field(&mut self, toql_field :&str, sql_field :&str, options: MapperOptions){
        let sql_target = self.fields.get_mut(toql_field).expect("Field  is not mapped.");
        
            let f = SqlField {
                name: sql_field.to_string()
            };
        sql_target.options = options; 
        sql_target.handler = Box::new(f);
        
    }

    pub fn map_field<'a>(
        &'a mut self,
        toql_field: &str,
        sql_field: &str,
        options: MapperOptions,
    ) -> &'a mut Self {
        let f = SqlField {
            name: sql_field.to_string()
        };

        let j = toql_field.find('_').is_some(); 
        let t = SqlTarget {
            options: options,
            filter_type: FilterType::Where, // filter on where clause
            selected: false,
            alias: String::from(""),
            used: false,
            joined_target: j,
            joined: false,
            handler: Box::new(f),
        };

        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }

    /*   pub fn join_field<'a>(   &'a mut self,   toql_field: &str,  sql_field: &str, join_clause: &str) -> &'a mut Self {

       let f = SqlField {
            name: sql_field.to_string(),
            join_clause: Some(join_clause.to_string()),
            selected: false,
        };
        let t = SqlTarget {
            options: MapperOptions::new().alias(false),
            filter_type: FilterType::Where, // filter on where clause
            selected: false,
            handler: Box::new(f),
        };

        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    } */

    pub fn join_fields<'a>(
        &'a mut self,
        toql_field: &str,
        alias: &str,
        join_clause: &str,
    ) -> &'a mut Self {
        self.joins.insert(
            toql_field.to_string(),
            Join {
                alias: alias.to_string(),
                join_clause: join_clause.to_string(),
                joined: false
            },
        );
        self
    }

    fn alias_fields(&mut self) {
            for (field_name, sql_target) in &mut self.fields {
                   
                    if !self.alias.is_empty() && sql_target.options.alias {
                            // joined fields use join alias , other use field alias
                            let al = if sql_target.joined_target {
                                if let Some(n) = field_name.split('_').rev().nth(1) {
                                    &self.joins.get(n).unwrap().alias
                                } else {
                                    panic!("Join not found for field {}", field_name);
                                }
                            } else {
                                &self.alias
                            }; 
                            sql_target.alias = format!("{}.",al);
                    }

            }
        
            }

    fn clean_targets (&mut self) {
        for (k, t) in &mut self.fields {
            t.selected = false;
        }
    }
    fn clean_joins (&mut self) {
        for (k, t) in &mut self.joins {
            t.joined = false;
        }
    }

    fn build_ordering(
        result: &mut MapperResult,
         targets: &HashMap<String, SqlTarget>,
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
                     if let Some(sql_target) = targets.get(toql_field) {
                         if let Some(s) = sql_target.handler.build_select(&sql_target.alias) {
                            result.order_by_clause.push_str(&s);
                         }
                     }
                    //result.order_by_clause.push_str(name);
                    result.order_by_clause.push_str(o);
                    result.order_by_clause.push_str(", ");
                }
            }
        }
        result.order_by_clause = result.order_by_clause.trim_end_matches(", ").to_string();
    }
    fn build_select_clause(
        &self,
        result: &mut MapperResult,
        sql_targets: &HashMap<String, SqlTarget>,
        field_order: &Vec<String>,
    ) {
        // build select clause
        // select all fields
        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                if sql_target.selected || sql_target.options.always_selected {
                    if let Some(sql_field) = sql_target.handler.build_select(&sql_target.alias) {
                        /* if !self.alias.is_empty() && sql_target.options.alias {
                            // joined fields use join alias , other use field alias
                            let al = if sql_target.joined_target {
                                if let Some(n) = field_name.split('_').rev().nth(1) {
                                    &self.joins.get(n).unwrap().alias
                                } else {
                                    panic!("Join not found for field {}", field_name);
                                }
                            } else {
                                &self.alias
                            };

                            result.select_clause.push_str(al);
                            result.select_clause.push_str(".");
                        } */
                      //  result.select_clause.push_str(&sql_target.alias);
                        result.select_clause.push_str(&sql_field);
                    }
                } else {
                    result.select_clause.push_str("null");
                }
                result.select_clause.push_str(", ");
            }
        }

        // Remove last ,
        result.select_clause = result.select_clause.trim_end_matches(", ").to_string();
    }
    fn build_join_clause(&mut self, result: &mut MapperResult) {
        // build select clause
        // select all fields

        for (k, v) in &mut self.fields {
            if v.used && !v.joined {
                let fi = k.split('_').rev().nth(1);
                if let Some(f) = fi {
                    let j = self.joins.get_mut(f).unwrap();
                    if !j.joined {
                        result.join_clause.push_str(&j.join_clause);
                        result.join_clause.push(' ');
                        j.joined = true;
                    }
                }
            }
        }
    }

    pub fn build(&mut self, query: Query, options: BuildOptions) -> MapperResult {
        let mut ordinals: BTreeSet<u8> = BTreeSet::new();
        let mut ordering: HashMap<u8, Vec<(FieldOrder, String)>> = HashMap::new();

        
        if self.dirty {
            self.clean_targets();
            self.clean_joins();
        } else {
            self.alias_fields();  // Alias all fields for selecting, ordering and filtering
        }

        let mut result = MapperResult {
            need_where_concatenation: false,
            need_having_concatenation: false,
            pending_where_parens_concatenation: None,
            pending_having_parens_concatenation: None,
            pending_where_parens: 0,
            pending_having_parens: 0,
            join_clause: String::from(""),
            select_clause: String::from(""),
            where_clause: String::from(""),
            order_by_clause: String::from(""),
            having_clause: String::from(""),
            count_where_clause: String::from(""),
            count_having_clause: String::from(""),
            unused_fields: vec![],
            where_params: vec![],
            having_params: vec![],
        };

        for t in query.tokens {
            {
                match t {
                    QueryToken::LeftBracket(ref concatenation) => {
                        result.pending_where_parens += 1;
                        result.pending_having_parens += 1;
                        result.pending_having_parens_concatenation = Some(concatenation.clone());
                        result.pending_where_parens_concatenation = Some(concatenation.clone());
                    }
                    QueryToken::RightBracket => {
                        if result.pending_where_parens > 0 {
                            result.pending_where_parens -= 1;
                        } else {
                            result.where_clause.push_str(")");
                            result.need_where_concatenation = true;
                        }
                        if result.pending_having_parens > 0 {
                            result.pending_having_parens -= 1;
                        } else {
                            result.having_clause.push_str(")");
                            result.need_having_concatenation = true;
                        }
                    }

                    QueryToken::Wildcard => {
                        for (_, mapper_field) in &mut self.fields {
                            if !mapper_field.options.ignore_wildcard {
                                mapper_field.selected = true; // Select field
                            }
                        }
                    }
                    QueryToken::Field(query_field) => {
                        // Skip field, if name does not start with subpath
                        if !query_field.name.starts_with(&options.subpath) {
                            break;
                        }

                        match self.fields.get_mut(&query_field.name) {
                            Some(sql_target) => {
                                // Verify user role and skip field role mismatches
                                let role_accepted = match (&options.role, &sql_target.options.role)
                                {
                                    (Some(a), Some(b)) => a == b,
                                    (None, None) => true,
                                    _ => false,
                                };
                                if role_accepted == false {
                                    break;
                                }

                                // Skip field if count query should be build and field is not used in count query
                                if options.count_query && !sql_target.options.count_query {
                                    break;
                                }

                                sql_target.selected = !query_field.hidden;
                                sql_target.used = !query_field.hidden;

                                if let Some(f) = query_field.filter {
                                    sql_target.used = true;
                                    if let Some(f) = sql_target.handler.build_filter( &sql_target.alias,&f) {
                                        if query_field.aggregation == true {
                                            if result.need_having_concatenation == true {
                                                if result.pending_having_parens > 0 {
                                                    MapperResult::push_concatenation(
                                                        &mut result.having_clause,
                                                        &result.pending_having_parens_concatenation,
                                                    );
                                                } else {
                                                    MapperResult::push_concatenation(
                                                        &mut result.having_clause,
                                                        &Some(query_field.concatenation),
                                                    );
                                                }
                                            }

                                            MapperResult::push_pending_parens(
                                                &mut result.having_clause,
                                                &result.pending_having_parens,
                                            );

                                            MapperResult::push_filter(
                                                &mut result.having_clause,
                                                &f,
                                            );

                                            result.need_having_concatenation = true;
                                            result.pending_having_parens = 0;
                                        } else {
                                            if result.need_where_concatenation == true {
                                                if result.pending_where_parens > 0 {
                                                    MapperResult::push_concatenation(
                                                        &mut result.where_clause,
                                                        &result.pending_where_parens_concatenation,
                                                    );
                                                } else {
                                                    MapperResult::push_concatenation(
                                                        &mut result.where_clause,
                                                        &Some(query_field.concatenation),
                                                    );
                                                }
                                            }
                                            MapperResult::push_pending_parens(
                                                &mut result.where_clause,
                                                &result.pending_where_parens,
                                            );
                                            MapperResult::push_filter(&mut result.where_clause, &f);

                                            result.pending_where_parens = 0;
                                            result.need_where_concatenation = true;
                                        }
                                    }

                                    if let Some(p) = sql_target.handler.build_param(&f) {
                                        if query_field.aggregation == true {
                                            result.having_params.push(p);
                                        } else {
                                            result.where_params.push(p);
                                        }
                                    }

                                    if let Some(j) = sql_target.handler.build_join() {
                                        result.join_clause.push_str(&j);
                                        result.join_clause.push_str(" ");
                                    }
                                }
                                if let Some(o) = query_field.order {
                                    let num = match o {
                                        FieldOrder::Asc(num) => num,
                                        FieldOrder::Desc(num) => num,
                                    };
                                    ordinals.insert(num);
                                    let l = ordering.entry(num).or_insert(Vec::new());
                                    l.push((o, query_field.name));
                                }
                            }
                            None => result.unused_fields.push(query_field.name),
                        }
                    }
                }
            }
        }

       

        Self::build_ordering(&mut result, &self.fields, &ordinals, &ordering);
        self.build_select_clause(&mut result, &self.fields, &self.field_order);
        self.build_join_clause(&mut result);

        // If where is empty add true
        if result.where_clause.is_empty() {
            result.where_clause = "true".to_string();
        }

        // Remove trailing white space on joins
        result.join_clause.trim_end();
        result.order_by_clause.trim_end();

        self.dirty= true;

        result
    }
}
