use crate::{
    alias_translator::AliasTranslator,
    backend::{map, Backend},
    error::ToqlError,
    parameter_map::ParameterMap,
    query::field_path::FieldPath,
    result::Result,
    sql::Sql,
    sql_builder::{sql_builder_error::SqlBuilderError, SqlBuilder},
    sql_expr::resolver::Resolver,
    table_mapper::{mapped::Mapped, TableMapper},
    toql_api::{insert::Insert, paths::Paths},
    tree::{
        tree_identity::{IdentityAction, TreeIdentity},
        tree_insert::TreeInsert,
    },
};
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::{HashMap, HashSet},
};

pub async fn insert<B, Q, T, R, E>(
    backend: &mut B,
    entities: &mut [Q],
    paths: Paths,
) -> std::result::Result<(), E>
where
    Q: BorrowMut<T>,
    T: Insert,
    B: Backend<R, E>,
    E: From<ToqlError>,
{
    {
        let registry = &mut *backend.registry_mut()?;
        map::map::<T>(registry)?;
    }

    // Build up execution tree
    // Path `a_b_merge1_c_d_merge2_e` becomes
    // [j0] = [a, c, e]
    // [j1] = [a_b, c_d]
    // [m] = [merge1, merge2]
    // [p0] = [a, c, e]
    // [p1] = [a_b, c_d]
    // Then execution order is [j1], [j0], [m], [p0], [p1]

    let mut joins: Vec<HashSet<String>> = Vec::new();
    let mut partials: Vec<HashSet<String>> = Vec::new();
    let mut merges: HashSet<String> = HashSet::new();

    plan_insert_order::<T, _>(
        &backend.registry()?.mappers,
        paths.list.as_ref(),
        &mut joins,
        &mut merges,
        &mut partials,
    )?;

    // Insert joins from bottom to top
    for l in (0..joins.len()).rev() {
        for p in joins.get(l).unwrap() {
            let path = FieldPath::from(&p);

            let sql = build_insert_sql(
                backend,
                entities,
                &path,
                &mut std::iter::repeat(&true),
                "",
                "",
            )?;
            insert_sql(backend, path, sql, entities).await?;
        }
    }

    // Insert root
    let home_path = FieldPath::default();
    let sql = build_insert_sql(
        backend,
        entities,
        &home_path,
        &mut std::iter::repeat(&true),
        "",
        "",
    )?;
    insert_sql(backend, home_path, sql, entities).await?;

    // Insert merges
    for p in &merges {
        let path = FieldPath::from(&p);

        let sql = build_insert_sql(
            backend,
            entities,
            &path,
            &mut std::iter::repeat(&true),
            "",
            "",
        )?;

        insert_sql(backend, path, sql, entities).await?;
    }

    // Insert partials from top to bottom
    for l in 0..partials.len() {
        for p in partials.get(l).unwrap() {
            // Ensure not already inserted (unsure if needed)
            if merges.contains(p) {
                continue;
            }

            let path = FieldPath::from(&p);

            let sql = build_insert_sql(
                backend,
                entities,
                &path,
                &mut std::iter::repeat(&true),
                "",
                "",
            )?;

            insert_sql(backend, path, sql, entities).await?;
        }
    }
    Ok(())
}

pub(crate) async fn insert_sql<'a, Q, B, T, R, E>(
    backend: &mut B,
    path: FieldPath<'_>,
    sql: Option<Sql>,
    entities: &mut [Q],
) -> std::result::Result<(), E>
where
    B: Backend<R, E>,
    Q: BorrowMut<T>,
    T: TreeIdentity,
    E: From<ToqlError>,
{
    if sql.is_none() {
        return Ok(());
    }
    let sql = sql.unwrap();

    let descendents = path.children();
    if <T as TreeIdentity>::auto_id(descendents)? {
        let ids = backend.insert_sql(sql).await?;
        set_tree_identity(
            IdentityAction::Set(RefCell::new(ids)),
            entities,
            path.children(),
        )?;
    } else {
        backend.execute_sql(sql).await?;
    }
    Ok(())
}

pub(crate) fn set_tree_identity<'a, T, Q, I>(
    action: IdentityAction,
    entities: &mut [Q],
    descendents: I,
) -> Result<()>
where
    T: TreeIdentity,
    Q: BorrowMut<T>,
    I: Iterator<Item = FieldPath<'a>> + Clone,
{
    for e in entities.iter_mut() {
        let e_mut = e.borrow_mut();
        <T as TreeIdentity>::set_id(e_mut, descendents.clone(), &action)?;
    }

    Ok(())
}

