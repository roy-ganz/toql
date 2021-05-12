use crate::{
    alias::AliasFormat,
    alias_translator::AliasTranslator,
    error::ToqlError,
    query::field_path::FieldPath,
    result::Result,
    sql::Sql,
    sql_builder::sql_builder_error::SqlBuilderError,
    sql_expr::resolver::Resolver,
    sql_mapper::{mapped::Mapped, SqlMapper},
    tree::tree_update::TreeUpdate,
};

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

pub fn build_update_sql<T, Q>(
    alias_format: AliasFormat,

    entities: &[Q],
    path: &FieldPath,
    fields: &HashSet<String>,
    roles: &HashSet<String>,
    _modifier: &str,
    _extra: &str,
) -> Result<Vec<Sql>>
where
    T: Mapped + TreeUpdate,
    Q: Borrow<T>,
{
    let mut alias_translator = AliasTranslator::new(alias_format);

    let mut update_sqls = Vec::new();

    let mut exprs = Vec::new();
    for e in entities.iter() {
        //let mut descendents = path.descendents();
        let mut descendents = path.step_down();
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

// separate out fields, that refer to merged entities
// E.g on struct user "userLanguage_order" will update all orders in userLanguages
// "userLanguage" refers to merges -> will replace rows
pub fn plan_update_order<T, S: AsRef<str>>(
    mappers: &HashMap<String, SqlMapper>,
    query_paths: &[S],
    fields: &mut HashMap<String, HashSet<String>>, // paths that refer to fields
    merges: &mut HashMap<String, HashSet<String>>, // paths that refer to merges
) -> Result<()>
where
    T: Mapped,
{
    let ty = <T as Mapped>::type_name();
    for path in query_paths {
        let (descendent_name, ancestor_path) =
            FieldPath::split_basename(path.as_ref().trim_end_matches('_'));

        let children = ancestor_path.children();

        let mut current_mapper: String = ty.to_owned();

        // Get mapper for path
        for c in children {
            let mapper = mappers
                .get(&current_mapper)
                .ok_or(ToqlError::MapperMissing(current_mapper))?;

            if let Some(j) = mapper.joined_mapper(c.as_str()) {
                current_mapper = j.to_string();
            } else if let Some(m) = mapper.merged_mapper(c.as_str()) {
                current_mapper = m.to_string();
            } else {
                return Err(SqlBuilderError::JoinMissing(c.as_str().to_owned()).into());
            }
        }
        let mapper = mappers
            .get(&current_mapper)
            .ok_or(ToqlError::MapperMissing(current_mapper))?;

        // Triage field
        // Join, convert to wildcard
        if mapper.joined_mapper(descendent_name).is_some() {
            fields
                .entry(path.as_ref().trim_end_matches('_').to_string())
                .or_insert_with(HashSet::new)
                .insert("*".to_string());
        }
        // Merged field
        else if mapper.merged_mapper(descendent_name).is_some() {
            merges
                .entry(ancestor_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(descendent_name.to_string());
        }
        // Normal field
        else {
            fields
                .entry(ancestor_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(descendent_name.to_string());
        }
    }
    Ok(())
}
