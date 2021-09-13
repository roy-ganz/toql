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

    let (field_order, merge_order, fields_map) =
        plan_update_order3::<T, _>(&*backend.registry()?, fields.list.as_ref())?;

    let mut refreshed_paths = HashSet::new();

    for query_path in &field_order {
        let empty = HashSet::new();
        let fields = match fields_map.get(query_path) {
            Some(f) => f,
            None => &empty,
        };

        let parent_path = FieldPath::trim_basename(query_path);
        if !refreshed_paths.contains(parent_path.as_str()) {
            let path = FieldPath::from(&query_path);
            for e in entities.iter_mut() {
                <T as TreeIdentity>::set_id(
                    e.borrow_mut(),
                    &mut path.children(),
                    &IdentityAction::RefreshValid,
                )?;
            }
            refreshed_paths.insert(parent_path.to_string());
        }

        update_field_or_join(backend, entities, &query_path, fields).await?;
    }

    // Save insert positions for merged entities
    // After key refresh of parent invalid keys in merged entities will become valid.
    let mut query_path_should_insert_map: HashMap<&str, Vec<bool>> = HashMap::new();
    for query_merge_path in &merge_order {
        let qp = FieldPath::from(query_merge_path);

        let cols = <T as TreePredicate>::columns(&mut qp.children())?;
        for e in entities.iter() {
            let mut args = Vec::new();
            let qp = FieldPath::from(query_merge_path);

            <T as TreePredicate>::args(e.borrow(), &mut qp.children(), &mut args)?;
            for arg in args.chunks(cols.len()) {
                query_path_should_insert_map
                    .entry(query_merge_path)
                    .or_insert_with(Vec::new)
                    .push(crate::sql_arg::is_invalid(&arg));
            }
        }
    }
    let mut refreshed_paths = HashSet::new();
    for query_field in &merge_order {
        // Ensure composite keys in merged items have correct parrent key value
        let query_path = FieldPath::trim_basename(query_field);
        if !refreshed_paths.contains(query_path.as_str()) {
            for e in entities.iter_mut() {
                <T as TreeIdentity>::set_id(
                    e.borrow_mut(),
                    &mut query_path.children(),
                    &IdentityAction::RefreshInvalid,
                )?;
            }
            refreshed_paths.insert(query_path.to_string());
        }

        let should_insert_vec = query_path_should_insert_map
            .get(query_field.as_str())
            .unwrap(); // Is always none

        let mut should_insert = should_insert_vec.iter();

        delete_removed_merges(backend, entities, &query_field, &mut should_insert).await?;

        let mut should_insert = should_insert_vec.iter();
        insert_new_merges(backend, entities, &query_field, &mut should_insert).await?;
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
        build_update_sql(backend, entities, &field_path, &fields, "", "")
    }?;

    // Update joins
    for sql in sqls {
        backend.execute_sql(sql).await?;
    }
    Ok(())
}

