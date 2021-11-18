use crate::{
    alias_translator::AliasTranslator,
    error::ToqlError,
    from_row::FromRow,
    result::Result,
    sql::Sql,
    sql_arg::SqlArg,
    sql_builder::SqlBuilder,
    sql_expr::{resolver::Resolver, PredicateColumn, SqlExpr},
    table_mapper::{mapped::Mapped, TableMapper},
    tree::{
        tree_identity::{IdentityAction, TreeIdentity},
        tree_predicate::TreePredicate,
        tree_update::TreeUpdate,
    },
};

use super::{
    insert::{build_insert_sql, set_tree_identity},
    map, Backend,
};
use std::{
    borrow::{Borrow, BorrowMut},
    collections::{HashMap, HashSet},
};

use crate::{
    query::field_path::FieldPath,
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
    // Ensure entity is mapped
    {
        let registry = &mut *backend.registry_mut()?;
        map::map::<T>(registry)?;
    }

    let (field_order, merge_order, fields_map) =
        plan_update_order::<T, _>(&*backend.registry()?, fields.list.as_ref())?;

    let mut refreshed_paths = HashSet::new();

    for query_path in &field_order {
        let fields = match fields_map.get(query_path) {
            Some(f) => f,
            None => continue,
        };

        let parent_path = FieldPath::trim_basename(query_path);
        if !refreshed_paths.contains(parent_path.as_str()) {
            let path = FieldPath::from(&query_path);
            for e in entities.iter_mut() {
                <T as TreeIdentity>::set_id(
                    e.borrow_mut(),
                    path.children(),
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

            // Collect key arguments, may be empty if merge is unselected
            <T as TreePredicate>::args(e.borrow(), qp.children(), &mut args)?;
            for arg in args.chunks(cols.len()) {
                query_path_should_insert_map
                    .entry(query_merge_path)
                    .or_insert_with(Vec::new)
                    .push(!crate::sql_arg::valid_key(&arg));
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
                    query_path.children(),
                    &IdentityAction::RefreshInvalid,
                )?;
            }
            refreshed_paths.insert(query_path.to_string());
        }
        // Only selected merges contain insert information
        if let Some(should_insert_vec) = query_path_should_insert_map.get(query_field.as_str()) {
            let should_insert = should_insert_vec.iter();
            delete_removed_merges(backend, entities, &query_field, should_insert).await?;
            let mut should_insert = should_insert_vec.iter();
            insert_new_merges(backend, entities, &query_field, &mut should_insert).await?;
        }
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
    should_insert: &mut J,
) -> std::result::Result<(), E>
where
    T: Update,
    Q: BorrowMut<T> + Sync,
    B: Backend<R, E>,
    E: From<ToqlError>,
    SqlArg: FromRow<R, E>,
    J: Iterator<Item = &'b bool> + Clone,
{
    use std::cell::RefCell;
    let merge_path = FieldPath::from(&query_path);

    // Insert
    let sql = build_insert_sql(
        backend,
        entities,
        &merge_path,
        &mut should_insert.clone(),
        "",
        "",
    )?;
    if let Some(sql) = sql {
        // Insert and refresh generated id
        if <T as TreeIdentity>::auto_id(merge_path.children())? {
            let ids = backend.insert_sql(sql).await?;
            set_tree_identity(
                IdentityAction::SetInvalid(RefCell::new(ids)),
                &mut entities.borrow_mut(),
                merge_path.children(),
            )?;
        } else {
            backend.execute_sql(sql).await?;
        }
    }

    // Cascade insert for partial tables
    let mut partial_merge_paths = Vec::new();
    add_partial_tables::<T>(&*backend.registry()?, &merge_path, &mut partial_merge_paths)?;

    for partial_merge_path in partial_merge_paths {
        let sql = build_insert_sql(
            backend,
            entities,
            &FieldPath::from(&partial_merge_path),
            &mut should_insert.clone(),
            "",
            "",
        )?;
        if let Some(sql) = sql {
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
    let columns = <T as TreePredicate>::columns(parent_path.children())?;
    let mut args = Vec::new();
    for e in entities.iter() {
        <T as TreePredicate>::args(e.borrow(), parent_path.children(), &mut args)?;
    }
    let columns = columns
        .into_iter()
        .map(PredicateColumn::SelfAliased)
        .collect::<Vec<_>>();
    key_predicate.push_predicate(columns, args);

    // Fetch merge keys
    // to prevent added entities from being deleted
    let columns = <T as TreePredicate>::columns(merge_path.children())?;
    let mut unfiltered_args = Vec::new();
    for e in entities.iter() {
        <T as TreePredicate>::args(e.borrow(), merge_path.children(), &mut unfiltered_args)?;
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
            .map(PredicateColumn::OtherAliased)
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

        let mut alias_translator = AliasTranslator::new(backend.alias_format());
        let resolver = Resolver::new();
        resolver
            .to_sql(&delete_expr, &mut alias_translator)
            .map_err(ToqlError::from)?
    };
    backend.execute_sql(sql).await?;

    Ok(())
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
        let descendents = path.children();
        TreeUpdate::update(e.borrow(), descendents, fields, backend.roles(), &mut exprs)?;
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
// "userLanguage" refers to merges -> will delete and insert rows
fn plan_update_order<T, S: AsRef<str>>(
    registry: &TableMapperRegistry,
    query_fields: &[S],
) -> Result<(Vec<String>, Vec<String>, HashMap<String, HashSet<String>>)>
where
    T: Update,
{
    let mut field_path_order: Vec<String> = Vec::new();
    let mut merge_path_order = Vec::new();
    let mut fields: HashMap<String, HashSet<String>> = HashMap::new(); // paths that refer to fields

    let ty = <T as Mapped>::type_name();
    for query_field in query_fields {
        let trimmed_query_field = query_field.as_ref().trim_end_matches('_');
        let (query_path, fieldname) = FieldPath::split_basename(trimmed_query_field);

        let sql_builder = SqlBuilder::new(&ty, registry);
        let mapper: &TableMapper = sql_builder.mapper_for_query_path(&query_path)?;

        let merge = mapper.merged_mapper(fieldname).is_some();
        if !merge {
            let k = query_path.to_string();
            if !fields.contains_key(&k) {
                // Faster lookup than Vec
                field_path_order.push(k);
            }

            fields
                .entry(query_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(fieldname.to_string());
        } else {
            let k = trimmed_query_field.to_string();
            if !merge_path_order.contains(&k) {
                // TODO maybe unordered Hashset works too
                merge_path_order.push(k);
            }
        }
    }

    Ok((field_path_order, merge_path_order, fields))
}

fn add_partial_tables<T>(
    registry: &TableMapperRegistry,
    query_path: &FieldPath,
    paths: &mut Vec<String>,
) -> std::result::Result<(), ToqlError>
where
    T: Mapped + TreePredicate,
{
    let ty = <T as Mapped>::type_name();

    let sql_builder = SqlBuilder::new(&ty, registry);
    let mapper = sql_builder.mapper_for_query_path(query_path)?;

    let partial_joins: Vec<(String, String)> = mapper.joined_partial_mappers();

    for (p, _m) in &partial_joins {
        let qp = query_path.append(p);
        add_partial_tables::<T>(registry, &qp, paths)?;
        paths.push(qp.to_string());
    }

    Ok(())
}
