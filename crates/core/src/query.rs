//! The [Query] represents a Toql query but also comes with query builder methods.
//!
//! While it's perfectly possible to use the query builder directly
//! it is recommended to use the `query!` macro. However sometimes
//! mixed uses of the macro and the builder may make sense.
//!
//! ## Example
//!
//! ```ignore
//! use toql::prelude::{Query, Field};
//!
//! let  q = Query::<FooBar>::new()
//!        .and(Field::from("foo").hide().eq(5).asc(1))
//!        .and(Field::from("bar").desc(2));
//!    assert_eq!("+1.foo EQ 5,-2bar", q.to_string());
//! ```
//! The above code generated with the [query!](toql_query_macro/macro.query) macro
//! ```ignore
//! use toql::prelude::{Query, Field};
//!
//! let q :Query<FooBar>= query!(FooBar, "+1.foo EQ 5, -2bar");
//! assert_eq!("+1.foo EQ 5,-2bar", q.to_string());
//! ```
//! The query macro produces a [Query] type, so the result can
//! modified with builder functions.
pub mod concatenation;
pub mod field;
pub mod field_filter;
pub mod field_order;
pub mod field_path;
pub mod from_key_fields;
pub mod predicate;
pub mod query_token;
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
use query_token::QueryToken;
use query_with::QueryWith;
use wildcard::Wildcard;

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

    /// Aux params used with query
    pub aux_params: HashMap<String, SqlArg>, // generic build params

    /// Additional where clause
    pub where_predicates: Vec<String>,

    /// Query params for additional sql restriction
    pub where_predicate_params: Vec<SqlArg>,

    /// Additional select columns
    pub select_columns: Vec<String>,

    /// Additional joins statements
    pub join_stmts: Vec<String>,

    // Join params for additional sql restriction
    pub join_stmt_params: Vec<SqlArg>,

    /// Type marker
    pub type_marker: std::marker::PhantomData<M>,
}

impl<M> Query<M> {
    /// Create a new empty query.
    pub fn new() -> Self {
        Query::<M> {
            tokens: vec![],
            distinct: false,
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
    /// Create a new query for a home path.
    pub fn traverse<T>(&self, home_path: &str) -> Query<T> {
        let tokens = self
            .tokens
            .iter()
            .filter_map(|t| match t {
                QueryToken::Field(field) => {
                    if field.name.starts_with(home_path) {
                        let mut field = field.clone();
                        field.name = field
                            .name
                            .trim_start_matches(home_path)
                            .trim_start_matches('_')
                            .to_string();
                        Some(QueryToken::Field(field))
                    } else {
                        None
                    }
                }
                QueryToken::Wildcard(wildcard) => {
                    if wildcard.path.starts_with(home_path) {
                        let mut wildcard = wildcard.clone();
                        wildcard.path = wildcard
                            .path
                            .trim_start_matches(home_path)
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
    pub fn selection(name: impl Into<String>) -> Self {
        Query::<M> {
            tokens: vec![QueryToken::Selection(Selection::from(name.into()))],
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
        if !self.tokens.is_empty() {
            match query.tokens.get_mut(0) {
                Some(QueryToken::LeftBracket(c)) => *c = Concatenation::Or,
                Some(QueryToken::RightBracket) => {}
                Some(QueryToken::Field(f)) => f.concatenation = Concatenation::Or,
                Some(QueryToken::Wildcard(w)) => w.concatenation = Concatenation::Or,
                Some(QueryToken::Predicate(p)) => p.concatenation = Concatenation::Or,
                Some(QueryToken::Selection(p)) => p.concatenation = Concatenation::Or,
                None => {}
            }
        }

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
                    QueryToken::Selection(selection) => {
                        s.push(get_concatenation(&selection.concatenation))
                    }
                    _ => {}
                }
            }
            s.push_str(&token.to_string());
            match token {
                QueryToken::LeftBracket(..) => concatenation_needed = false,
                QueryToken::Field(..) => concatenation_needed = true,
                QueryToken::Wildcard(..) => concatenation_needed = true,
                QueryToken::Predicate(..) => concatenation_needed = true,
                QueryToken::Selection(..) => concatenation_needed = true,
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

#[cfg(test)]
mod test {
    use super::{query_with::QueryWith, Field, Predicate, Query, Selection, Wildcard};
    use crate::sql_arg::SqlArg;

    struct User;
    struct Level2;

    struct Item;

    impl<T> QueryWith<T> for Item {
        fn with(&self, query: Query<T>) -> Query<T> {
            query
                .and(Field::from("item").eq(1))
                .aux_param("thing", "item")
        }
    }

    #[test]
    fn build() {
        assert_eq!(Query::<User>::default().to_string(), "");
        assert_eq!(Query::<User>::new().to_string(), "");

        let qf = Query::<User>::from(Field::from("prop"));
        let qp = Query::<User>::from(Predicate::from("pred"));
        let qs = Query::<User>::from(Selection::from("std"));
        let qw = Query::<User>::from(Wildcard::from("level1"));

        assert_eq!(qf.to_string(), "prop");
        assert_eq!(qp.to_string(), "@pred");
        assert_eq!(qs.to_string(), "$std");
        assert_eq!(qw.to_string(), "level1_*");

        assert_eq!(
            qf.clone_for_type::<User>()
                .and(qp.clone_for_type())
                .to_string(),
            "prop,@pred"
        );
        assert_eq!(
            qf.clone_for_type::<User>()
                .and_parentized(qp.clone_for_type())
                .to_string(),
            "prop,(@pred)"
        );
        assert_eq!(
            qf.clone_for_type::<User>()
                .or(qp.clone_for_type())
                .to_string(),
            "prop;@pred"
        );
        assert_eq!(
            qf.clone_for_type::<User>()
                .or_parentized(qp.clone_for_type())
                .to_string(),
            "prop;(@pred)"
        );

        let q: Query<User> = Query::from(Field::from("level2_prop2"))
            .and(Query::from(Wildcard::from("level2")))
            .and(Query::from(Field::from("level3_prop4")))
            .and(Query::from(Field::from("level1_prop1")))
            .and(Query::from(Field::from("prop")));
        assert_eq!(qw.contains_path("level1"), true);
        assert_eq!(qw.contains_path("level4"), false);
        assert_eq!(q.traverse::<Level2>("level2").to_string(), "prop2,*");

        let q = qf.with(Item);
        assert_eq!(q.to_string(), "prop,item EQ 1");
        assert_eq!(q.aux_params.get("thing"), Some(&SqlArg::from("item")));
    }
}
