use crate::query::FieldFilter;
use std::collections::BTreeSet;
use std::collections::HashMap;

#[derive(Debug)]
#[allow(dead_code)] // IMPROVE Having AND None are considered unused
pub(crate) enum FilterType {
    Where,
    Having,
    None,
}

#[derive(Debug)]
pub struct SqlTarget {
    pub(crate) options: MapperOptions,                   // Options
    pub(crate) filter_type: FilterType,                  // Filter on where or having clause
    pub(crate) handler: Box<FieldHandler + Send + Sync>, // Handler to create clauses
    pub(crate) subfields: bool,                          // Target name has subfields separated by underscore
    pub(crate) expression: String,                       // Column name or SQL expression
}

#[derive(Debug)]
pub struct SqlField {}

#[derive(Debug)]
pub struct MapperOptions {
    pub(crate) always_selected: bool,   // Always select this field, regardless of query fields
    pub(crate) count_filter: bool,      // Filter field on count query
    pub(crate) count_select: bool,      // Select field on count query
    pub(crate) ignore_wildcard: bool,   // Ignore field for wildcard selection
    pub(crate) roles: BTreeSet<String>, // Only for use by these roles
}

// OPT use references
impl MapperOptions {
    pub fn new() -> Self {
        MapperOptions {
            always_selected: false,
            count_filter: false,
            count_select: false,
            ignore_wildcard: false,
            roles: BTreeSet::new(),
        }
    }
    pub fn select_always(mut self, always_selected: bool) -> Self {
        self.always_selected = always_selected;
        self
    }

    pub fn count_filter(mut self, count_filter: bool) -> Self {
        self.count_filter = count_filter;
        self
    }
    pub fn count_select(mut self, count_select: bool) -> Self {
        self.count_select = count_select;
        self
    }
    pub fn ignore_wildcard(mut self, ignore_wildcard: bool) -> Self {
        self.ignore_wildcard = ignore_wildcard;
        self
    }
    pub fn restrict_roles(mut self, roles: BTreeSet<String>) -> Self {
        self.roles = roles;
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
    fn build_select(&self, sql: &str) -> Option<String>;
    fn build_filter(&self, sql: &str, _filter: &FieldFilter) -> Option<String>;
    fn build_param(&self, _filter: &FieldFilter) -> Vec<String>;
    fn build_join(&self) -> Option<String> {
        None
    }
}

impl std::fmt::Debug for (dyn FieldHandler + std::marker::Send + std::marker::Sync + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FieldHandler()")
    }
}



impl FieldHandler for SqlField {
    fn build_select(&self, expression: &str) -> Option<String> {
        Some(format!("{}", expression))
    }

    fn build_param(&self, filter: &FieldFilter) -> Vec<String> {
        match filter {
            FieldFilter::Eq(criteria) => vec![criteria.clone()],
            FieldFilter::Eqn => vec![],
            FieldFilter::Ne(criteria) => vec![criteria.clone()],
            FieldFilter::Nen => vec![],
            FieldFilter::Ge(criteria) => vec![criteria.clone()],
            FieldFilter::Gt(criteria) => vec![criteria.clone()],
            FieldFilter::Le(criteria) => vec![criteria.clone()],
            FieldFilter::Lt(criteria) => vec![criteria.clone()],
            FieldFilter::Bw(lower, upper) => vec![lower.clone(), upper.clone()],
            FieldFilter::Re(criteria) => vec![criteria.clone()],
            FieldFilter::Sc(criteria) => vec![criteria.clone()],
            FieldFilter::In(args) => args.clone(),
            FieldFilter::Out(args) => args.clone(),
            FieldFilter::Lk(criteria) => vec![criteria.clone()],
            FieldFilter::Fn(_name, _args) => vec![], // must be implemented by user
        }
    }

    fn build_filter(&self, expression: &str, filter: &FieldFilter) -> Option<String> {
        match filter {
            FieldFilter::Eq(_) => Some(format!("{} = ?", expression)),
            FieldFilter::Eqn => Some(format!("{} IS NULL", expression)),
            FieldFilter::Ne(_) => Some(format!("{} <> ?", expression)),
            FieldFilter::Nen => Some(format!("{} IS NOT NULL", expression)),
            FieldFilter::Ge(_) => Some(format!("{} >= ?", expression)),
            FieldFilter::Gt(_) => Some(format!("{} > ?", expression)),
            FieldFilter::Le(_) => Some(format!("{} <= ?", expression)),
            FieldFilter::Lt(_) => Some(format!("{} < ?", expression)),
            FieldFilter::Bw(_, _) => Some(format!("{} BETWEEN ? AND ?", expression)),
            FieldFilter::Re(_) => Some(format!("{} RLIKE ?", expression)),
            FieldFilter::In(values) => Some(format!(
                "{} IN ({})",
                expression,
                std::iter::repeat("?")
                    .take(values.len())
                    .collect::<Vec<&str>>()
                    .join(",")
            )),
            FieldFilter::Out(values) => Some(format!(
                "{} NOT IN ({})",
                expression,
                std::iter::repeat("?")
                    .take(values.len())
                    .collect::<Vec<&str>>()
                    .join(",")
            )),
            FieldFilter::Sc(_) => Some(format!("FIND_IN_SET (?, {})", expression)),
            FieldFilter::Lk(_) => Some(format!("{} LIKE ?", expression)),
            FieldFilter::Fn(_, _) => None, // Must be implemented by user
        }
    }
}

