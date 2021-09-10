use crate::{
    alias_format::AliasFormat,
    alias_translator::AliasTranslator,
    error::ToqlError,
    from_row::FromRow,
    parameter_map::ParameterMap,
    query::field_path::FieldPath,
    result::Result,
    sql::Sql,
    sql_arg::SqlArg,
    sql_builder::{sql_builder_error::SqlBuilderError, SqlBuilder},
    sql_expr::{resolver::Resolver, PredicateColumn, SqlExpr},
    table_mapper::{mapped::Mapped, TableMapper},
    tree::{
        tree_identity::{IdentityAction, TreeIdentity},
        tree_index::TreeIndex,
        tree_insert::TreeInsert,
        tree_predicate::TreePredicate,
        tree_update::TreeUpdate,
    },
};

use super::{insert::build_insert_sql2, map, Backend};
use std::{
    borrow::{Borrow, BorrowMut},
    collections::{HashMap, HashSet},
};

use crate::{
    table_mapper_registry::TableMapperRegistry,
    toql_api::{fields::Fields, update::Update},
};

pub async fn update<B, Q, T, R, E>(
    backend: &mut B,
    entities: &mut [Q],
    fields: Fields,
) -> std::result::Result<(), E>
where
    T: Update,
    Q: BorrowMut<T> + Sync,
    B: Backend<R, E>,
    E: From<ToqlError>,
    SqlArg: FromRow<R, E>,
{
   

    // TODO should be possible to impl with &str
    /*  let mut joined_or_normal_fields: HashMap<String, HashSet<String>> = HashMap::new();
    let mut merges: HashMap<String, HashSet<String>> = HashMap::new(); */

    // Ensure entity is mapped
    {
        let registry = &mut *backend.registry_mut()?;
        map::map::<T>(registry)?;
    }

    
    let (query_path_order, mut auto_key_len, fields_map) =
        plan_update_order2::<T, _>(&*backend.registry()?, fields.list.as_ref())?;

    for (query_path, merge) in query_path_order {
        let empty = HashSet::new();
        let fields = match fields_map.get(&query_path) {
            Some(f) => f,
            None => &empty,
        };
        println!("Processing path: {}", &query_path);

        // Ensure dependend keys are up to date
        // Maybe a seperate query_path list needs to be calculated for more accurate auto id updates.
         if auto_key_len > 0 {
             let p = FieldPath::from(&query_path);
            for e in entities.iter_mut() {
                <T as TreeIdentity>::set_id(
                    e.borrow_mut(),
                    &mut p.children(),
                    &IdentityAction::Refresh,
                )?;
            }
        } 

        if merge {
            update_merge(backend, entities, &query_path, fields).await?;
        } else {
            update_field_or_join(backend, entities, &query_path, fields).await?;
        }
        auto_key_len -= 1;
    }

    Ok(())
}

async fn update_field_or_join<T, B, R, E, Q>(
    backend: &mut B,
    entities: &mut [Q],
    query_path: &str,
    fields: &HashSet<String>,
) -> std::result::Result<(), E>
where
    T: Update,
    Q: BorrowMut<T> + Sync,
    B: Backend<R, E>,
    E: From<ToqlError>,
{
    // Ensure keys are valid
    let sqls = {
        let field_path = FieldPath::from(&query_path);
        build_update_sql(backend, entities, &field_path, &fields, None, "", "")
    }?;

    // Update joins
    for sql in sqls {
        backend.execute_sql(sql).await?;
    }
    Ok(())
}

