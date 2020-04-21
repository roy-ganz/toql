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
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

/// A trait to convert any structure into a Query. 
/// For emxaple implement this for your configuration
/// and you can do `Query::new().with(config)`
pub trait QueryWith {
    fn with<T>(&self, query: Query<T>) -> Query<T>;
}

// An trait to turn entity keys into a query perdicate (used by toql derive)
pub trait QueryPredicate<T> {
    fn predicate(self, path: &str) -> Query<T>;
}

/// A trait to convert a simple datatype into a filter argument. Used by builder functions. Not very interesting ;)
pub trait PredicateArg {
    fn to_args(self) -> Vec<SqlArg>;
}

/* impl PredicateArg for String {
    fn to_args (self) -> Vec<String> {
        vec![self]
    }
}   
impl PredicateArg for &str {
    fn to_args (self) -> Vec<String> {
        vec![self.to_owned()]
    }
} 
impl PredicateArg for &[&str] {
    fn to_args (self) -> Vec<String> {
        let mut args = Vec::new();
        for a in self {
            args.push(a.to_string());
        }
        args
    }
}   */
/* 

macro_rules! impl_num_predicate_arg {
    ($($mty:ty),+) => {
        $(
            impl PredicateArg for $mty {
                 fn to_args(self) -> Vec<String> {
                    vec![self.to_string()]
                 }
            }
            impl<'a> PredicateArg for &'a $mty {
                 fn to_args(self) -> Vec<String> {
                    vec![self.to_string()]
                 }
            }
        )+
    }
} */

//impl_num_predicate_arg!(usize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);



/// A trait to convert a simple datatype into a filter argument. Used by builder functions. Not very interesting ;)
pub trait FilterArg {
    fn to_sql(&self) -> String;
}

impl FilterArg for &str {
    fn to_sql(&self) -> String {
         self.to_string()
        /* let mut s = String::from("'");
        // TODO escape for sql
        s.push_str(*self);
        s.push('\'');
        s */
    }
}
// TODO combine with above
impl FilterArg for String {
    fn to_sql(&self) -> String {
         self.to_string()
      /*   let mut s = String::from("'");
        // TODO escape for sql
        s.push_str(self);
        s.push('\'');
        s */
    }
}
impl FilterArg for &String {
    fn to_sql(&self) -> String {
        self.to_string()
       /*  let mut s = String::from("'");
        // TODO escape for sql
        s.push_str(self.as_str());
        s.push('\'');
        s */
    }
}

macro_rules! impl_num_filter_arg {
    ($($mty:ty),+) => {
        $(
            impl FilterArg for $mty {
                 fn to_sql(&self) -> String {
                    self.to_string()
                 }
            }
            impl<'a> FilterArg for &'a $mty {
                 fn to_sql(&self) -> String {
                    self.to_string()
                 }
            }
        )+
    }
}

