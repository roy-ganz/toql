pub mod resolver;
pub mod resolver_error;

use crate::sql_arg::SqlArg;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum PredicateColumn {
    SelfAliased(String),
    OtherAliased(String),
    Literal(String),
}

#[derive(Debug, Clone)]
pub enum SqlExprToken {
    SelfAlias,
    OtherAlias,
    AuxParam(String),
    UnresolvedArg,

    Literal(String),
    Arg(SqlArg),
    Alias(String),
    //InClause { column: String, args: Vec<SqlArg> },
    Predicate {
        columns: Vec<PredicateColumn>,
        args: Vec<SqlArg>,
    },
    Placeholder(u16, SqlExpr, usize),
}

#[derive(Debug, Clone)]
pub struct SqlExpr {
    tokens: Vec<SqlExprToken>,
}

impl SqlExpr {
    /*
    pub fn new() -> Self {
        SqlExpr {
            tokens:Vec::new()
        }
    } */

    pub fn from(tokens: Vec<SqlExprToken>) -> Self {
        SqlExpr { tokens }
    }
    pub fn new() -> Self {
        SqlExpr { tokens: Vec::new() }
    }
    
    pub fn literal(lit: impl Into<String>) -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::Literal(lit.into())],
        }
    }
    pub fn aliased_column(column_name: String) -> Self {
        SqlExpr {
            tokens: vec![
                SqlExprToken::SelfAlias,
                SqlExprToken::Literal(".".to_string()),
                SqlExprToken::Literal(column_name),
            ],
        }
    }

    pub fn push_placeholder(
        &mut self,
        number: u16,
        expr: SqlExpr,
        selection_position: usize,
    ) -> &mut Self {
        self.tokens
            .push(SqlExprToken::Placeholder(number, expr, selection_position));
        self
    }

    pub fn push_literal(&mut self, lit: impl Into<String>) -> &mut Self {
        self.tokens.push(SqlExprToken::Literal(lit.into()));
        self
    }

    /* fn ends_with(token : &SqlExprToken, lit: &str) -> bool {
        match token {

            SqlExprToken::Literal(l) => {l.ends_with(lit)}
            SqlExprToken::Placeholder(_, e, _) => { e.tokens.last().map(|t|Self::ends_with(t, lit)).unwrap_or(false) }
            _ => false
        }

    } */

    pub fn push_separator(&mut self, lit: impl Into<String>) -> &mut Self {
        if let Some(SqlExprToken::Literal(l)) = self.tokens.last_mut() {
            let lit = lit.into();
            if !l.trim_end().ends_with(lit.as_str()) {
                l.push_str(lit.as_str())
            }
        } else {
            self.tokens.push(SqlExprToken::Literal(lit.into()));
        }
        self
    }
    pub fn pop_literals(&mut self, count: usize) -> &mut Self {
        if let Some(SqlExprToken::Literal(l)) = self.tokens.last_mut() {
            for _ in 0..count {
                l.pop();
            }
        }
        self
    }
    pub fn pop(&mut self) -> &mut Self {
        self.tokens.pop();
        self
    }

    pub fn push_self_alias(&mut self) -> &mut Self {
        self.tokens.push(SqlExprToken::SelfAlias);
        self
    }
    pub fn push_other_alias(&mut self) -> &mut Self {
        self.tokens.push(SqlExprToken::OtherAlias);
        self
    }
    pub fn push_alias(&mut self, alias: impl Into<String>) -> &mut Self {
        self.tokens.push(SqlExprToken::Alias(alias.into()));
        self
    }
    pub fn push_arg(&mut self, arg: SqlArg) -> &mut Self {
        self.tokens.push(SqlExprToken::Arg(arg));
        self
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

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

    pub fn extend(&mut self, expr: SqlExpr) -> &mut Self {
        self.tokens.extend(expr.tokens);
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
            // SqlExprToken::InClause { column, args: _ } => {write!(f, "{} IN (..TODO..)", column )}
            SqlExprToken::Predicate {
                columns: _,
                args: _,
            } => write!(f, "ToDo"),
            SqlExprToken::Placeholder(n, e, _) => {
                write!(f, "|{}:", n)?;
                e.fmt(f)?;
                write!(f, "|")
            }
        }
    }
}
