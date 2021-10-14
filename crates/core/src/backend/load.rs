use super::{map, Backend};
use crate::{
    alias_translator::AliasTranslator,
    error::ToqlError,
    from_row::FromRow,
    keyed::Keyed,
    page::Page,
    parameter_map::ParameterMap,
    query::{field_path::FieldPath, Query},
    sql::Sql,
    sql_builder::SqlBuilder,
    sql_expr::{resolver::Resolver, PredicateColumn, SqlExpr},
    table_mapper::mapped::Mapped,
    tree::{tree_index::TreeIndex, tree_merge::TreeMerge, tree_predicate::TreePredicate},
};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

use crate::page_counts::PageCounts;
use crate::toql_api::load::Load;

pub async fn load<B, Q, T, R, E>(
    backend: &mut B,
    query: Q,
    page: Option<Page>,
) -> std::result::Result<(Vec<T>, Option<PageCounts>), E>
where
    B: Backend<R, E>,
    E: From<ToqlError>,
    T: Load<R, E> + Send,
    Q: Borrow<Query<T>> + Sync + Send,
    <T as Keyed>::Key: FromRow<R, E>,
{
    {
        let registry = &mut *backend.registry_mut()?;
        map::map::<T>(registry)?;
    }

    let (mut entities, unmerged_paths, counts) = load_top(backend, &query, page).await?;
    if !entities.is_empty() {
        let mut pending_paths = unmerged_paths;
        loop {
            pending_paths = load_and_merge(backend, &query, &mut entities, &pending_paths).await?;

            // Quit, if all paths have been merged
            if pending_paths.is_empty() {
                break;
            }

            // Select and merge next paths
            // unmerged_paths.extend(pending_paths.drain());
        }
    }

    Ok((entities, counts))
}

async fn load_and_merge<B, Q, T, R, E>(
    backend: &mut B,
    query: &Q,
    entities: &mut Vec<T>,
    unmerged_home_paths: &HashSet<String>,
) -> std::result::Result<HashSet<String>, E>
where
    B: Backend<R, E>,
    T: Load<R, E>,
    Q: Borrow<Query<T>> + Sync,
    <T as crate::keyed::Keyed>::Key: FromRow<R, E>,
    E: From<ToqlError>,
{
    let ty = <T as Mapped>::type_name();
    let mut pending_home_paths = HashSet::new();

    let canonical_base = {
        let registry = backend.registry()?;
        let mapper = registry
            .mappers
            .get(&ty)
            .ok_or(ToqlError::MapperMissing(ty.clone()))?;
        mapper.canonical_table_alias.clone()
    };

    for home_path in unmerged_home_paths {
        // Get merge JOIN with ON from mapper
        let hp = FieldPath::from(&home_path);
        let parent_home_path = hp.ancestors().nth(1); // Skip unchanged value

        let merge_base_alias = if let Some(hp) = &parent_home_path {
            format!("{}_{}", &canonical_base, hp.to_string())
        } else {
            canonical_base.to_string()
        };

        let mut result = {
            let registry = backend.registry()?;
            let mut builder = SqlBuilder::new(&ty, &*registry)
                .with_aux_params(backend.aux_params().clone()) // todo ref
                .with_roles(backend.roles().clone()); // todo ref// Add alias format or translator to constructor
            builder.build_select(home_path.as_str(), query.borrow())?
        };

        pending_home_paths = result.unmerged_home_paths().clone();

        let other_alias = result.table_alias().clone();
        let merge_resolver = Resolver::new()
            .with_self_alias(&merge_base_alias)
            .with_other_alias(other_alias.as_str());

        // Build merge join
        // Get merge join and custom on predicate from mapper
        let (mut merge_join_sql_expr, merge_join_predicate) = {
            let registry = backend.registry()?;
            let builder = SqlBuilder::new(&ty, &*registry)
                .with_aux_params(backend.aux_params().clone()) // TODO ref
                .with_roles(backend.roles().clone());
            builder.merge_expr(&home_path)?
        };

        let merge_join_predicate = merge_resolver
            .resolve(&merge_join_predicate)
            .map_err(ToqlError::from)?;

        // Get key columns
        let (merge_join, key_select_expr) = {
            let parent_home_path = parent_home_path.unwrap_or_default();
            let registry = backend.registry()?;
            let builder = SqlBuilder::new(&ty, &*registry); // No aux params for key
            let (key_select_expr, key_join) =
                builder.columns_expr(parent_home_path.as_str(), &merge_base_alias)?;

            let merge_join = if key_join.is_empty() {
                &merge_join_sql_expr
            } else {
                merge_join_sql_expr.push_literal(" ").extend(key_join)
            };

            (
                merge_resolver
                    .resolve(merge_join)
                    .map_err(ToqlError::from)?,
                key_select_expr,
            )
        };

        result.set_preselect(key_select_expr); // Select key columns for indexing
        result.push_join(merge_join);
        result.push_join(SqlExpr::literal(" ON ("));
        result.push_join(merge_join_predicate);

        // Get ON predicate from entity keys
        let mut predicate_expr = SqlExpr::new();
        let ancestor_path = FieldPath::trim_basename(home_path.as_str());
        // let ancestor_path = ancestor_path.unwrap_or(FieldPath::from(""));
        //let mut d = ancestor_path.descendents();
        let mut d = ancestor_path.children();

        let columns = <T as TreePredicate>::columns(&mut d).map_err(ToqlError::from)?;

        let mut args = Vec::new();
        //let mut d = ancestor_path.descendents();

        for e in entities.iter() {
            let d = ancestor_path.children();
            TreePredicate::args(e, d, &mut args).map_err(ToqlError::from)?;
        }
        let rows = if args.is_empty() {
            Vec::new()
        } else {
            let predicate_columns = columns
                .into_iter()
                .map(|c| PredicateColumn::SelfAliased(c))
                .collect::<Vec<_>>();
            predicate_expr.push_predicate(predicate_columns, args);

            let predicate_expr = {
                let merge_resolver = Resolver::new()
                    .with_self_alias(&merge_base_alias)
                    .with_other_alias(other_alias.as_str());
                merge_resolver
                    .resolve(&predicate_expr)
                    .map_err(ToqlError::from)?
            };
            result.push_join(SqlExpr::literal(" AND "));
            result.push_join(predicate_expr);
            result.push_join(SqlExpr::literal(")"));

            // Build SQL query statement

            let mut alias_translator = AliasTranslator::new(backend.alias_format());
            let aux_params = [backend.aux_params()];
            let aux_params = ParameterMap::new(&aux_params);
            let sql = result
                .to_sql(&aux_params, &mut alias_translator)
                .map_err(ToqlError::from)?;

            // Load from database
            backend.select_sql(sql).await? // Default vector size
        };

        // Build index
        let mut index: HashMap<u64, Vec<usize>> = HashMap::new(); // Hashed key, row array positions

        let (ancestor_path, field) = FieldPath::split_basename(home_path.as_str());

        // TODO Batch process rows
        // TODO Introduce traits that do not need to copy into vec

        let row_offset = 0; // Key must be first column(s) in row

        // Build up index to provide fast lookup for the merge function
        <T as TreeIndex<R, E>>::index(ancestor_path.children(), &rows, row_offset, &mut index)?;

        // Merge into entities
        for e in entities.iter_mut() {
            <T as TreeMerge<_, E>>::merge(
                e,
                ancestor_path.children(),
                field,
                &rows,
                row_offset,
                &index,
                result.select_stream(),
            )?;
        }
    }
    Ok(pending_home_paths)
}