impl_num_filter_arg!(usize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

 impl<U, T: Into<Query<U>>> Into<Query<U>> for Vec<T> {
    fn into(self) -> Query<U> {
        let mut query = Query::<U>::new();
        for key in self {
            query = query.or(key);
        }
        query
    }
}

impl<U,T: Into<Query<U>> + Clone> Into<Query<U>> for &Vec<T> {
    fn into(self) -> Query<U> {
        let mut query = Query::<U>::new();
        for key in self {
            query = query.or(key.clone());
        }
        query
    }
} 
impl<U,T: Into<Query<U>> + Clone> Into<Query<U>> for &[T] {
    fn into(self) -> Query<U> {
        let mut query = Query::<U>::new();
        for key in self {
            query = query.or(key.clone());
        }
        query
    }
} 

impl FilterArg for bool {
    fn to_sql(&self) -> String {
        String::from(if *self == true { "1" } else { "0" })
    }
}

impl FilterArg for &bool {
    fn to_sql(&self) -> String {
        String::from(if **self == true { "1" } else { "0" })
    }
}

#[derive(Clone, Debug)]
pub enum Concatenation {
    And,
    Or,
}

/// A wildcard is used to select all fields from top or from a path.
///
/// Example
/// ```ignore
///
///  let q = Query::new().and(Wildcard::new()).and(Wildcard::from("bar")); // more elegant -> Query::wildcard().and(...)
///
///  assert_eq!("*, bar_*", q.to_string());
/// ```
/// Note that the Toql derive builds a wildcard function too.
/// If a struct `Foo` contained a struct `Bar`, it would be possible to replace the second call to _and()_ with  `.and(Bar::fields().bar().wildcard())`
#[derive(Clone, Debug)]
pub struct Wildcard {
    pub(crate) concatenation: Concatenation,
    pub(crate) path: String,
}

impl Wildcard {
    /// Creates a new wildcard to select all fields from top
    pub fn new() -> Self {
        Wildcard {
            concatenation: Concatenation::And,
            path: String::from(""),
        }
    }
    /// Creates a new wildcard to select all fields from a path
    pub fn from<T>(path: T) -> Self
    where
        T: Into<String>,
    {
        let mut path = path.into();
        #[cfg(debug)]
        {
            if !path.chars().all(|x| x.is_alphanumeric() || x == '_') {
                panic!(
                    "Path {:?} must only contain alphanumeric characters and underscores.",
                    path
                );
            }
        }
        // Remove optional trainling *
        if path.ends_with("*") {
            path.pop();
        }
        // Add _ at end if missing
        if !path.ends_with("_") {
            path.push('_');
        }

        Wildcard {
            concatenation: Concatenation::And,
            path: path.into(),
        }
    }
}

/// A Toql field can select, filter and order a database column or expression
/// A field can be created from a field name and filtered, sorted with its methods.
/// However the Toql derive creates fields structs for a derived struct, so instead of
/// ``` ignore
///  
///  let f = Field::from("id");
/// ```
/// its easier and recommended to write
/// ``` ignore
///  let f = User::fields().id();
/// ```
#[derive(Clone, Debug)]
pub struct Field {
    pub(crate) concatenation: Concatenation,
    pub(crate) name: String,
    pub(crate) hidden: bool,
    pub(crate) order: Option<FieldOrder>,
    pub(crate) filter: Option<FieldFilter>,
    pub(crate) aggregation: bool,
}

impl Field {
    /// Create a field for the given name.
    pub fn from<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        let name = name.into();
        #[cfg(debug)]
        {
            // Ensure name does not end with wildcard
            if name.ends_with("*") {
                panic!("Fieldname {:?} must not end with wildcard.", name);
            }
        }

        Field {
            concatenation: Concatenation::And,
            name: name.into(),
            hidden: false,
            order: None,
            filter: None,
            aggregation: false,
        }
    }
    /// Hide field. Useful if a field should not be selected, but be used for filtering.
    pub fn hide(mut self) -> Self {
        self.hidden = true;
        self
    }
    /// Aggregate a field to make the filter be in SQL HAVING clause instead of WHERE clause
    pub fn aggregate(mut self) -> Self {
        self.aggregation = true;
        self
    }
    /// Use this field to order records in ascending way. Give ordering priority when records are ordered by multiple fields.
    pub fn asc(mut self, order: u8) -> Self {
        self.order = Some(FieldOrder::Asc(order));
        self
    }
    /// Use this field to order records in descending way. Give ordering priority when records are ordered by multiple fields.
    pub fn desc(mut self, order: u8) -> Self {
        self.order = Some(FieldOrder::Desc(order));
        self
    }
    /// Filter records with _equal_ predicate.
    pub fn eq(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Eq(criteria.into()));
        self
    }
    /// Filter records with _equal null_ predicate.
    pub fn eqn(mut self) -> Self {
        self.filter = Some(FieldFilter::Eqn);
        self
    }
    /// Filter records with _not equal_ predicate.
    pub fn ne(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Ne(criteria.into()));
        self
    }
    /// Filter records with _not equal null_ predicate.
    pub fn nen(mut self) -> Self {
        self.filter = Some(FieldFilter::Nen);
        self
    }
    /// Filter records with greater that_ predicate.
    pub fn gt(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Gt(criteria.into()));
        self
    }
    /// Filter records with greater or equal_ predicate.
    pub fn ge(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Ge(criteria.into()));
        self
    }
    /// Filter records with lesser than_ predicate.
    pub fn lt(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Lt(criteria.into()));
        self
    }
    /// Filter records with lesser or equal_ predicate.
    pub fn le(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Le(criteria.into()));
        self
    }
    /// Filter records with _between_ predicate. This is inclusive, so `x bw 3 6` is the same as `x ge 3, x le 6`
    pub fn bw(mut self, lower: impl Into<SqlArg>, upper: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Bw(lower.into(), upper.into()));
        self
    }
    /// Filter records with _like_ predicate.
    pub fn lk(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Lk(criteria.into()));
        self
    }
    /// Filter records with _regex_ predicate.
    pub fn re(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Re(criteria.into()));
        self
    }
   
    /// Filter records with _inside_ predicate.
    pub fn ins<T, I>(mut self, criteria: I) -> Self
    where T: Into<SqlArg>, I :IntoIterator<Item = T>
     {
        self.filter = Some(FieldFilter::In(
            criteria.into_iter().map(|c| c.into()).collect(),
        ));
        self
    }
    /// Filter records with _outside_ predicate.
    pub fn out<T,I>(mut self, criteria: I) -> Self 
     where T: Into<SqlArg>, I :IntoIterator<Item = T>
    {
        self.filter = Some(FieldFilter::Out(
            criteria.into_iter().map(|c| c.into()).collect(),
        ));
        self
    }
    /// Filter records with custom function.
    /// To provide a custom function you must implement (FieldHandler)[../sql_mapper/trait.FieldHandler.html]
    /// See _custom handler test_ for an example.
    pub fn fnc<U, T, I>(mut self, name: U, args: I) -> Self
    where
        U: Into<String>, T: Into<SqlArg>, I :IntoIterator<Item = T>
    {
        self.filter = Some(FieldFilter::Fn(
            name.into(),
            args.into_iter().map(|c| c.into()).collect(),
        ));
        self
    }

    /// Filter records with custom function.
    /// To provide a custom function you must implement (FieldHandler)[../sql_mapper/trait.FieldHandler.html]
    /// See _custom handler test_ for an example.
    pub fn concatenate(mut self, concatenation: Concatenation) -> Self
    {
        self.concatenation = concatenation;
        self
    }

}

