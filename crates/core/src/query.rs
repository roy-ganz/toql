//!
//! This module contains the query builder to build a Toql query programatically.
//!
//! ## Example
//!
//! ```ignore
//! use toql::query::{Query, Field};
//!
//! let  q = Query::new()
//!        .and(Field::from("foo").hide().eq(5).aggregate().asc(1))
//!        .and(Field::from("bar").desc(2));
//!    assert_eq!("+1.foo !EQ 5,-2bar", q.to_string());
//! ```
//!
//! To avoid typing mistakes the Toql derive builds functions for all fields in a struct.
//!
//! In the example above it would be possible to write
//! `.and(Foobar::fields().bar().desc(2)` for a derived struct `Foobar`.
//!
//! Read the guide for more information on the query syntax or see (Query)[struct.Query.html]
//!

pub mod concatenation;
pub mod field;
pub mod field_filter;
pub mod field_order;
pub mod field_path;
pub mod predicate;
pub mod query_with;
pub mod selection;
pub mod wildcard;

use crate::query::selection::Selection;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

use concatenation::Concatenation;
use field::Field;
use predicate::Predicate;
use query_with::QueryWith;
use wildcard::Wildcard;

#[derive(Clone, Debug)]
pub(crate) enum QueryToken {
    LeftBracket(Concatenation),
    RightBracket,
    Wildcard(Wildcard),
    Field(Field),
    Predicate(Predicate),
    Selection(Selection),
}

impl From<&str> for QueryToken {
    fn from(s: &str) -> QueryToken {
        if s.ends_with('*') {
            QueryToken::Wildcard(Wildcard::from(s))
        } else {
            QueryToken::Field(Field::from(s))
        }
    }
}

impl ToString for QueryToken {
    fn to_string(&self) -> String {
        match self {
            QueryToken::RightBracket => String::from(")"),
            QueryToken::LeftBracket(c) => match c {
                Concatenation::And => String::from("("),
                Concatenation::Or => String::from("("),
            },
            QueryToken::Field(field) => field.to_string(),
            QueryToken::Predicate(predicate) => predicate.to_string(),
            QueryToken::Selection(selection) => selection.name.to_string(),
            QueryToken::Wildcard(wildcard) => format!("{}*", wildcard.path),
        }
    }
}

/// A Query allows to create a Toql query programmatically or modify a parsed string query.
///
/// This is faster than the [QueryParser](../query_parser/struct.QueryParser.html) and cannot fail.
/// It should be used whenever possible.
///
/// A query can be turned into SQL using the [SQL Builder](../sql_builder/struct.SqlBuilder.html).
///
/// To build a big query simply add fields, wildcards ans other (sub)querries with [and()](struct.Query.html#method.and) resp. [or()](struct.Query.html#method.or) function.
///
/// Watch out: Logical AND has precendence over OR. So `a OR b AND c` is the same as `a OR (b AND c)`.
///
/// **Always parenthesize a user query if you add a permission filter to it**.
///
/// Malicious users will try circumvent your permission filter with a simple OR clause at the beginning.
/// Compare an evil query with a safe one:
///
/// Evil: `(*, id nen); id, permission ne ""`
///
/// Safe: `((*, id nen); id), permission ne ""`.
///
/// In the evil query, the permission is ignored, because the predicate `id nen` is always true and returns all records.
///
/// To parenthesize a query use the [parenthesize()](struct.Query.html#method.parenthesize) method.
///
/// ``` ignore
/// let q1 = Query::new().and(Field("b").eq(3)).and(Field("c").eq(2));
/// let q2 = Query::new().and(Field("a").eq(1)).or(q1.parens());
///
/// assert_eq!("a eq 1; (b eq 3, c eq 2)", q2.to_string())
/// ```
///
/// For every fields of a struct the Toql derive generates fields.
/// For a Toql derived struct it's possible to write
///
/// ``` ignore
/// let q1 = Query::wildcard().and(User::fields().addresses().street().eq("miller street")).and(UserKey(10));
/// ```
///
/// To modify q query with a custom struct see [QueryWith](struct.QueryWith.html)
///
///
///
use crate::sql_arg::SqlArg;
#[derive(Debug)]
pub struct Query<M> {
    pub(crate) tokens: Vec<QueryToken>,
    /// Select DISTINCT
    pub distinct: bool,
    /* /// Roles a query has to access fields.
    /// See [MapperOption](../table_mapper/struct.MapperOptions.html#method.restrict_roles) for explanation.
    pub roles: HashSet<String>, */
    pub aux_params: HashMap<String, SqlArg>, // generic build params