async fn load_top<B, Q, T, R, E>(
    backend: &mut B,
    query: &Q,
    page: Option<Page>,
) -> std::result::Result<(Vec<T>, HashSet<String>, Option<PageCounts>), E>
where
    B: Backend<R, E>,
    T: Load<R, E> + Send + FromRow<R, E>,
    Q: Borrow<Query<T>> + Sync + Send,
    <T as crate::keyed::Keyed>::Key: FromRow<R, E>,
    E: From<ToqlError>,
{
    let alias_format = backend.alias_format();

    let ty = <T as Mapped>::type_name();

    let (mut result, count_result) = {
        let registry = &*backend.registry()?;
        tracing::event!(tracing::Level::INFO, query =  %query.borrow(), "Building Sql for Toql query.");

        let mut builder = SqlBuilder::new(&ty, registry)
            .with_aux_params(backend.aux_params().clone()) // todo ref
            .with_roles(backend.roles().clone()); // todo ref;
        let result = builder.build_select("", query.borrow())?;
        let count_result = if matches!(page, Some(Page::Counted(_, _))) {
            let count_result = builder.build_count("", query.borrow(), true)?;
            Some(count_result)
        } else {
            None
        };
        (result, count_result)
    };

    let unmerged = result.unmerged_home_paths().clone();
    let mut alias_translator = AliasTranslator::new(alias_format);

    let sql = {
        let aux_params = [backend.aux_params()];
        let aux_params = ParameterMap::new(&aux_params);
        if let Some(p) = &page {
            backend.prepare_page(&mut result, p);
        }
        result
            .to_sql(&aux_params, &mut alias_translator)
            .map_err(ToqlError::from)?
    };

    let entities = {
        let rows = backend.select_sql(sql).await?;
        let mut entities = Vec::with_capacity(rows.len());

        for r in rows {
            let mut iter = result.select_stream().iter();
            let mut i = 0usize;
            if let Some(e) = <T as FromRow<R, E>>::from_row(&r, &mut i, &mut iter)? {
                entities.push(e);
            }
        }
        entities
    };

    let page_counts = if let Some(count_result) = count_result {
        let count_sql = Sql::new(); // TODO for postgres
        let filtered = backend.select_max_page_size_sql(count_sql).await?;

        let total_page_size_sql = {
            let aux_params = [backend.aux_params()];
            let aux_params = ParameterMap::new(&aux_params);
            count_result
                .to_sql(&aux_params, &mut alias_translator)
                .map_err(|e| e.into())?
        };
        let total = backend.select_count_sql(total_page_size_sql).await?;
        Some(PageCounts { filtered, total })
    } else {
        None
    };

    Ok((entities, unmerged, page_counts))
}