impl ToString for Field {
    fn to_string(&self) -> String {
        let mut s = String::new();
        match self.order {
            Some(FieldOrder::Asc(o)) => {
                s.push('+');
                s.push_str(&o.to_string());
            }
            Some(FieldOrder::Desc(o)) => {
                s.push('-');
                s.push_str(&o.to_string());
            }
            None => {}
        };
        if self.hidden {
            s.push('.');
        }
        s.push_str(&self.name);

        if self.filter.is_some() {
            if self.aggregation {
                s.push_str(" !");
            } else {
                s.push(' ');
            }
        }
        match self.filter {
            None => {}
            Some(ref filter) => {
                s.push_str(filter.to_string().as_str());
            }
        }
        s
    }
}

impl From<&str> for Field {
    fn from(s: &str) -> Field {
        Field::from(s)
    }
}

impl Into<QueryToken> for Field {
    fn into(self) -> QueryToken {
        QueryToken::Field(self)
    }
}

/// The filter operation on a field. You use this when creating a [FieldHandler](../sql_mapper/trait.FieldHandler.html)
/// to provide custom functions through the _Fn_ filter or implement a alternative mapping to SQL.
#[derive(Clone, Debug)]
pub enum FieldFilter {
    Eq(SqlArg),
    Eqn,
    Ne(SqlArg),
    Nen,
    Gt(SqlArg),
    Ge(SqlArg),
    Lt(SqlArg),
    Le(SqlArg),
    Lk(SqlArg),
    Bw(SqlArg, SqlArg), // Lower, upper limit
    In(Vec<SqlArg>),
    Out(Vec<SqlArg>),
    Re(SqlArg),
    //  Sc(String),
    Fn(String, Vec<SqlArg>), // Function name, args
}