async fn update_merge<T, B, R, E, Q>(
    backend: &mut B,
    entities: &mut [Q],
    query_path: &str,
    fields: &HashSet<String>,
) -> std::result::Result<(), E>
where
    T: Update,
    Q: BorrowMut<T> + Sync,
    B: Backend<R, E>,
    E: From<ToqlError>,
    SqlArg: FromRow<R, E>,
{
    delete_unused_merge_items(backend, entities, &query_path).await?;
    let (inserts, updates) =
        calculate_insert_and_update_items(backend, entities, &query_path).await?;

    let parent_path = FieldPath::from(&query_path);
    let merge_path = FieldPath::from(&query_path);

    //Update
    let sqls = build_update_sql(
        backend,
        entities,
        &merge_path,
        &fields,
        Some(&updates),
        "",
        "",
    )?;
    for sql in sqls {
        backend.execute_sql(sql).await?;
    }

    // Insert

    let sql = build_insert_sql2(backend, entities, &merge_path, &inserts, "", "")?;
    if let Some(sql) = sql {
        println!("SQL {:?}", &sql);
        let mut descendents = parent_path.children();
        if <T as TreeIdentity>::auto_id(&mut descendents)? {
            let ids = backend.insert_sql(sql).await?;

            let mut descendents = merge_path.children();
            crate::backend::insert::set_tree_identity2(
                ids,
                &mut entities.borrow_mut(),
                &mut descendents,
            )?;
        } else {
            backend.execute_sql(sql).await?;
        }
    }

    Ok(())
}

async fn delete_unused_merge_items<T, B, R, E, Q>(
    backend: &mut B,
    entities: &[Q],
    merge_path: &str,
) -> std::result::Result<(), E>
where
    T: Mapped + TreePredicate,
    Q: Borrow<T> + Sync,
    B: Backend<R, E>,
    E: From<ToqlError>,
{
    let (_, parent_path) = FieldPath::split_basename(&merge_path);
    let merge_path = FieldPath::from(&merge_path);
    let mut key_predicate: SqlExpr = SqlExpr::new();

    // Fetch parent key
    let columns = <T as TreePredicate>::columns(&mut parent_path.children())?;
    let mut args = Vec::new();
    for e in entities.iter() {
        <T as TreePredicate>::args(e.borrow(), &mut parent_path.children(), &mut args)?;
    }
    let columns = columns
        .into_iter()
        .map(|c| PredicateColumn::SelfAliased(c))
        .collect::<Vec<_>>();
    key_predicate.push_predicate(columns, args);

    // Fetch merge keys
    let columns = <T as TreePredicate>::columns(&mut merge_path.children())?;
    let mut args = Vec::new();
    for e in entities.iter() {
        <T as TreePredicate>::args(e.borrow(), &mut merge_path.children(), &mut args)?;
    }
    let columns = columns
        .into_iter()
        .map(|c| PredicateColumn::OtherAliased(c))
        .collect::<Vec<_>>();
    key_predicate.push_literal(" AND NOT (");
    key_predicate.push_predicate(columns, args);
    key_predicate.push_literal(")");

    // Construct sql
    let sql = {
        let type_name = <T as Mapped>::type_name();
        let registry = &*backend.registry()?;
        let mut sql_builder = SqlBuilder::new(&type_name, registry)
            .with_aux_params(backend.aux_params().clone()) // todo ref
            .with_roles(backend.roles().clone()); // todo ref
        let delete_expr = sql_builder.build_merge_delete(&merge_path, key_predicate.to_owned())?;

        let mut alias_translator = AliasTranslator::new(backend.alias_format().clone());
        let resolver = Resolver::new();
        resolver
            .to_sql(&delete_expr, &mut alias_translator)
            .map_err(ToqlError::from)?
    };

    println!("{:?}", sql);
    //dbg!(sql.to_unsafe_string());
    //backend.execute_sql(sql).await?;

    Ok(())
}

