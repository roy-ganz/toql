use crate::{
    alias_format::AliasFormat,
    alias_translator::AliasTranslator,
    error::ToqlError,
    parameter_map::ParameterMap,
    query::field_path::FieldPath,
    sql::Sql,
    sql_builder::{SqlBuilder, sql_builder_error::SqlBuilderError},
    sql_expr::resolver::Resolver,
    table_mapper::{mapped::Mapped, TableMapper},
    tree::{tree_identity::TreeIdentity, tree_insert::TreeInsert},
};
use std::{borrow::BorrowMut, collections::HashMap};

use crate::{sql_arg::SqlArg, result::Result};
use std::collections::HashSet;
use super::{Backend, map};
use crate::toql_api::paths::Paths;

use crate::{table_mapper_registry::TableMapperRegistry, toql_api::insert::Insert};


  pub async fn insert<B, Q, T, R, E>(backend : &mut B, mut entities: &mut [Q], paths: Paths) ->std::result::Result<(), E> where
            Q: BorrowMut<T>,
            T: Insert,
            B: Backend<R, E>, E: From<ToqlError>
    {
        
         {
                let registry = &mut *backend.registry_mut()?;
                map::map::<T>(registry)?;
            }
    

        // Build up execution tree
        // Path `a_b_merge1_c_d_merge2_e` becomes
        // [0] = [a, c, e]
        // [1] = [a_b, c_d]
        // [m] = [merge1, merge2]
        // Then execution order is [1], [0], [m]


        // TODO should be possible to impl with &str
        let mut joins: Vec<HashSet<String>> = Vec::new();
        let mut merges: HashSet<String> = HashSet::new();



        crate::backend::insert::plan_insert_order::<T, _>(
            &backend.registry()?.mappers,
            paths.list.as_ref(),
            &mut joins,
            &mut merges,
        )?;

        // Insert root
        let sql = {
            let aux_params = [backend.aux_params()];
            let aux_params = ParameterMap::new(&aux_params);
            let home_path = FieldPath::default();

            crate::backend::insert::build_insert_sql::<T, _>(
                &backend.registry()?.mappers,
                backend.alias_format(),
                &aux_params,
                entities,
                &backend.roles(),
                &home_path,
                "",
                "",
            )
        }?;
        if sql.is_none() {
            return Ok(());
        }
        let sql = sql.unwrap();

        let home_path = FieldPath::default();
        let mut descendents = home_path.children();

        let ids = backend.insert_sql(sql).await?;

        // Updated auto keys
        if <T as TreeIdentity>::auto_id(&mut descendents)? {
            let mut descendents = home_path.children();
            crate::backend::insert::set_tree_identity2(
                ids ,
                &mut entities,
                &mut descendents,
            )?;
         }


        // Insert joins
        for l in (0..joins.len()).rev() { // TEST not rev
            for p in joins.get(l).unwrap() {
                let mut path = FieldPath::from(&p);

                let sql = {
                    let aux_params = [backend.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    crate::backend::insert::build_insert_sql::<T, _>(
                        &backend.registry()?.mappers,
                        backend.alias_format(),
                        &aux_params,
                        entities,
                        &backend.roles(),
                        &mut path,
                        "",
                        "",
                    )
                }?;
                if sql.is_none() {
                    break;
                }
                let sql = sql.unwrap();

            let mut descendents = path.children();
            if <T as TreeIdentity>::auto_id(&mut descendents)? {
                let ids= backend.insert_sql(sql).await?;

                let mut descendents = home_path.children();
                crate::backend::insert::set_tree_identity2(
                    ids ,
                    &mut entities,
                    &mut descendents,
                )?;
             } else {
                backend.execute_sql(sql).await?;
             }
            }
        }

        // Insert merges
        for p in merges {
            let path = FieldPath::from(&p);

            let sql = {
                let aux_params = [backend.aux_params()];
                let aux_params = ParameterMap::new(&aux_params);
                crate::backend::insert::build_insert_sql::<T, _>(
                    &backend.registry()?.mappers,
                    backend.alias_format(),
                    &aux_params,
                    entities,
                    &backend.roles(),
                    &path,
                    "",
                    "",
                )
            }?;
            if sql.is_none() {
                break;
            }
            let sql = sql.unwrap();

            // Merges must not contain auto value as identity, skip set_tree_identity
            backend.execute_sql(sql).await?;

        }
        Ok(())
    }
    


pub fn set_tree_identity2<'a, T, Q, I>(
    ids: Vec<SqlArg>,
    entities: &mut [Q],
    mut descendents: &mut I,
) -> Result<()>
where
    T: TreeIdentity,
    Q: BorrowMut<T>,
    I: Iterator<Item = FieldPath<'a>>,
{
    
    use crate::tree::tree_identity::IdentityAction;
    use std::cell::RefCell;

    //   let home_path = FieldPath::default();
    //    let mut descendents= home_path.descendents();
    let action = IdentityAction::Set(RefCell::new(ids));
    for e in entities.iter_mut() {
        {
            let e_mut = e.borrow_mut();
            <T as TreeIdentity>::set_id(e_mut, &mut descendents, &action)?;
        }
    }
    
    Ok(())
}

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
    
    use crate::tree::tree_identity::IdentityAction;
    use std::cell::RefCell;

   // if <T as TreeIdentity>::auto_id() {
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
    //}
    Ok(())
}