impl ToString for FieldFilter {
    fn to_string(&self) -> String {
        let mut s = String::new();
        match self {
            FieldFilter::Eq(ref arg) => {
                s.push_str("EQ ");
                s.push_str(&arg.to_sql_string());
            }
            FieldFilter::Eqn => {
                s.push_str("EQN");
            }
            FieldFilter::Ne(ref arg) => {
                s.push_str("NE ");
                s.push_str(&arg.to_sql_string());
            }
            FieldFilter::Nen => {
                s.push_str("NEN");
            }
            FieldFilter::Gt(ref arg) => {
                s.push_str("GT ");
                s.push_str(&arg.to_sql_string());
            }
            FieldFilter::Ge(ref arg) => {
                s.push_str("GE ");
                s.push_str(&arg.to_sql_string());
            }
            FieldFilter::Lt(ref arg) => {
                s.push_str("LT ");
                s.push_str(&arg.to_sql_string());
            }
            FieldFilter::Le(ref arg) => {
                s.push_str("LE ");
                s.push_str(&arg.to_sql_string());
            }
            FieldFilter::Lk(ref arg) => {
                s.push_str("LK ");
                s.push_str(&arg.to_sql_string());
            }
            FieldFilter::Re(ref arg) => {
                s.push_str("RE ");
                s.push_str(&arg.to_sql_string());
            }
            FieldFilter::Bw(ref lower, ref upper) => {
                s.push_str("BW ");
                s.push_str(&lower.to_sql_string());
                s.push(' ');
                s.push_str(&upper.to_sql_string());
            }
            FieldFilter::In(ref args) => {
                s.push_str("IN ");
                s.push_str(&args.iter().map(|a|a.to_sql_string()).collect::<Vec<_>>().join(" "))
            }
            FieldFilter::Out(ref args) => {
                s.push_str("OUT ");
                s.push_str(&args.iter().map(|a|a.to_sql_string()).collect::<Vec<_>>().join(" "))
            }
            FieldFilter::Fn(ref name, ref args) => {
                s.push_str("FN ");
                s.push_str(name);
                s.push(' ');
                s.push_str(&args.iter().map(|a|a.to_sql_string()).collect::<Vec<_>>().join(" "))
            }
        }
        s
    }
}

// A Toql predicate provides additional filtering. 
/// A field can be created from a field name and filtered, sorted with its methods.
/// However the Toql derive creates fields structs for a derived struct, so instead of
/// ``` ignore
///  
///  let f = Predicate::from("search").is(["what"]);
/// ```
/// its easier and recommended to write
/// ``` ignore
///  let f = User::predicates().search(&["what"]);
/// ```
#[derive(Clone, Debug)]
pub struct Predicate {
    pub(crate) concatenation: Concatenation,
    pub(crate) name: String,
    pub(crate) args: Vec<SqlArg>,
}


impl Predicate {
    /// Create a field for the given name.
    pub fn from<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        let name = name.into();
        #[cfg(debug)]
        {
            // Ensure name does not end with wildcard
            if name.ends_with("*") {
                panic!("Fieldname {:?} must not end with wildcard.", name);
            }
        }

        Predicate {
            concatenation: Concatenation::And,
            name: name.into(),
            args: Vec::new(),
          
        }
    }

    /// Add Argument to predicate
    pub fn is(mut self, arg: impl Into<SqlArg>) -> Self{
        //self.args = arg.to_args();
        self.args.push(arg.into());
        self
    }
   /*   pub fn are<'a, T : 'a >(mut self, args: &'a[T]) -> Self
     where &'a T : Into<SqlArg>
     {
         
         for a in args {
             self.args.push(a.into());
         }
        
        self
    }
 */
}

impl ToString for Predicate {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push('@');
        
        s.push_str(&self.name);
        s.push(' ');

        for a in &self.args {
            s.push_str (&a.to_string());
            s.push(' ');
        }
    
        s.pop();
        s
    }
}

impl From<&str> for Predicate {
    fn from(s: &str) -> Predicate {
        Predicate::from(s)
    }
}

impl Into<QueryToken> for Predicate {
    fn into(self) -> QueryToken {
        QueryToken::Predicate(self)
    }
}


#[derive(Clone, Debug)]
pub(crate) enum FieldOrder {
    Asc(u8),
    Desc(u8),
}

