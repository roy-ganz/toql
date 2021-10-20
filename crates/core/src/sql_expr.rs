//! SQL expression than can be resolved into raw SQL.
pub mod resolver;
pub mod resolver_error;
use crate::sql_arg::SqlArg;
use std::fmt;

/// Hold information about a predicate column.
/// Used by the [SqlExprToken::Predicate]
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateColumn {
    /// The column name is self aliased.
    SelfAliased(String),
    /// The column name is other aliased.
    OtherAliased(String),
    /// The column name is not aliased.
    Literal(String),
    /// The column name is already aliased.
    /// Tuple with (Alias name, column).
    Aliased(String, String),
}

/// Token that makes up [SqlExpr].
/// The expression is a string of tokens.
/// To turn the expression into SQL, each
// token must be fully resolved. This is done by the [Resolver].
#[derive(Debug, Clone)]
pub enum SqlExprToken {
    /// Self alias in expression
    SelfAlias,
    /// Other alias in expression
    OtherAlias,
    /// Aux param in expression
    AuxParam(String),
    /// Unresolved arguent in expression
    UnresolvedArg,
    /// Literal raw SQL in expression
    Literal(String),
    /// Argument in expression
    Arg(SqlArg),
    /// Canonical alias in expression
    Alias(String),
    /// Predicate in expression
    /// A predicate token has been introduced to
    /// improve SQL formatting.
    /// Depending on the number of columns and arguments
    /// this translates different SQL.
    Predicate {
        columns: Vec<PredicateColumn>,
        args: Vec<SqlArg>,
    },
}

/// A SQL expression is a list of tokens that can be resolved into SQL.
///
/// Library users are advised to not build it programmatically,
/// but to use the [sql_expr!](toql_sql_expr_macro::sql_expr) macro.
/// This macro provides compile time safety and convenience.
///
/// However it's also possible to build it programmatically:
///
/// ### Example
///
/// ```rust
/// let mut e = SqlExpr::literal("SELECT ");
///  e.push_self_alias();
///  e.push_literal("id FROM User ");
///  e.push_self_alias();
///  assert("SELECT ..id FROM User ..", e.to_string());
/// ```
/// The resolver will replace the self aliases into real aliases and build proper SQL.
#[derive(Debug, Clone)]
pub struct SqlExpr {
    tokens: Vec<SqlExprToken>,
    /// Hint to speed up function first_aux_param
    maybe_aux_params: bool,
}