async fn calculate_insert_and_update_items<T, B, R, E, Q>(
    backend: &mut B,
    entities: &[Q],
    merge_path: &str,
) -> std::result::Result<(Vec<Vec<SqlArg>>, Vec<Vec<SqlArg>>), E>
where
    T: Mapped + TreePredicate + TreeInsert,
    Q: Borrow<T> + Sync,
    B: Backend<R, E>,
    E: From<ToqlError>,
    SqlArg: FromRow<R, E>,
{
    let (_, parent_path) = FieldPath::split_basename(&merge_path);
    let merge_path = FieldPath::from(&merge_path);
    let mut key_predicate: SqlExpr = SqlExpr::new();

    // Fetch parent key
    let columns = <T as TreePredicate>::columns(&mut parent_path.children())?;
    let mut args = Vec::new();
    for e in entities.iter() {
        <T as TreePredicate>::args(e.borrow(), &mut parent_path.children(), &mut args)?;
    }
    let columns = columns
        .into_iter()
        .map(|c| PredicateColumn::SelfAliased(c))
        .collect::<Vec<_>>();
    key_predicate.push_predicate(columns, args);

    // Fetch merge keys
    let columns = <T as TreePredicate>::columns(&mut merge_path.children())?;
    let mut args = Vec::new();
    for e in entities.iter() {
        <T as TreePredicate>::args(e.borrow(), &mut merge_path.children(), &mut args)?;
    }
    let mut key_columns_expr = SqlExpr::new();
    for c in &columns {
        key_columns_expr.extend(SqlExpr::other_alias());
        key_columns_expr.push_literal(".");
        key_columns_expr.push_literal(c.to_owned());
        key_columns_expr.push_literal(", ");
    }
    key_columns_expr.pop();

    //let key_columns = columns.iter().map(|c| SqlExpr::aliased_column(c.to_owned())).collect::<Vec<_>>();

    let columns = columns
        .into_iter()
        .map(|c| PredicateColumn::OtherAliased(c))
        .collect::<Vec<_>>();
    let number_of_columns = columns.len();
    key_predicate.push_literal(" AND (");
    key_predicate.push_predicate(columns, args);
    key_predicate.push_literal(")");

    // Construct sql
    let sql = {
        let type_name = <T as Mapped>::type_name();
        let registry = &*backend.registry()?;
        let mut sql_builder = SqlBuilder::new(&type_name, registry)
            .with_aux_params(backend.aux_params().clone()) // todo ref
            .with_roles(backend.roles().clone()); // todo ref
        let delete_expr = sql_builder.build_merge_key_select(
            &merge_path,
            key_columns_expr,
            key_predicate.to_owned(),
        )?;

        let mut alias_translator = AliasTranslator::new(backend.alias_format().clone());
        let resolver = Resolver::new();
        resolver
            .to_sql(&delete_expr, &mut alias_translator)
            .map_err(ToqlError::from)?
    };

    println!("{:?}", sql);
    //dbg!(sql.to_unsafe_string());
    let rows = backend.select_sql(sql).await?;

    let mut keys_index: Vec<Vec<SqlArg>> = Vec::new();
    let mut iter = std::iter::repeat(&crate::sql_builder::select_stream::Select::Query);
    for row in rows {
        let mut keys: Vec<SqlArg> = Vec::new();
        let mut index = 0;
        for _n in 0..number_of_columns {
            let a = <crate::sql_arg::SqlArg as crate::from_row::FromRow<R, E>>::from_row(
                &row, &mut index, &mut iter,
            )?;
            keys.push(a.ok_or(ToqlError::ValueMissing("Key column".to_string()))?);
        }
        keys_index.push(keys);
    }
    println!("{:?}", keys_index);

    // Filter what to update and what to insert
    let mut entities_to_insert: Vec<Vec<SqlArg>> = Vec::new();
    let mut entities_to_update: Vec<Vec<SqlArg>> = Vec::new();

    for (n, e) in entities.iter().enumerate() {
        // Get all keys from vector
        let mut args = Vec::new();
        <T as TreePredicate>::args(e.borrow(), &mut merge_path.children(), &mut args)?;

        // Get key Sizes
        for a in args.chunks(number_of_columns) {
            if keys_index.contains(&a.to_owned()) {
                // Update

                entities_to_update.push(a.to_owned());
            } else {
                println!("New needs insert");
                //insert
                entities_to_insert.push(a.to_owned());
            }
        }
    }

    Ok((entities_to_insert, entities_to_update))
}