async fn insert_new_merges<'b, T, B, R, E, Q, J>(
    backend: &mut B,
    entities: &mut [Q],
    query_path: &str,
    inserts: &mut J,
) -> std::result::Result<(), E>
where
    T: Update,
    Q: BorrowMut<T> + Sync,
    B: Backend<R, E>,
    E: From<ToqlError>,
    SqlArg: FromRow<R, E>,
    J: Iterator<Item = &'b bool>,
{
    let merge_path = FieldPath::from(&query_path);

    // Insert
    let sql = build_insert_sql2(backend, entities, &merge_path, inserts, "", "")?;
    if let Some(sql) = sql {
        println!("SQL {:?}", &sql);
        // Insert and refresh generated id
        let mut descendents = merge_path.children();
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

async fn delete_removed_merges<'b, T, B, R, E, Q, J>(
    backend: &mut B,
    entities: &[Q],
    merge_path: &str,
    should_insert: J,
) -> std::result::Result<(), E>
where
    T: Mapped + TreePredicate,
    Q: Borrow<T> + Sync,
    B: Backend<R, E>,
    E: From<ToqlError>,
    J: Iterator<Item = &'b bool>,
{
    let parent_path = FieldPath::trim_basename(&merge_path);
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
    // to prevent added entities from being deleted
    let columns = <T as TreePredicate>::columns(&mut merge_path.children())?;
    let mut unfiltered_args = Vec::new();
    for e in entities.iter() {
        <T as TreePredicate>::args(e.borrow(), &mut merge_path.children(), &mut unfiltered_args)?;
    }
    let mut args: Vec<SqlArg> = Vec::with_capacity(unfiltered_args.len());

    for (ua, s) in unfiltered_args.chunks(columns.len()).zip(should_insert) {
        if !*s {
            args.extend(ua.to_owned());
        }
    }

    if !args.is_empty() {
        let columns = columns
            .into_iter()
            .map(|c| PredicateColumn::OtherAliased(c))
            .collect::<Vec<_>>();
        key_predicate.push_literal(" AND NOT (");
        key_predicate.push_predicate(columns, args);
        key_predicate.push_literal(")");
    }

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
    backend.execute_sql(sql).await?;

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
    let parent_path = FieldPath::trim_basename(&merge_path);
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
        let (ancestor_path, descendent_name) =
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
    query_fields: &[S],
) -> Result<(Vec<(String, bool)>, HashMap<String, HashSet<String>>, usize)>
// Execution order
where
    T: Update,
{
    let mut ak_execution_order = Vec::new();
    let mut execution_order = Vec::new();
    let mut fields: HashMap<String, HashSet<String>> = HashMap::new(); // paths that refer to fields

    let ty = <T as Mapped>::type_name();
    for query_field in query_fields {
        let trimmed_query_field = query_field.as_ref().trim_end_matches("_");
        let (query_path, fieldname) = FieldPath::split_basename(trimmed_query_field);

        let sql_builder = SqlBuilder::new(&ty, registry);
        // let (parent_path, base_mapper) = FieldPath::split_basename(query_path.as_str());

        let mapper: &TableMapper = sql_builder.mapper_for_query_path(&query_path)?;

        let merge = mapper.merged_mapper(fieldname).is_some();
        if !merge {
            fields
                .entry(query_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(fieldname.to_string());

            let auto_id_path = FieldPath::from(&query_path);
            if <T as TreeIdentity>::auto_id(&mut auto_id_path.children())? {
                ak_execution_order.push((query_path.as_str().to_string(), merge));
            // Insert only base path and add fields seperately
            } else {
                execution_order.push((query_path.as_str().to_string(), merge));
            }
        } else {
            let auto_id_path = FieldPath::from(query_path.as_str());
            if <T as TreeIdentity>::auto_id(&mut auto_id_path.children())? {
                ak_execution_order.push((trimmed_query_field.to_string(), merge));
            // Insert merge path
            } else {
                execution_order.push((trimmed_query_field.to_string(), merge));
            }
        }
    }

    let ak_execution_order_len = ak_execution_order.len();
    ak_execution_order.extend_from_slice(&execution_order);
    Ok((ak_execution_order, fields, ak_execution_order_len))
}
// separate out fields, that refer to merged entities
// E.g on struct user "userLanguage_order" will update all orders in userLanguages
// "userLanguage" refers to merges -> will replace rows
fn plan_update_order3<T, S: AsRef<str>>(
    registry: &TableMapperRegistry,
    query_fields: &[S],
) -> Result<(Vec<String>, Vec<String>, HashMap<String, HashSet<String>>)>
// Execution order
where
    T: Update,
{
    let mut field_order = Vec::new();
    let mut merge_order = Vec::new();
    let mut fields: HashMap<String, HashSet<String>> = HashMap::new(); // paths that refer to fields

    let ty = <T as Mapped>::type_name();
    for query_field in query_fields {
        let trimmed_query_field = query_field.as_ref().trim_end_matches("_");
        let (query_path, fieldname) = FieldPath::split_basename(trimmed_query_field);

        let sql_builder = SqlBuilder::new(&ty, registry);
        // let (parent_path, base_mapper) = FieldPath::split_basename(query_path.as_str());

        let mapper: &TableMapper = sql_builder.mapper_for_query_path(&query_path)?;

        let merge = mapper.merged_mapper(fieldname).is_some();
        if !merge {
            fields
                .entry(query_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(fieldname.to_string());
            field_order.push(query_path.to_string());
        } else {
            merge_order.push(trimmed_query_field.to_string());
        }
    }

    Ok((field_order, merge_order, fields))
}
