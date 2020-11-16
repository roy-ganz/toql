
use std::{borrow::{Borrow, BorrowMut}, collections::{HashSet, HashMap}};
use toql_core::{alias::AliasFormat, sql_mapper::{mapped::Mapped, SqlMapper}, parameter::ParameterMap, tree::tree_update::TreeUpdate, query::field_path::FieldPath};
use toql_core::{sql_expr::resolver::Resolver, sql::Sql, alias_translator::AliasTranslator, error::ToqlError, sql_builder::sql_builder_error::SqlBuilderError};
use crate::error::Result;

pub(crate) fn build_update_sql<T, Q>( alias_format: AliasFormat, 
  
    entities: &[Q], 
    path: &FieldPath, 
    fields: &HashSet<String>,
    roles: &HashSet<String>,
    modifier: &str, 
    extra: &str) -> toql_core::error::Result<Vec<Sql>>
    where
        T: Mapped + TreeUpdate,
        Q: Borrow<T>,
    {
        let mut alias_translator = AliasTranslator::new(alias_format);

        let mut update_sqls = Vec::new();

        let mut exprs = Vec::new();
        for e in entities.iter() {
            let mut descendents = path.descendents();
            TreeUpdate::update(e.borrow(), &mut descendents, fields, roles, &mut exprs)?;
        }

        // Resolve to Sql
        
        let resolver = Resolver::new();
            
            

            for sql_expr in exprs {
              
                  let update_sql = resolver
                        .to_sql(&sql_expr, &mut alias_translator)
                        .map_err(ToqlError::from)?;
                update_sqls.push(update_sql);

            }


        Ok(update_sqls)
    }