fn build_update_sql<B, T, Q, R, E>(
    backend: &mut B,
    entities: &[Q],
    path: &FieldPath,
    fields: &HashSet<String>,
    selected_keys: Option<&[Vec<SqlArg>]>,
    _modifier: &str,
    _extra: &str,
) -> Result<Vec<Sql>>
where
    B: Backend<R, E>,
    T: Mapped + TreeUpdate,
    Q: Borrow<T>,
    E: From<ToqlError>,
{
    let mut alias_translator = AliasTranslator::new(backend.alias_format());

    let mut update_sqls = Vec::new();

    let mut exprs = Vec::new();
    for e in entities.iter() {
        //let mut descendents = path.descendents();
        let mut descendents = path.children();
        TreeUpdate::update(
            e.borrow(),
            &mut descendents,
            fields,
            backend.roles(),
            selected_keys,
            &mut exprs,
        )?;
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
fn plan_update_order<T, S: AsRef<str>>(
    mappers: &HashMap<String, TableMapper>,
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

        let mut is_merge = false;
        // Get mapper for path
        for c in children {
            let mapper = mappers
                .get(&current_mapper)
                .ok_or(ToqlError::MapperMissing(current_mapper))?;

            if let Some(j) = mapper.joined_mapper(c.as_str()) {
                is_merge = false;
                current_mapper = j.to_string();
            } else if let Some(m) = mapper.merged_mapper(c.as_str()) {
                is_merge = true;
                current_mapper = m.to_string();
            } else {
                return Err(SqlBuilderError::JoinMissing(c.as_str().to_owned()).into());
            }
        }
        /*  let mapper = mappers
        .get(&current_mapper)
        .ok_or(ToqlError::MapperMissing(current_mapper))?; */

        // Triage field
        // Join use as normal field (this will insert keys of the join)
        /*  if mapper.joined_mapper(descendent_name).is_some() {
            fields
                .entry(ancestor_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(descendent_name.to_string());
          /*   fields
                .entry(path.as_ref().trim_end_matches('_').to_string())
                .or_insert_with(HashSet::new)
                .insert("*".to_string()); */
        } */
        // Merged field
        if is_merge {
            merges
                .entry(ancestor_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(descendent_name.to_string());
        }
        // Joins and normal field
        else {
            fields
                .entry(ancestor_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(descendent_name.to_string());
        }
    }
    Ok(())
}
// separate out fields, that refer to merged entities
// E.g on struct user "userLanguage_order" will update all orders in userLanguages
// "userLanguage" refers to merges -> will replace rows
fn plan_update_order2<T, S: AsRef<str>>(
    registry: &TableMapperRegistry,
    query_paths: &[S],
) -> Result<(Vec<(String, bool)>, usize, HashMap<String, HashSet<String>>)>
// Execution order
where
    T: Update,
{
    let mut ak_execution_order = Vec::new();
    let mut execution_order = Vec::new();
    let mut fields: HashMap<String, HashSet<String>> = HashMap::new(); // paths that refer to fields

    let ty = <T as Mapped>::type_name();
    for query_path in query_paths {
        let (field, query_path) = FieldPath::split_basename(query_path.as_ref());
        fields
            .entry(query_path.to_string())
            .or_insert_with(HashSet::new)
            .insert(field.to_string());
        let sql_builder = SqlBuilder::new(&ty, registry);
        let (base_mapper, parent_path) = FieldPath::split_basename(query_path.as_str());

        let mapper: &TableMapper = sql_builder.mapper_for_query_path(&parent_path)?;

        let merge = mapper.merged_mapper(base_mapper).is_some();

        let auto_id_path = FieldPath::from(query_path.as_str());
        if <T as TreeIdentity>::auto_id(&mut auto_id_path.children())? {
            ak_execution_order.push((query_path.as_str().to_string(), merge));
        } else {
            execution_order.push((query_path.as_str().to_string(), merge));
        }
    }

    let ak_execution_order_len = ak_execution_order.len();
    ak_execution_order.extend_from_slice(&execution_order);
    Ok((ak_execution_order, ak_execution_order_len, fields))
}