#[derive(Clone, Debug)]
pub(crate) enum QueryToken {
    LeftBracket(Concatenation),
    RightBracket,
    Wildcard(Wildcard),
    Field(Field),
    Predicate(Predicate)
}

impl From<&str> for QueryToken {
    fn from(s: &str) -> QueryToken {
        if s.ends_with("*") {
            QueryToken::Wildcard(Wildcard::from(s))
        } else {
            QueryToken::Field(Field::from(s))
        }
    }
}

impl ToString for QueryToken {
    fn to_string(&self) -> String {
        let s = match self {
            QueryToken::RightBracket => String::from(")"),
            QueryToken::LeftBracket(c) => match c {
                Concatenation::And => String::from("("),
                Concatenation::Or => String::from("("),
            },
            QueryToken::Field(field) => field.to_string(),
             QueryToken::Predicate(predicate) => predicate.to_string(),
            QueryToken::Wildcard(wildcard) => format!("{}*", wildcard.path),
        };
        s
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
use crate::sql::SqlArg;
#[derive(Clone, Debug)]
pub struct Query<M> {
    pub(crate) tokens: Vec<QueryToken>,
    /// Select DISTINCT
    pub distinct: bool,
    /* /// Roles a query has to access fields.
    /// See [MapperOption](../sql_mapper/struct.MapperOptions.html#method.restrict_roles) for explanation.
    pub roles: HashSet<String>, */
    pub aux_params: HashMap<String, SqlArg>, // generic build params

    pub where_predicates: Vec<String>, // Additional where clause
    pub where_predicate_params: Vec<SqlArg>, // Query params for additional sql restriction
    pub select_columns: Vec<String>,   // Additional select columns

    pub join_stmts: Vec<String>, // Additional joins statements
    pub join_stmt_params: Vec<SqlArg>, // Join params for additional sql restriction
   // pub wildcard_scope: Option<HashSet<String>> // Restrict wildcard to certain fields
   pub type_marker: std::marker::PhantomData<M>
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
            type_marker: std::marker::PhantomData
          //  wildcard_scope: None
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
    pub fn clone_for_type<T>(&self) -> Query::<T>
    {
       Query::<T> {
            tokens: self.tokens.clone(),
            distinct: self.distinct.clone(),
            aux_params: self.aux_params.clone(),
            where_predicates: self.where_predicates.clone(),
            where_predicate_params: self.where_predicate_params.clone(),
            select_columns: self.select_columns.clone(),
            join_stmts: self.join_stmts.clone(),
            join_stmt_params: self.join_stmt_params.clone(),
            type_marker: std::marker::PhantomData
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
            type_marker: std::marker::PhantomData
          //  wildcard_scope: None
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
    /// Concatenate field or query with OR.
    pub fn or<T>(mut self, query: T) -> Self
    where
        T: Into<Query<M>>,
    {
        // Change first token of query to concatenate with OR
        let mut query = query.into();
        if let QueryToken::LeftBracket(c) = query.tokens.get_mut(0).unwrap() {
            *c = Concatenation::Or;
        } else if let QueryToken::Field(field) = query.tokens.get_mut(0).unwrap() {
            field.concatenation = Concatenation::Or;
        } else if let QueryToken::Wildcard(wildcard) = query.tokens.get_mut(0).unwrap() {
            wildcard.concatenation = Concatenation::Or;
        }

        self.tokens.append(&mut query.tokens);
        self
    }

    /// Modifiy the query with an additional stuct.
    pub fn with(self, query_with: impl QueryWith) -> Self {
        query_with.with(self)
    }

    /// Convenence method to add aux params
    pub fn aux_param<S, A>(mut self, name: S, value: A) -> Self 
    where A: Into<SqlArg>, S: Into<String>
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
                    path.starts_with(&wildcard.path) ||  wildcard.path.starts_with(pth)
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
                QueryToken::Wildcard(wildcard) => {
                    wildcard.path.starts_with(pth)
                }
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
            s = s.trim_start_matches(",").trim_start_matches(";").to_owned();
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
        q.tokens.push(if string.ends_with("*") {
            QueryToken::Wildcard(Wildcard::from(string))
        } else {
            QueryToken::Field(Field::from(string))
        });
        q
    }
}
