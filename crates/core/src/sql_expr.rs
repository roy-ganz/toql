pub mod resolver;
pub mod resolver_error;

use crate::sql_arg::SqlArg;
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum PredicateColumn {
    SelfAliased(String),
    OtherAliased(String),
    Literal(String),
    Aliased(String, String) // Alias name / column 
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
    
}

#[derive(Debug, Clone)]
pub struct SqlExpr {
    tokens: Vec<SqlExprToken>,
    maybe_aux_params: bool // Hint to speed up function first_aux_param
}

impl SqlExpr {
    
    pub fn from(tokens: Vec<SqlExprToken>) -> Self {
        let maybe_aux_params = tokens.iter().find(|t| if let SqlExprToken::AuxParam(_) = t {true} else {false}).is_some();
        SqlExpr { tokens, maybe_aux_params }
    }
    pub fn new() -> Self {
        SqlExpr { tokens: Vec::new(), maybe_aux_params: false }
    }
    
    pub fn literal(lit: impl Into<String>) -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::Literal(lit.into())],
             maybe_aux_params: false
        }
    }
    pub fn self_alias() -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::SelfAlias],
             maybe_aux_params: false
        }
    }
    pub fn other_alias() -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::OtherAlias],
             maybe_aux_params: false
        }
    }
    pub fn unresolved_arg() -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::UnresolvedArg],
             maybe_aux_params: false
        }
    }
    pub fn arg(a: SqlArg) -> Self {
        SqlExpr {
            tokens: vec![SqlExprToken::Arg(a)],
             maybe_aux_params: false
        }
    }
    pub fn aliased_column(column_name: String) -> Self {
        SqlExpr {
            tokens: vec![
                SqlExprToken::SelfAlias,
                SqlExprToken::Literal(".".to_string()),
                SqlExprToken::Literal(column_name),
            ],
             maybe_aux_params: false
        }
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

    /* pub fn push_separator(&mut self, lit: impl Into<String>) -> &mut Self {
        if let Some(SqlExprToken::Literal(l)) = self.tokens.last_mut() {
            let lit = lit.into();
            if !l.trim_end().ends_with(lit.as_str()) {
                l.push_str(lit.as_str())
            }
        } else {
            self.tokens.push(SqlExprToken::Literal(lit.into()));
        }
        self
    } */
    pub fn pop_literals(&mut self, count: usize) -> &mut Self {
        if let Some(SqlExprToken::Literal(l)) = self.tokens.last_mut() {
            for _ in 0..count {
                l.pop();
            }
        }
        self
    }
    pub fn ends_with_literal(&mut self, lit: &str) -> bool {
        if let Some(SqlExprToken::Literal(l)) = self.tokens.last_mut() {
            l.ends_with(lit)
        } else {
            false
        }
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
    pub fn push_unresolved_arg(&mut self) -> &mut Self {
        self.tokens.push(SqlExprToken::UnresolvedArg);
        self
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn first_aux_param(&self) -> Option<&String> {

        if self.maybe_aux_params == true {
            for t in self.tokens() {
                if let SqlExprToken::AuxParam(p) = t {
                    return Some(p);
                }
            }
        }
        None
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

    pub fn extend(&mut self, expr: impl Into<SqlExpr>) -> &mut Self {
        let tokens = expr.into().tokens;
        let maybe_aux_params = tokens.iter().find(|t| if let SqlExprToken::AuxParam(_) = t {true} else {false}).is_some();
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
            // SqlExprToken::InClause { column, args: _ } => {write!(f, "{} IN (..TODO..)", column )}
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