pub type SqlMapperCache = HashMap<String, SqlMapper>;

#[derive(Debug)]
pub struct SqlMapper {
    pub(crate) table: String,
    pub(crate) field_order: Vec<String>,
    pub(crate) fields: HashMap<String, SqlTarget>,
    pub(crate) joins: HashMap<String, Join>,
}

#[derive(Debug)]
pub struct Join {
    pub(crate) join_clause: String,
}

pub trait Mappable {
    fn insert_new_mapper(cache: &mut SqlMapperCache) -> &mut SqlMapper;     // Create new SQL Mapper and insert into mapper cache
    fn new_mapper(sql_alias: &str) -> SqlMapper;                            // Create new SQL Mapper and map entity fields
    fn map(mapper: &mut SqlMapper, toql_path: &str, sql_alias: &str);       // Map entity fields
}

impl SqlMapper {
    pub fn new<T>(table: T) -> Self
    where
        T: Into<String>,
    {
        SqlMapper {
            table: table.into(),
            joins: HashMap::new(),
            fields: HashMap::new(),
            field_order: Vec::new(),
        }
    }
    pub fn insert_new_mapper<T: Mappable>(cache: &mut SqlMapperCache) -> &mut SqlMapper {
        T::insert_new_mapper(cache)
    }
    pub fn map<T: Mappable>(sql_alias: &str) -> Self {
        // Mappable must create mapper for top level table
        T::new_mapper(sql_alias)
    }
    pub fn map_join<'a, T: Mappable>(
        &'a mut self,
        toql_path: &str,
        sql_alias: &str,
    ) -> &'a mut Self {
        T::map(self, toql_path, sql_alias);
        self
    }

    pub fn map_handler<'a>(
        &'a mut self,
        toql_field: &str,
        expression: &str,
        handler: Box<FieldHandler + Sync + Send>,
        options: MapperOptions,
    ) -> &'a mut Self {
        let t = SqlTarget {
            options: options,
            filter_type: FilterType::Where, // Filter on where clause
            subfields: toql_field.find('_').is_some(),
            handler: handler,
            expression: expression.to_string(),
        };
        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }
    pub fn alter_handler(
        &mut self,
        toql_field: &str,
        handler: Box<FieldHandler + Sync + Send>,
    ) -> &mut Self {
        let sql_target = self.fields.get_mut(toql_field).expect(&format!(
            "Cannot alter \"{}\": Field is not mapped.",
            toql_field
        ));

        sql_target.handler = handler;
        self
    }

    pub fn alter_handler_with_options(
        &mut self,
        toql_field: &str,
        handler: Box<FieldHandler + Sync + Send>,
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

    pub fn map_field<'a>(&'a mut self, toql_field: &str, sql_field: &str) -> &'a mut Self {
        self.map_field_with_options(toql_field, sql_field, MapperOptions::new())
    }

    pub fn map_field_with_options<'a>(
        &'a mut self,
        toql_field: &str,
        sql_expression: &str,
        options: MapperOptions,
    ) -> &'a mut Self {
        let f = SqlField {};

        let t = SqlTarget {
            expression: sql_expression.to_string(),
            options: options,
            filter_type: FilterType::Where, // Filter on where clause
            subfields: toql_field.find('_').is_some(),
            handler: Box::new(f),
        };

        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }

    pub fn join<'a>(&'a mut self, toql_field: &str, join_clause: &str) -> &'a mut Self {
        self.joins.insert(
            toql_field.to_string(),
            Join {
                join_clause: join_clause.to_string(),
            },
        );

        // Find targets that use join and set join field

        self
    }

    pub fn alter_join<'a>(&'a mut self, toql_field: &str, join_clause: &str) -> &'a mut Self {
        let j = self.joins.get_mut(toql_field).expect("Join is missing.");
        j.join_clause = join_clause.to_string();
        self
    }
}