    pub where_predicates: Vec<String>, // Additional where clause
    pub where_predicate_params: Vec<SqlArg>, // Query params for additional sql restriction
    pub select_columns: Vec<String>,   // Additional select columns

    pub join_stmts: Vec<String>,       // Additional joins statements
    pub join_stmt_params: Vec<SqlArg>, // Join params for additional sql restriction
    // pub wildcard_scope: Option<HashSet<String>> // Restrict wildcard to certain fields
    pub type_marker: std::marker::PhantomData<M>,
}

impl<M> Query<M> {
    /// Create a new empty query.
    pub fn new() -> Self {
        Query::<M> {
            tokens: vec![],
            distinct: false,
            // roles: HashSet::new(),
            aux_params: HashMap::new(),
            where_predicates: Vec::new(),
            where_predicate_params: Vec::new(),
            select_columns: Vec::new(),
            join_stmts: Vec::new(),
            join_stmt_params: Vec::new(),
            type_marker: std::marker::PhantomData, //  wildcard_scope: None
        }
    }

    /// Create a new query from another query.
    pub fn from<T>(query: T) -> Self
    where
        T: Into<Query<M>>,
    {
        query.into()
    }

    /// Create a new query from another query.
    pub fn clone_for_type<T>(&self) -> Query<T> {
        Query::<T> {
            tokens: self.tokens.clone(),
            distinct: self.distinct,
            aux_params: self.aux_params.clone(),
            where_predicates: self.where_predicates.clone(),
            where_predicate_params: self.where_predicate_params.clone(),
            select_columns: self.select_columns.clone(),
            join_stmts: self.join_stmts.clone(),
            join_stmt_params: self.join_stmt_params.clone(),
            type_marker: std::marker::PhantomData,
        }
    }
    /// Create a new query from the path of another query.
    pub fn traverse<T>(&self, path: &str) -> Query<T> {
        let tokens = self
            .tokens
            .iter()
            .filter_map(|t| match t {
                QueryToken::Field(field) => {
                    if field.name.starts_with(path) {
                        let mut field = field.clone();
                        field.name = field
                            .name
                            .trim_start_matches(path)
                            .trim_start_matches('_')
                            .to_string();
                        Some(QueryToken::Field(field))
                    } else {
                        None
                    }
                }
                QueryToken::Wildcard(wildcard) => {
                    if wildcard.path.starts_with(path) {
                        let mut wildcard = wildcard.clone();
                        wildcard.path = wildcard
                            .path
                            .trim_start_matches(path)
                            .trim_start_matches('_')
                            .to_string();
                        Some(QueryToken::Wildcard(wildcard))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        Query::<T> {
            tokens,
            distinct: self.distinct,
            aux_params: self.aux_params.clone(),
            where_predicates: self.where_predicates.clone(),
            where_predicate_params: self.where_predicate_params.clone(),
            select_columns: self.select_columns.clone(),
            join_stmts: self.join_stmts.clone(),
            join_stmt_params: self.join_stmt_params.clone(),
            type_marker: std::marker::PhantomData,
        }
    }

    /// Create a new query that select all top fields.
    pub fn wildcard() -> Self {
        Query::<M> {
            tokens: vec![QueryToken::Wildcard(Wildcard::new())],
            distinct: false,
            //roles: HashSet::new(),
            aux_params: HashMap::new(),
            where_predicates: Vec::new(),
            where_predicate_params: Vec::new(),
            select_columns: Vec::new(),
            join_stmts: Vec::new(),
            join_stmt_params: Vec::new(),
            type_marker: std::marker::PhantomData, //  wildcard_scope: None
        }
    }

    /// Wrap query with parentheses.
    pub fn parenthesize(mut self) -> Self {
        if self.tokens.is_empty() {
            return self;
        }
        self.tokens
            .insert(0, QueryToken::LeftBracket(Concatenation::And));
        self.tokens.push(QueryToken::RightBracket);
        self
    }
    /// Concatenate field or query with AND.
    pub fn and<T>(mut self, query: T) -> Self
    where
        T: Into<Query<M>>,
    {
        // All tokens are by default concatenated with AND
        self.tokens.append(&mut query.into().tokens);
        self
    }
    /// Concatenate field or query with AND.
    pub fn and_parentized<T>(mut self, query: T) -> Self
    where
        T: Into<Query<M>>,
    {
        self.tokens
            .push(QueryToken::LeftBracket(Concatenation::And));
        self.tokens.append(&mut query.into().tokens);
        self.tokens.push(QueryToken::RightBracket);
        self
    }

    /// Concatenate field or query with OR.
    pub fn or<T>(mut self, query: T) -> Self
    where
        T: Into<Query<M>>,
    {
        // Change first token of query to concatenate with OR
        let mut query = query.into();
        match query.tokens.get_mut(0) {
            Some(QueryToken::LeftBracket(c)) => *c = Concatenation::Or,
            Some(QueryToken::RightBracket) => {}
            Some(QueryToken::Field(f)) => f.concatenation = Concatenation::Or,
            Some(QueryToken::Wildcard(w)) => w.concatenation = Concatenation::Or,
            Some(QueryToken::Predicate(p)) => p.concatenation = Concatenation::Or,
            Some(QueryToken::Selection(p)) => p.concatenation = Concatenation::Or,
            None => {}
        }
        /*         if let QueryToken::LeftBracket(c) = query.tokens.get_mut(0).unwrap() {
                   *c = Concatenation::Or;
               } else if let QueryToken::Field(field) = query.tokens.get_mut(0).unwrap() {
                   field.concatenation = Concatenation::Or;
               } else if let QueryToken::Wildcard(wildcard) = query.tokens.get_mut(0).unwrap() {
                   wildcard.concatenation = Concatenation::Or;
               }
        */

        self.tokens.append(&mut query.tokens);

        self
    }
    /// Concatenate field or query with AND.
    pub fn or_parentized<T>(mut self, query: T) -> Self
    where
        T: Into<Query<M>>,
    {
        self.tokens.push(QueryToken::LeftBracket(Concatenation::Or));
        self.tokens.append(&mut query.into().tokens);
        self.tokens.push(QueryToken::RightBracket);
        self
    }

    /// Modifiy the query with an additional stuct.
    pub fn with(self, query_with: impl QueryWith<M>) -> Self {
        query_with.with(self)
    }

    /// Convenence method to add aux params
    pub fn aux_param<S, A>(mut self, name: S, value: A) -> Self
    where
        A: Into<SqlArg>,
        S: Into<String>,
    {
        self.aux_params.insert(name.into(), value.into());
        self
    }

    /// Check if query contains path
    /// Example: Path is 'user_address'
    /// Valid query paths are 'user_*', 'user_address_*', 'user_address_country_*,'user_address_id'
    pub fn contains_path(&self, path: &str) -> bool {
        let p = format!("{}_", path.trim_end_matches('_')); // ensure path ends with _
        self.tokens.iter().any(|t| {
            let pth = p.as_str();
            match t {
                QueryToken::Field(field) => field.name.starts_with(pth),
                QueryToken::Wildcard(wildcard) => {
                    path.starts_with(&wildcard.path) || wildcard.path.starts_with(pth)
                }
                _ => false,
            }
        })
    }
    /// Check if query contains path starting with a subpath
    /// Example: Path is 'user_address'
    /// Valid query paths are 'user_address_*', 'user_address_id'
    pub fn contains_path_starts_with(&self, path: &str) -> bool {
        let p = format!("{}_", path.trim_end_matches('_')); // ensure path ends with _
        self.tokens.iter().any(|t| {
            let pth = p.as_str();
            match t {
                QueryToken::Field(field) => field.name.starts_with(pth),
                QueryToken::Wildcard(wildcard) => wildcard.path.starts_with(pth),
                _ => false,
            }
        })
    }
}

/// Asserts that the provided roles contains all required roles.
/// The first missing role is returned as error.
pub fn assert_roles(
    provided_roles: &HashSet<String>,
    required_roles: &HashSet<String>,
) -> Result<(), String> {
    for r in required_roles {
        if !provided_roles.contains(r) {
            return Err(r.to_owned());
        }
    }

    Ok(())
}

// Doc: Display  implements automatically .to_string()
impl<M> fmt::Display for Query<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn get_concatenation(c: &Concatenation) -> char {
            match c {
                Concatenation::And => ',',
                Concatenation::Or => ';',
            }
        }

        let mut s = String::new();
        let mut concatenation_needed = false;

        for token in &self.tokens {
            if concatenation_needed {
                match &token {
                    QueryToken::LeftBracket(concatenation) => {
                        s.push(get_concatenation(concatenation))
                    }
                    QueryToken::Wildcard(wildcard) => {
                        s.push(get_concatenation(&wildcard.concatenation))
                    }
                    QueryToken::Field(field) => s.push(get_concatenation(&field.concatenation)),
                    QueryToken::Predicate(field) => s.push(get_concatenation(&field.concatenation)),
                    _ => {}
                }
            }
            s.push_str(&token.to_string());
            match token {
                QueryToken::LeftBracket(..) => concatenation_needed = false,
                QueryToken::Field(..) => concatenation_needed = true,
                QueryToken::Wildcard(..) => concatenation_needed = true,
                QueryToken::Predicate(..) => concatenation_needed = true,
                _ => {}
            }
        }

        // To avoid allocation check first if string starts with a separator
        let t = s.chars().next().unwrap_or(' ');
        if t == ',' || t == ';' {
            s = s.trim_start_matches(',').trim_start_matches(';').to_owned();
        }

        write!(f, "{}", s)
    }
}

impl<M> From<Field> for Query<M> {
    fn from(field: Field) -> Query<M> {
        let mut q = Query::new();
        q.tokens.push(QueryToken::Field(field));
        q
    }
}

impl<M> From<Predicate> for Query<M> {
    fn from(predicate: Predicate) -> Query<M> {
        let mut q = Query::new();
        q.tokens.push(QueryToken::Predicate(predicate));
        q
    }
}
impl<M> From<Selection> for Query<M> {
    fn from(selection: Selection) -> Query<M> {
        let mut q = Query::new();
        q.tokens.push(QueryToken::Selection(selection));
        q
    }
}

impl<M> From<Wildcard> for Query<M> {
    fn from(wildcard: Wildcard) -> Query<M> {
        let mut q = Query::new();
        q.tokens.push(QueryToken::Wildcard(wildcard));
        q
    }
}

impl<M> From<&str> for Query<M> {
    fn from(string: &str) -> Query<M> {
        let mut q = Query::new();
        q.tokens.push(if string.ends_with('*') {
            QueryToken::Wildcard(Wildcard::from(string))
        } else {
            QueryToken::Field(Field::from(string))
        });
        q
    }
}

impl<M> Default for Query<M> {
    fn default() -> Self {
        Self::new()
    }
}