pub(crate) fn build_insert_sql<'a, T, Q, B, R, E, J>(
    backend: &mut B,
    entities: &[Q],
    query_path: &FieldPath,
    inserts: &mut J,
    _modifier: &str,
    _extra: &str,
) -> Result<Option<Sql>>
where
    B: Backend<R, E>,
    T: Mapped + TreeInsert,
    Q: BorrowMut<T>,
    E: From<ToqlError>,
    J: Iterator<Item = &'a bool>,
{
    use crate::sql_expr::SqlExpr;

    let ty = <T as Mapped>::type_name();

    let mut values_expr = SqlExpr::new();

    let mut d = query_path.children();
    let columns_expr = <T as TreeInsert>::columns(&mut d)?;
    for e in entities {
        //let mut d = query_path.children();
        <T as TreeInsert>::values(
            e.borrow(),
            query_path.children(),
            backend.roles(),
            inserts,
            &mut values_expr,
        )?;
    }
    if values_expr.is_empty() {
        return Ok(None);
    }

    let mut alias_translator = AliasTranslator::new(backend.alias_format());

    let registry = &*backend.registry()?;
    let sql_builder = SqlBuilder::new(&ty, registry);
    let mapper = sql_builder.mapper_for_query_path(query_path)?;
    let canonical_table_alias = &mapper.canonical_table_alias;
    let table_name = &mapper.table_name;

    let aux_params = [backend.aux_params()];
    let aux_params_map = ParameterMap::new(&aux_params);
    let resolver = Resolver::new()
        .with_aux_params(&aux_params_map)
        .with_self_alias(&canonical_table_alias);
    let columns_sql = resolver
        .to_sql(&columns_expr, &mut alias_translator)
        .map_err(ToqlError::from)?;
    let values_sql = resolver
        .to_sql(&values_expr, &mut alias_translator)
        .map_err(ToqlError::from)?;

    let mut insert_stmt = String::from("INSERT INTO ");
    insert_stmt.push_str(&table_name);
    insert_stmt.push(' ');
    insert_stmt.push_str(&columns_sql.0);
    insert_stmt.push_str(" VALUES ");
    insert_stmt.push_str(&values_sql.0);

    insert_stmt.pop(); // Remove ', '
    insert_stmt.pop();

    Ok(Some(Sql(insert_stmt, values_sql.1)))
}

pub fn plan_insert_order<T, S: AsRef<str>>(
    mappers: &HashMap<String, TableMapper>,
    paths: &[S],
    joins: &mut Vec<HashSet<String>>,
    merges: &mut HashSet<String>,
    partials: &mut Vec<HashSet<String>>,
) -> Result<()>
where
    T: Mapped,
{
    let ty = <T as Mapped>::type_name();
    // Add partials for root
    insert_partial_tables_order(mappers, &ty, 0, &FieldPath::default(), partials)?;

    for path in paths {
        let field_path = FieldPath::from(path.as_ref().trim_end_matches('_'));
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
                if partials.len() <= level {
                    partials.push(HashSet::new());
                }
                if !mapper.is_partial_join(c.as_str()) {
                    joins.get_mut(level).unwrap().insert(d.as_str().to_string());
                } else {
                    partials
                        .get_mut(level)
                        .unwrap()
                        .insert(d.as_str().to_string());
                    insert_partial_tables_order(mappers, &j, level + 1, &d, partials)?;
                }

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
                insert_partial_tables_order(mappers, &m, level, &d, partials)?;
            } else {
                return Err(SqlBuilderError::JoinMissing(
                    c.as_str().to_owned(),
                    mapper.table_name.to_string(),
                )
                .into());
            }
        }
    }

    Ok(())
}

fn insert_partial_tables_order(
    mappers: &HashMap<String, TableMapper>,
    mapper_name: &str,
    level: usize,
    query_path: &FieldPath,
    joins_or_merges: &mut Vec<HashSet<String>>,
) -> Result<()> {
    let mapper = mappers
        .get(mapper_name)
        .ok_or_else(|| ToqlError::MapperMissing(mapper_name.to_owned()))?;
    let partial_joins: Vec<(String, String)> = mapper.joined_partial_mappers();

    for (path, mapper_name) in &partial_joins {
        let qp = query_path.append(path);
        if joins_or_merges.len() <= level {
            joins_or_merges.push(HashSet::new())
        }
        insert_partial_tables_order(&mappers, &mapper_name, level + 1, &qp, joins_or_merges)?;
        joins_or_merges
            .get_mut(level)
            .unwrap()
            .insert(qp.as_str().to_string());
    }

    Ok(())
}