pub fn build_insert_sql2<T, Q>(
    registry: &TableMapperRegistry,
    alias_format: AliasFormat,
    aux_params: &ParameterMap,
    entities: &[Q],
    roles: &HashSet<String>,
    path: &FieldPath,
    _modifier: &str,
    _extra: &str,
    key_set: &[Vec<SqlArg>]
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
    for e in entities{
    
        //let mut d = path.descendents();
        let mut d = path.step_down();
        <T as TreeInsert>::values(e.borrow(), &mut d, roles, Some(key_set), &mut values_expr)?;
    
    }
    if values_expr.is_empty() {
        return Ok(None);
    }

    /* let mut mapper = mappers
        .get(&ty)
        .ok_or_else(|| ToqlError::MapperMissing(ty.to_owned()))?; */
    let mut alias_translator = AliasTranslator::new(alias_format);

    let sql_builder = SqlBuilder::new(&ty, registry);
    let mapper = sql_builder.mapper_for_query_path(path)?;
   /*  // Walk down mappers
    for d in path.step_down() {
        //for d in path.descendents(){
        let mapper_name = mapper
            .joined_mapper(d.as_str())
            .or_else(|| mapper.merged_mapper(d.as_str()));
        let mapper_name =
            mapper_name.ok_or_else(|| ToqlError::MapperMissing(d.as_str().to_owned()))?;
        mapper = mappers
            .get(&mapper_name)
            .ok_or_else(|| ToqlError::MapperMissing(mapper_name.to_owned()))?;
    } */

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
    insert_stmt.push(' ');
    insert_stmt.push_str(&columns_sql.0);
    insert_stmt.push_str(" VALUES ");
    insert_stmt.push_str(&values_sql.0);

    insert_stmt.pop(); // Remove ', '
    insert_stmt.pop();

    Ok(Some(Sql(insert_stmt, values_sql.1)))
}
pub fn build_insert_sql<T, Q>(
    mappers: &HashMap<String, TableMapper>,
    alias_format: AliasFormat,
    aux_params: &ParameterMap,
    entities: &[Q],
    roles: &HashSet<String>,
    path: &FieldPath,
    _modifier: &str,
    _extra: &str,
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
        <T as TreeInsert>::values(e.borrow(), &mut d, roles, None,&mut values_expr)?;
    }
    if values_expr.is_empty() {
        return Ok(None);
    }

    let mut mapper = mappers
        .get(&ty)
        .ok_or_else(|| ToqlError::MapperMissing(ty.to_owned()))?;
    let mut alias_translator = AliasTranslator::new(alias_format);

    // Walk down mappers
    for d in path.step_down() {
        //for d in path.descendents(){
        let mapper_name = mapper
            .joined_mapper(d.as_str())
            .or_else(|| mapper.merged_mapper(d.as_str()));
        let mapper_name =
            mapper_name.ok_or_else(|| ToqlError::MapperMissing(d.as_str().to_owned()))?;
        mapper = mappers
            .get(&mapper_name)
            .ok_or_else(|| ToqlError::MapperMissing(mapper_name.to_owned()))?;
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
    insert_stmt.push(' ');
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
    mappers: &HashMap<String, TableMapper>,
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
            .ok_or_else(|| ToqlError::MapperMissing(ty.to_owned()))?;

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
                    .ok_or_else(|| ToqlError::MapperMissing(j.to_owned()))?;
            } else if let Some(m) = mapper.merged_mapper(c.as_str()) {
                level = 0;
                merges.insert(d.as_str().to_string());
                mapper = mappers
                    .get(&m)
                    .ok_or_else(|| ToqlError::MapperMissing(m.to_owned()))?;
            } else {
                return Err(SqlBuilderError::JoinMissing(c.as_str().to_owned()).into());
            }
        }
    }
    Ok(())
}
