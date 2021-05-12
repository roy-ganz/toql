use crate::parameter_map::ParameterMap;
use crate::{
    alias::AliasFormat,
    alias_translator::AliasTranslator,
    error::ToqlError,
    query::field_path::{Descendents, FieldPath},
    sql::Sql,
    sql_builder::sql_builder_error::SqlBuilderError,
    sql_expr::resolver::Resolver,
    sql_mapper::{mapped::Mapped, SqlMapper},
    tree::{tree_identity::TreeIdentity, tree_insert::TreeInsert},
};
use std::{borrow::BorrowMut, collections::HashMap};

use crate::result::Result;
use std::collections::HashSet;

pub fn set_tree_identity<'a, T, Q, I>(
    first_id: u64,
    number_of_ids: u64,
    entities: &mut [Q],
    mut descendents: &mut I,
) -> Result<()>
where
    T: TreeIdentity,
    Q: BorrowMut<T>,
    I: Iterator<Item = FieldPath<'a>>,
{
    use crate::sql_arg::SqlArg;
    use crate::tree::tree_identity::IdentityAction;
    use std::cell::RefCell;

    if <T as TreeIdentity>::auto_id() {
        let mut id: u64 = first_id + number_of_ids;
        let mut ids: Vec<SqlArg> = Vec::with_capacity(number_of_ids as usize);
        for _ in 0..number_of_ids {
            ids.push(SqlArg::U64(id));
            id -= 1;
        }

        //   let home_path = FieldPath::default();
        //    let mut descendents= home_path.descendents();
        let action = IdentityAction::Set(RefCell::new(ids));
        for e in entities.iter_mut() {
            {
                let e_mut = e.borrow_mut();
                <T as TreeIdentity>::set_id(e_mut, &mut descendents, &action)?;
            }
        }
    }
    Ok(())
}
pub fn build_insert_sql<T, Q>(
    mappers: &HashMap<String, SqlMapper>,
    alias_format: AliasFormat,
    aux_params: &ParameterMap,
    entities: &[Q],
    roles: &HashSet<String>,
    path: &FieldPath,
    modifier: &str,
    extra: &str,
) -> Result<Option<Sql>>
where
    T: Mapped + TreeInsert,
    Q: BorrowMut<T>,
{
    use crate::sql_expr::SqlExpr;

    let ty = <T as Mapped>::type_name();

    let mut values_expr = SqlExpr::new();
    //let mut d = path.descendents();
    let mut d = path.step_down();
    let columns_expr = <T as TreeInsert>::columns(&mut d)?;
    for e in entities {
        //let mut d = path.descendents();
        let mut d = path.step_down();
        <T as TreeInsert>::values(e.borrow(), &mut d, roles, &mut values_expr)?;
    }
    if values_expr.is_empty() {
        return Ok(None);
    }

    let mut mapper = mappers
        .get(&ty)
        .ok_or(ToqlError::MapperMissing(ty.to_owned()))?;
    let mut alias_translator = AliasTranslator::new(alias_format);

    // Walk down mappers
    for d in path.step_down() {
        //for d in path.descendents(){
        let mapper_name = mapper
            .joined_mapper(d.as_str())
            .or(mapper.merged_mapper(d.as_str()));
        let mapper_name = mapper_name.ok_or(ToqlError::MapperMissing(d.as_str().to_owned()))?;
        mapper = mappers
            .get(&mapper_name)
            .ok_or(ToqlError::MapperMissing(mapper_name.to_owned()))?;
    }

    let resolver = Resolver::new()
        .with_aux_params(&aux_params)
        .with_self_alias(&mapper.canonical_table_alias);
    let columns_sql = resolver
        .to_sql(&columns_expr, &mut alias_translator)
        .map_err(ToqlError::from)?;
    let values_sql = resolver
        .to_sql(&values_expr, &mut alias_translator)
        .map_err(ToqlError::from)?;

    let mut insert_stmt = String::from("INSERT INTO ");
    insert_stmt.push_str(&mapper.table_name);
    insert_stmt.push_str(" ");
    insert_stmt.push_str(&columns_sql.0);
    insert_stmt.push_str(" VALUES ");
    insert_stmt.push_str(&values_sql.0);

    insert_stmt.pop(); // Remove ', '
    insert_stmt.pop();

    Ok(Some(Sql(insert_stmt, values_sql.1)))
}
pub fn split_basename(
    fields: &[String],
    path_basenames: &mut HashMap<String, HashSet<String>>,
    paths: &mut Vec<String>,
) {
    for f in fields {
        let (base, path) = FieldPath::split_basename(f);
        if !path.is_empty() {
            if path_basenames.get(path.as_str()).is_none() {
                path_basenames.insert(path.as_str().to_string(), HashSet::new());
            }
            path_basenames
                .get_mut(path.as_str())
                .unwrap()
                .insert(base.to_string());

            paths.push(path.to_string());
        }
    }
}

pub fn plan_insert_order<T, S: AsRef<str>>(
    mappers: &HashMap<String, SqlMapper>,
    paths: &[S],
    joins: &mut Vec<HashSet<String>>,
    merges: &mut HashSet<String>,
) -> Result<()>
where
    T: Mapped,
{
    let ty = <T as Mapped>::type_name();
    for path in paths {
        let field_path = FieldPath::from(path.as_ref());
        let steps = field_path.step_down();
        let children = field_path.children();
        let mut level = 0;
        let mut mapper = mappers
            .get(&ty)
            .ok_or(ToqlError::MapperMissing(ty.to_owned()))?;

        for (d, c) in steps.zip(children) {
            if let Some(j) = mapper.joined_mapper(c.as_str()) {
                if joins.len() <= level {
                    joins.push(HashSet::new());
                }

                // let rel = d.relative_path(home_path.as_str()).unwrap_or(FieldPath::default());
                joins.get_mut(level).unwrap().insert(d.as_str().to_string());
                level += 1;
                mapper = mappers
                    .get(&j)
                    .ok_or(ToqlError::MapperMissing(j.to_owned()))?;
            } else if let Some(m) = mapper.merged_mapper(c.as_str()) {
                level = 0;
                merges.insert(d.as_str().to_string());
                mapper = mappers
                    .get(&m)
                    .ok_or(ToqlError::MapperMissing(m.to_owned()))?;
            } else {
                return Err(SqlBuilderError::JoinMissing(c.as_str().to_owned()).into());
            }
        }
    }
    Ok(())
}