impl SqlExpr {
    /// Create SQL expression from token list.
    pub fn from(tokens: Vec<SqlExprToken>) -> Self {
        let maybe_aux_params = tokens
            .iter()
            .any(|t| matches!(t, SqlExprToken::AuxParam(_)));

        SqlExpr {
            tokens,
            maybe_aux_params,
        }
    }
    /// Create new empty SQL expression.
    pub fn new() -> Self {
        SqlExpr {
            tokens: Vec::new(),
            maybe_aux_params: false,
        }
    }
    /// Create SQL expression from literal.
    pub fn literal(lit: impl Into<String>) -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::Literal(lit.into())],
            maybe_aux_params: false,
        }
    }
    /// Create SQL expression from alias.
    pub fn alias(lit: impl Into<String>) -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::Alias(lit.into())],
            maybe_aux_params: false,
        }
    }
    /// Create SQL expression from self alias.
    pub fn self_alias() -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::SelfAlias],
            maybe_aux_params: false,
        }
    }
    /// Create SQL expression from other alias.
    pub fn other_alias() -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::OtherAlias],
            maybe_aux_params: false,
        }
    }
    /// Create SQL expression from unresolved argument.
    pub fn unresolved_arg() -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::UnresolvedArg],
            maybe_aux_params: false,
        }
    }
    /// Create SQL expression from argument.
    pub fn arg(a: SqlArg) -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::Arg(a)],
            maybe_aux_params: false,
        }
    }
    /// Create SQL expression from aliased column.
    pub fn aliased_column(column_name: String) -> Self {
        SqlExpr {
            tokens: vec![
                SqlExprToken::SelfAlias,
                SqlExprToken::Literal(".".to_string()),
                SqlExprToken::Literal(column_name),
            ],
            maybe_aux_params: false,
        }
    }

    /// Add literal at the end of token list.
    pub fn push_literal(&mut self, lit: impl Into<String>) -> &mut Self {
        self.tokens.push(SqlExprToken::Literal(lit.into()));
        self
    }
    /// Remove a number of characters -or less- from the end of the list.
    /// This affects only the last (literal) token.
    pub fn pop_literals(&mut self, count: usize) -> &mut Self {
        if let Some(SqlExprToken::Literal(l)) = self.tokens.last_mut() {
            for _ in 0..count {
                l.pop();
            }
        }
        self
    }
    /// Return true if last literal token ends with `lit`.
    pub fn ends_with_literal(&mut self, lit: &str) -> bool {
        if let Some(SqlExprToken::Literal(l)) = self.tokens.last_mut() {
            l.ends_with(lit)
        } else {
            false
        }
    }
    /// Remove last token from list.
    pub fn pop(&mut self) -> &mut Self {
        self.tokens.pop();
        self
    }
    /// Add self alias to the end of the list.
    pub fn push_self_alias(&mut self) -> &mut Self {
        self.tokens.push(SqlExprToken::SelfAlias);
        self
    }
    /// Add other alias to the end of the list.
    pub fn push_other_alias(&mut self) -> &mut Self {
        self.tokens.push(SqlExprToken::OtherAlias);
        self
    }
    /// Add custom alias to the end of the list.
    pub fn push_alias(&mut self, alias: impl Into<String>) -> &mut Self {
        self.tokens.push(SqlExprToken::Alias(alias.into()));
        self
    }
    /// Add argument to the end of the list.
    pub fn push_arg(&mut self, arg: SqlArg) -> &mut Self {
        self.tokens.push(SqlExprToken::Arg(arg));
        self
    }
    /// Add unresolved argument to the end of the list.
    pub fn push_unresolved_arg(&mut self) -> &mut Self {
        self.tokens.push(SqlExprToken::UnresolvedArg);
        self
    }
    /// Returns true, if list is empty.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    /// Returns first auxiliary parameter, if any.
    pub fn first_aux_param(&self) -> Option<&String> {
        if self.maybe_aux_params {
            for t in self.tokens() {
                if let SqlExprToken::AuxParam(p) = t {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Add a predicate to the end of the list.
    pub fn push_predicate(
        &mut self,
        columns: Vec<PredicateColumn>,
        args: Vec<SqlArg>,
    ) -> &mut Self {
        // Append args to last predicate if they have the same columns
        if let Some(SqlExprToken::Predicate {
            columns: c,
            args: a,
        }) = self.tokens.last_mut()
        {
            if c.iter().eq(&columns) {
                a.extend(args);
            } else {
                self.tokens.push(SqlExprToken::Predicate { columns, args });
            }
        } else {
            self.tokens.push(SqlExprToken::Predicate { columns, args });
        }
        self
    }

    /// Add another SQL expression to the end of the list.
    pub fn extend(&mut self, expr: impl Into<SqlExpr>) -> &mut Self {
        let tokens = expr.into().tokens;
        let maybe_aux_params = tokens
            .iter()
            .any(|t| matches!(t, SqlExprToken::AuxParam(_)));
        self.maybe_aux_params |= maybe_aux_params;
        self.tokens.extend(tokens);
        self
    }
    pub fn tokens(&self) -> &[SqlExprToken] {
        &self.tokens
    }
}

impl fmt::Display for SqlExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for t in &self.tokens {
            write!(f, "{}", t)?;
        }
        Ok(())
    }
}

impl fmt::Display for SqlExprToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SqlExprToken::SelfAlias => write!(f, ".."),
            SqlExprToken::OtherAlias => write!(f, "..."),
            SqlExprToken::AuxParam(name) => write!(f, "<{}>", name),
            SqlExprToken::UnresolvedArg => write!(f, "?"),
            SqlExprToken::Literal(l) => write!(f, "{}", l),
            SqlExprToken::Arg(a) => write!(f, "{}", a.to_string()),
            SqlExprToken::Alias(a) => write!(f, "{}", a),
            SqlExprToken::Predicate {
                columns: _,
                args: _,
            } => write!(f, "ToDo"),
        }
    }
}

impl std::convert::From<&str> for SqlExpr {
    fn from(s: &str) -> Self {
        SqlExpr::literal(s)
    }
}
impl std::convert::From<String> for SqlExpr {
    fn from(s: String) -> Self {
        SqlExpr::literal(s)
    }
}
impl std::convert::From<&String> for SqlExpr {
    fn from(s: &String) -> Self {
        SqlExpr::literal(s)
    }
}

impl Default for SqlExpr {
    fn default() -> Self {
        Self::new()
    }
}
