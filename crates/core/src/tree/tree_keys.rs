use crate::query::field_path::Descendents;
use crate::{sql_builder::sql_builder_error::SqlBuilderError, sql_expr::SqlExpr};
use std::result::Result;

pub trait TreeKeys {
    fn keys<'a>(
        descendents: &mut Descendents<'a>,
        field: &str,
        key_expr: &mut SqlExpr,
    ) -> Result<(), SqlBuilderError>;
}
