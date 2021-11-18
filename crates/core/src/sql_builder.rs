//! The SQL builder can build different select statements.
pub mod build_result;
pub mod select_stream;
pub mod sql_builder_error;

pub(crate) mod build_context;
pub(crate) mod path_tree;

use crate::{
    error::ToqlError,
    parameter_map::ParameterMap,
    query::{
        concatenation::Concatenation, field_order::FieldOrder, field_path::FieldPath,
        query_token::QueryToken, Query,
    },
    result::Result,
    role_validator::RoleValidator,
    sql_arg::SqlArg,
    sql_builder::{
        build_context::BuildContext, build_result::BuildResult, sql_builder_error::SqlBuilderError,
    },
    sql_expr::{resolver::Resolver, SqlExpr},
    table_mapper::{join_type::JoinType, DeserializeType, TableMapper},
    table_mapper_registry::TableMapperRegistry,
};

use path_tree::PathTree;
use select_stream::Select;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

enum MapperOrMerge<'a> {
    Mapper(&'a TableMapper),
    Merge(String),
}

/// The Sql builder to build normal queries and count queries.
pub struct SqlBuilder<'a> {
    root_mapper: String, // root type
    home_mapper: String, // home mapper, depends on query root
    table_mapper_registry: &'a TableMapperRegistry,
    roles: HashSet<String>,
    aux_params: HashMap<String, SqlArg>, // Aux params used for all queries with this builder instance, contains typically config or auth data
    extra_joins: HashSet<String>,        // Use this joins
}

impl<'a> SqlBuilder<'a> {
    /// Create a new SQL Builder from a root mapper and the table mapper registry
    pub fn new(root_mapper: &'a str, table_mapper_registry: &'a TableMapperRegistry) -> Self {
        SqlBuilder {
            root_mapper: root_mapper.to_string(),
            home_mapper: root_mapper.to_string(),
            table_mapper_registry,
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            extra_joins: HashSet::new(),
        }
    }
    /// Use these roles with the builder.
    pub fn with_roles(mut self, roles: HashSet<String>) -> Self {
        self.roles = roles;
        self
    }
    /// Use these auxiliary parameters with the builder.
    pub fn with_aux_params(mut self, aux_params: HashMap<String, SqlArg>) -> Self {
        self.aux_params = aux_params;
        self
    }
    /// Add this raw SQL join statement to the result.
    /// (For internal merge joins)
    pub fn with_extra_join<T: Into<String>>(mut self, join: T) -> Self {
        self.extra_joins.insert(join.into());
        self
    }

    pub fn columns_expr(&self, query_field_path: &str, alias: &str) -> Result<(SqlExpr, SqlExpr)> {
        let mut columns_expr = SqlExpr::new();
        let mut join_expr = SqlExpr::new();

        self.resolve_columns_expr(
            query_field_path,
            alias,
            &mut columns_expr,
            &mut join_expr,
            //    &mut on_expr,
        )?;

        Ok((columns_expr, join_expr))
    }
    fn resolve_columns_expr(
        &self,
        query_path: &str,
        alias: &str,
        columns_expr: &mut SqlExpr,
        join_expr: &mut SqlExpr,
        //   on_expr: &mut SqlExpr,
    ) -> Result<()> {
        let mapper = self.mapper_for_query_path(&FieldPath::from(query_path))?;

        for order in &mapper.deserialize_order {
            match order {
                DeserializeType::Field(name) => {
                    let field = mapper
                        .field(&name)
                        .ok_or_else(|| SqlBuilderError::FieldMissing(name.to_string()))?;
                    if !field.options.key {
                        return Ok(());
                    }
                    let resolver = Resolver::new().with_self_alias(alias);
                    if !columns_expr.is_empty() {
                        columns_expr.push_literal(", ");
                    }
                    columns_expr.extend(resolver.resolve(&field.expression)?);
                }
                DeserializeType::Join(name) => {
                    let join = mapper.join(&name).ok_or_else(|| {
                        SqlBuilderError::JoinMissing(
                            name.to_string(),
                            mapper.table_name.to_string(),
                        )
                    })?;
                    if !join.options.key {
                        return Ok(());
                    }
                    let other_alias = FieldPath::from(alias).append(&mapper.canonical_table_alias);
                    let resolver = Resolver::new()
                        .with_self_alias(alias)
                        .with_other_alias(&other_alias);

                    join_expr.push_literal("JOIN ");
                    join_expr.extend(resolver.resolve(&join.table_expression)?);
                    join_expr.push_literal(" ON (");
                    join_expr.extend(resolver.resolve(&join.on_expression)?);
                    join_expr.push_literal(") ");

                    let joined_query_path = FieldPath::from(query_path).append(&name);

                    self.resolve_columns_expr(
                        &joined_query_path,
                        &other_alias,
                        columns_expr,
                        join_expr,
                        //     on_expr,
                    )?;
                }

                DeserializeType::Merge(_) => {}
            }
        }
        Ok(())
    }

    pub fn merge_expr(&self, query_field_path: &str) -> Result<(SqlExpr, SqlExpr)> {
        let (query_path, basename) = FieldPath::split_basename(query_field_path);
        let mapper = self.mapper_for_query_path(&query_path)?;

        // Get merge join statement and on predicate
        let merge = mapper.merge(basename).ok_or(ToqlError::NotFound)?;

        Ok((
            merge.merge_join.to_owned(),
            merge.merge_predicate.to_owned(),
        ))
    }

    /// Build a delete statement from the [Query].
    /// This build a delete filter predicate from the field filters and predicates in the query.
    /// Any field selections are ignored.
    ///
    /// Returns a [BuildResult] that can be turned into SQL.
    pub fn build_delete<M>(&mut self, query: &Query<M>) -> Result<BuildResult> {
        let mut context = BuildContext::new();
        let root_mapper = self
            .table_mapper_registry
            .mappers
            .get(&self.home_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_owned()))?;

        if let Some(role_expr) = &root_mapper.delete_role_expr {
            if !RoleValidator::is_valid(&self.roles, role_expr) {
                return Err(SqlBuilderError::RoleRequired(
                    role_expr.to_string(),
                    self.home_mapper.to_string(),
                )
                .into());
            }
        }

        let mut result = BuildResult::new(SqlExpr::literal("DELETE"));

        result.set_from(
            root_mapper.table_name.to_owned(),
            root_mapper.canonical_table_alias.to_owned(),
        );
        self.preparse_filter_joins(&query, &mut context, false)?;
        self.build_where_clause(&query, &mut context, false, &mut result)?;
        self.build_join_clause(&query.aux_params, &mut context, &mut result, true, false)?;

        Ok(result)
    }

    //TODO move function itno separate unit
    pub fn build_merge_delete(
        &mut self,
        merge_path: &FieldPath,
        key_predicate: SqlExpr,
    ) -> Result<SqlExpr> {
        let root_mapper = self
            .table_mapper_registry
            .mappers
            .get(&self.home_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_owned()))?;

        let (query_path, merge_field) = FieldPath::split_basename(&merge_path);

        let base_mapper = self.joined_mapper_for_local_path(&query_path)?;
        let root_path = FieldPath::from(&root_mapper.canonical_table_alias);
        let canonical_path = root_path.append(&query_path);

        let merge = base_mapper
            .merge(merge_field)
            .ok_or_else(|| SqlBuilderError::FieldMissing(merge_field.to_string()))?;

        let merge_mapper = self
            .table_mapper_registry
            .mappers
            .get(&merge.merged_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(merge.merged_mapper.to_string()))?;

        let mut delete_expr = SqlExpr::new();

        // TODO move into backend
        // Mysql specific
        delete_expr.push_literal("DELETE ");
        delete_expr.push_other_alias();
        delete_expr.push_literal(" FROM ");
        delete_expr.push_literal(&merge_mapper.table_name);
        delete_expr.push_literal(" ");
        delete_expr.push_other_alias();
        delete_expr.push_literal(" ");
        delete_expr.extend(merge.merge_join.clone()); // Maybe conctruct custom join for postgres
        delete_expr.push_literal(" ON ");
        delete_expr.extend(merge.merge_predicate.clone());
        delete_expr.push_literal(" WHERE ");
        delete_expr.extend(key_predicate);

        let canonical_merge_alias = canonical_path.append(merge_field);
        let resolver = Resolver::new()
            .with_self_alias(&canonical_path)
            .with_other_alias(&canonical_merge_alias);

        resolver.resolve(&delete_expr).map_err(ToqlError::from)
    }
    pub fn build_merge_key_select(
        &mut self,
        merge_path: &FieldPath,
        key_selects: SqlExpr,
        key_predicate: SqlExpr,
    ) -> Result<SqlExpr> {
        let root_mapper = self
            .table_mapper_registry
            .mappers
            .get(&self.home_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_owned()))?;

        // TODO maybe alias wrong, see update build_merge_delete
        let (base_path, merge_field) = FieldPath::split_basename(&merge_path);

        let base_mapper = self.joined_mapper_for_local_path(&base_path)?;

        let root_path = FieldPath::from(&root_mapper.canonical_table_alias);
        let path = if base_path.is_empty() {
            root_path
        } else {
            base_path
        };
        let self_field = FieldPath::trim_basename(&path);

        let merge = base_mapper
            .merge(merge_field)
            .ok_or_else(|| SqlBuilderError::FieldMissing(merge_field.to_string()))?;

        let merge_mapper = self
            .table_mapper_registry
            .mappers
            .get(&merge.merged_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(merge.merged_mapper.to_string()))?;

        let mut delete_expr = SqlExpr::new();

        // TODO move into backend
        // Mysql specific
        delete_expr.push_literal("SELECT ");
        delete_expr.extend(key_selects);
        delete_expr.push_literal(" FROM ");
        delete_expr.push_literal(&merge_mapper.table_name);
        delete_expr.push_literal(" ");
        delete_expr.push_other_alias();
        delete_expr.push_literal(" ");
        delete_expr.extend(merge.merge_join.clone()); // Maybe conctruct custom join for postgres
        delete_expr.push_literal(" ON ");
        delete_expr.extend(merge.merge_predicate.clone());
        delete_expr.push_literal(" WHERE ");
        delete_expr.extend(key_predicate);

        let merge_field = format!("{}_{}", self_field.as_str(), merge_field);
        let resolver = Resolver::new()
            .with_self_alias(self_field.as_str())
            .with_other_alias(&merge_field);

        resolver.resolve(&delete_expr).map_err(ToqlError::from)
    }

    /// Build a normal select statement from the [Query].
    ///
    /// Returns a [BuildResult] that can be turned into SQL.
    pub fn build_select<M>(
        &mut self,
        query_home_path: &str,
        query: &Query<M>,
    ) -> Result<BuildResult> {
        let mut context = BuildContext::new();
        context.query_home_path = query_home_path.to_string();

        self.set_home_joined_mapper_for_path(&FieldPath::from(query_home_path))?;

        let mapper = self
            .table_mapper_registry
            .get(&self.home_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_string()))?;

        if let Some(role) = mapper.load_role_expr.as_ref() {
            if !RoleValidator::is_valid(&self.roles, role) {
                return Err(SqlBuilderError::RoleRequired(
                    role.to_string(),
                    query_home_path.to_string(),
                )
                .into());
            }
        }

        let mut result = BuildResult::new(SqlExpr::literal("SELECT"));
        result.set_from(
            mapper.table_name.to_owned(),
            mapper.canonical_table_alias.to_owned(),
        );

        self.preparse_query(&query, &mut context, &mut result)?;
        self.build_where_clause(&query, &mut context, false, &mut result)?;
        self.build_select_clause(&query, &mut context, &mut result)?;
        self.build_join_clause(&query.aux_params, &mut context, &mut result, false, true)?;
        self.build_order_clause(&query.aux_params, &mut context, &mut result)?;

        Ok(result)
    }

    /// Build a count statement from the [Query].
    /// This build a count filter predicate from the field filters and predicates.
    /// If `count_selection_ony` is true then only filters are used that are part
    /// of the count selection ($cnt) or predicates that are marked as count_filters.
    ///
    /// Returns a [BuildResult] that can be turned into SQL.
    pub fn build_count<M>(
        &mut self,
        query_root_path: &str,
        query: &Query<M>,
        count_selection_only: bool,
    ) -> Result<BuildResult> {
        let mut build_context = BuildContext::new();
        build_context.query_home_path = query_root_path.to_string();
        let root_mapper = self.root_mapper()?; // self.joined_mapper_for_path(&Self::root_field_path(root_path))?;

        let mut result = BuildResult::new(SqlExpr::literal("SELECT"));
        result.select_expr.push_literal("COUNT(*)");

        result.set_from(
            root_mapper.table_name.to_owned(),
            root_mapper.canonical_table_alias.to_owned(),
        );

        self.build_where_clause(
            &query,
            &mut build_context,
            count_selection_only,
            &mut result,
        )?;

        self.preparse_filter_joins(&query, &mut build_context, count_selection_only)?;

        self.build_join_clause(
            &query.aux_params,
            &mut build_context,
            &mut result,
            true,
            true,
        )?;

        Ok(result)
    }

    pub fn joined_mapper_for_local_path(&self, local_path: &FieldPath) -> Result<&TableMapper> {
        self.joined_mapper_for_path(&self.home_mapper, local_path)
    }
    pub fn joined_mapper_for_query_path(&self, query_path: &FieldPath) -> Result<&TableMapper> {
        self.joined_mapper_for_path(&self.root_mapper, query_path)
    }
    fn joined_mapper_for_path(&self, mapper_name: &str, path: &FieldPath) -> Result<&TableMapper> {
        let mut current_mapper = self
            .table_mapper_registry
            .get(mapper_name)
            .ok_or_else(|| ToqlError::MapperMissing(mapper_name.to_string()))?;

        if !path.is_empty() {
            for p in path.children() {
                if let Some(join) = current_mapper.joins.get(p.as_str()) {
                    current_mapper = self
                        .table_mapper_registry
                        .get(&join.joined_mapper)
                        .ok_or_else(|| ToqlError::MapperMissing(join.joined_mapper.to_string()))?;
                } else {
                    return Err(SqlBuilderError::JoinMissing(
                        p.to_string(),
                        current_mapper.table_name.to_string(),
                    )
                    .into());
                }
            }
        }

        Ok(current_mapper)
    }

    pub fn mapper_for_query_path(&self, query_path: &FieldPath) -> Result<&TableMapper> {
        let mut current_mapper = self
            .table_mapper_registry
            .get(&self.root_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.root_mapper.to_string()))?;

        if !query_path.is_empty() {
            for p in query_path.children() {
                if let Some(join) = current_mapper.joins.get(p.as_str()) {
                    current_mapper = self
                        .table_mapper_registry
                        .get(&join.joined_mapper)
                        .ok_or_else(|| ToqlError::MapperMissing(join.joined_mapper.to_string()))?;
                } else if let Some(merge) = current_mapper.merges.get(p.as_str()) {
                    current_mapper = self
                        .table_mapper_registry
                        .get(&merge.merged_mapper)
                        .ok_or_else(|| ToqlError::MapperMissing(merge.merged_mapper.to_string()))?;
                } else {
                    return Err(ToqlError::MapperMissing(p.to_string()));
                }
            }
        }

        Ok(current_mapper)
    }

    fn mapper_or_merge_for_path(&'a self, local_path: &'a FieldPath) -> Result<MapperOrMerge<'a>> {
        let mut current_mapper = self
            .table_mapper_registry
            .get(&self.home_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_string()))?;

        if !local_path.is_empty() {
            for (p, a) in local_path.children().zip(local_path.step_down()) {
                //  dbg!(&a);
                if current_mapper.merges.contains_key(p.as_str()) {
                    return Ok(MapperOrMerge::Merge(a.to_string()));
                }
                let join = current_mapper
                    .joins
                    .get(p.as_str())
                    .ok_or_else(|| ToqlError::MapperMissing(p.to_string()))?;
                current_mapper = self
                    .table_mapper_registry
                    .get(&join.joined_mapper)
                    .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_string()))?;
            }
        }

        Ok(MapperOrMerge::Mapper(current_mapper))
    }
    fn set_home_joined_mapper_for_path(&mut self, path: &FieldPath) -> Result<()> {
        if !path.is_empty() {
            let mut current_type: &str = &self.root_mapper;
            let mut current_mapper = self
                .table_mapper_registry
                .get(current_type)
                .ok_or_else(|| ToqlError::MapperMissing(current_type.to_string()))?;

            for p in path.children() {
                if let Some(merge) = current_mapper.merges.get(p.as_str()) {
                    current_mapper = self
                        .table_mapper_registry
                        .get(&merge.merged_mapper)
                        .ok_or_else(|| ToqlError::MapperMissing(merge.merged_mapper.to_string()))?;
                    current_type = &merge.merged_mapper;
                } else if let Some(join) = current_mapper.joins.get(p.as_str()) {
                    current_mapper = self
                        .table_mapper_registry
                        .get(&join.joined_mapper)
                        .ok_or_else(|| ToqlError::MapperMissing(join.joined_mapper.to_string()))?;
                    current_type = &join.joined_mapper;
                } else {
                    return Err(ToqlError::MapperMissing(p.to_string()));
                }
            }

            self.home_mapper = current_type.to_string();
        }

        Ok(())
    }

    fn build_join_clause(
        &self,
        query_aux_params: &HashMap<String, SqlArg>,
        mut build_context: &mut BuildContext,
        result: &mut BuildResult,
        enforce_inner_joins: bool,
        restrict_load: bool,
    ) -> Result<()> {
        // Build join tree for all selected paths
        // This allows to nest joins properly
        // Eg [user] = [user_address, user_folder]
        // [user_folder] = [ user_folder_owner]
        // [user_folder_owner] =[]
        // [user address] =[]

        let mut join_tree = PathTree::new();

        for local_path in &build_context.local_joined_paths {
            join_tree.insert(&FieldPath::from(&local_path));
        }

        // Build join
        let expr: SqlExpr = self.resolve_join(
            &join_tree,
            &join_tree.roots(),
            &mut build_context,
            enforce_inner_joins,
            restrict_load,
            query_aux_params,
        )?;
        result.join_expr.extend(expr);
        result.join_expr.pop_literals(1); // Remove trailing whitespace

        Ok(())
    }
    fn resolve_join(
        &self,
        join_tree: &PathTree,
        nodes: &HashSet<String>,
        build_context: &mut BuildContext,
        enforce_inner_joins: bool,
        restrict_load: bool,
        query_aux_params: &HashMap<String, SqlArg>,
    ) -> Result<SqlExpr> {
        let mut join_expr = SqlExpr::new();

        for local_path_with_join in nodes {
            let (local_path, join_name) = FieldPath::split_basename(local_path_with_join);

            let local_mapper = self.joined_mapper_for_local_path(&local_path)?;

            let join = local_mapper.join(join_name).ok_or_else(|| {
                SqlBuilderError::JoinMissing(
                    join_name.to_string(),
                    local_mapper.table_name.to_string(),
                )
            })?;
            if restrict_load {
                let joined_mapper = self
                    .table_mapper_registry
                    .get(&join.joined_mapper)
                    .ok_or_else(|| ToqlError::MapperMissing(join.joined_mapper.to_string()))?;

                if let Some(role) = joined_mapper.load_role_expr.as_ref() {
                    if !RoleValidator::is_valid(&self.roles, role) {
                        return Err(SqlBuilderError::RoleRequired(
                            role.to_string(),
                            local_path.to_string(),
                        )
                        .into());
                    }
                }
            }

            let canonical_self_alias = self.canonical_alias(&local_path)?.to_string();
            let canonical_other_alias = self
                .canonical_alias(&FieldPath::from(local_path_with_join))?
                .to_string();
            let resolver = Resolver::new()
                .with_self_alias(&canonical_self_alias)
                .with_other_alias(&canonical_other_alias);

            join_expr.push_literal(if enforce_inner_joins {
                "JOIN ("
            } else {
                match &join.join_type {
                    JoinType::Inner => "JOIN (",
                    JoinType::Left => "LEFT JOIN (",
                }
            });
            let join_e = resolver.resolve(&join.table_expression)?;
            join_expr.extend(join_e);
            join_expr.push_literal(" ");

            if let Some(subnodes) = join_tree.nodes(local_path_with_join) {
                if !subnodes.is_empty() {
                    let subjoin_expr = self.resolve_join(
                        join_tree,
                        &subnodes,
                        build_context,
                        enforce_inner_joins,
                        restrict_load,
                        query_aux_params,
                    )?;
                    if !subjoin_expr.is_empty() {
                        join_expr.extend(subjoin_expr);
                    }
                }
            }
            join_expr.pop_literals(1); // Remove trailing whitespace
            join_expr.push_literal(") ON (".to_string());

            let on_expr = resolver.resolve(&join.on_expression)?;

            let on_expr = {
                let p = [
                    &self.aux_params,
                    &join.options.aux_params,
                    &build_context.on_aux_params,
                    &query_aux_params,
                ];
                let aux_params = ParameterMap::new(&p);
                match &join.options.join_handler {
                    Some(handler) => handler.build_on_predicate(on_expr, &aux_params)?,
                    None => Resolver::resolve_aux_params(on_expr, &aux_params),
                }
            };

            // Skip left joins with unresolved aux params
            let on_expr = match on_expr.first_aux_param() {
                Some(p) if join.join_type == JoinType::Left => {
                    let query_path_with_join = FieldPath::from(&build_context.query_home_path)
                        .append(local_path_with_join);
                    tracing::info!("Setting condition of left join `{}` to `false`, because aux param `{}` is missing", query_path_with_join.as_str(), &p );
                    SqlExpr::literal("false")
                }
                _ => on_expr,
            };

            join_expr.extend(on_expr);
            join_expr.push_literal(") ");
        }

        Ok(join_expr)
    }

    fn build_where_clause<M>(
        &mut self,
        query: &Query<M>,
        build_context: &mut BuildContext,
        count_selection_only: bool,
        result: &mut BuildResult,
    ) -> Result<()> {
        let p = [&self.aux_params, &query.aux_params];
        let aux_params = ParameterMap::new(&p);

        for token in &query.tokens {
            match token {
                QueryToken::Field(field) => {
                    // Continue if field is not filtered
                    if field.filter.is_none() {
                        continue;
                    }
                    let (query_path, field_name) = FieldPath::split_basename(&field.name);

                    // skip if field path is not relative to root path
                    if !Self::home_contains(&build_context.query_home_path, &query_path) {
                        continue;
                    }

                    if count_selection_only {
                        let root_mapper = self.root_mapper()?;
                        match root_mapper.selections.get("cnt") {
                            Some(selection) => {
                                let wildcard_path = format!("{}_*", field.name.as_str());
                                if !selection.contains(&field.name)
                                    && !selection.contains(&wildcard_path)
                                {
                                    continue;
                                }
                            }
                            None => continue,
                        }
                    }

                    // Get relative path
                    let local_path = match query_path.localize_path(&build_context.query_home_path)
                    {
                        Some(l) => l,
                        None => return Ok(()),
                    };

                    let mapper_or_merge = self.mapper_or_merge_for_path(&local_path)?;

                    match mapper_or_merge {
                        MapperOrMerge::Mapper(mapper) => {
                            let mapped_field = mapper.fields.get(field_name).ok_or_else(|| {
                                SqlBuilderError::FieldMissing(field.name.to_string())
                            })?;

                            if let Some(role_expr) = &mapped_field.options.load_role_expr {
                                if !crate::role_validator::RoleValidator::is_valid(
                                    &self.roles,
                                    role_expr,
                                ) {
                                    return Err(SqlBuilderError::RoleRequired(
                                        role_expr.to_string(),
                                        FieldPath::from(&build_context.query_home_path)
                                            .append(field_name)
                                            .to_string(),
                                    )
                                    .into());
                                }
                            }
                            let canonical_alias = self.canonical_alias(&query_path)?;

                            let p = [
                                &self.aux_params,
                                &query.aux_params,
                                &mapped_field.options.aux_params,
                            ];
                            let aux_params = ParameterMap::new(&p);

                            let handler = mapped_field
                                .options
                                .field_handler
                                .as_ref()
                                .unwrap_or(&mapper.field_handler);
                            let select_expr = handler
                                .build_select(mapped_field.expression.clone(), &aux_params)?
                                .unwrap_or_default();

                            // Does filter apply
                            if let Some(expr) = handler.build_filter(
                                select_expr,
                                field.filter.as_ref().unwrap(),
                                &aux_params,
                            )? {
                                let resolver = Resolver::new().with_self_alias(&canonical_alias);
                                let expr = resolver.resolve(&expr)?;
                                if !result.where_expr.is_empty()
                                    && !result.where_expr.ends_with_literal("(")
                                {
                                    result.where_expr.push_literal(
                                        if field.concatenation == Concatenation::And {
                                            " AND "
                                        } else {
                                            " OR "
                                        },
                                    );
                                }
                                result.where_expr.extend(expr);
                            }
                        }
                        MapperOrMerge::Merge(_merge_path) => {
                            // result.unmerged_paths.insert(merge_path);
                        }
                    }
                }

                QueryToken::Predicate(predicate) => {
                    let (query_path, basename) = FieldPath::split_basename(&predicate.name);

                    // skip if field path is not relative to root path
                    if !Self::home_contains(&build_context.query_home_path, &query_path) {
                        continue;
                    }

                    let local_path = match query_path.localize_path(&build_context.query_home_path)
                    {
                        Some(l) => l,
                        None => return Ok(()),
                    };

                    let mapper_or_merge = self.mapper_or_merge_for_path(&local_path)?;

                    match mapper_or_merge {
                        MapperOrMerge::Mapper(mapper) => {
                            let mapped_predicate =
                                mapper.predicates.get(basename).ok_or_else(|| {
                                    SqlBuilderError::PredicateMissing(basename.to_string())
                                })?;

                            if count_selection_only && !mapped_predicate.options.count_filter {
                                continue;
                            }

                            if let Some(role) = &mapped_predicate.options.load_role_expr {
                                if !RoleValidator::is_valid(&self.roles, role) {
                                    return Err(SqlBuilderError::RoleRequired(
                                        role.to_string(),
                                        query_path.to_string(),
                                    )
                                    .into());
                                }
                            }

                            let canonical_alias = self.canonical_alias(&local_path)?;

                            let resolver = Resolver::new()
                                .with_self_alias(&canonical_alias)
                                .with_arguments(&predicate.args)
                                .with_aux_params(&aux_params);

                            let expr = resolver.resolve(&mapped_predicate.expression)?;
                            let handler = mapped_predicate
                                .options
                                .predicate_handler
                                .as_ref()
                                .unwrap_or(&mapper.predicate_handler);

                            if let Some(expr) =
                                handler.build_predicate(expr, &predicate.args, &aux_params)?
                            {
                                if !result.where_expr.is_empty()
                                    && !result.where_expr.ends_with_literal("(")
                                {
                                    result.where_expr.push_literal(
                                        if predicate.concatenation == Concatenation::And {
                                            " AND "
                                        } else {
                                            " OR "
                                        },
                                    );
                                }
                                result.where_expr.extend(expr);
                                if !mapped_predicate.options.on_aux_params.is_empty() {
                                    for (i, a) in &mapped_predicate.options.on_aux_params {
                                        if let Some(v) = predicate.args.get(*i as usize) {
                                            // tracing::info!("Setting on param `{}` = `{}`.", &a, v.to_string());
                                            build_context
                                                .on_aux_params
                                                .insert(a.clone(), v.clone());
                                        } else {
                                            tracing::warn!("Not enough predicate arguments to set on param `{}`.", &a);
                                        }
                                    }
                                }
                            }
                        }
                        MapperOrMerge::Merge(_merge_path) => {}
                    }
                }
                QueryToken::LeftBracket(concatenation) => {
                    // Omit concatenation if where expression is empty or left bracket follows an outer left bracket
                    if !result.where_expr.is_empty() && !result.where_expr.ends_with_literal("(") {
                        result
                            .where_expr
                            .push_literal(if concatenation == &Concatenation::And {
                                " AND "
                            } else {
                                " OR "
                            });
                    }
                    result.where_expr.push_literal("(");
                }
                QueryToken::RightBracket => {
                    // If parentheses are empty, remove right bracket and concatenation
                    if result.where_expr.ends_with_literal("(") {
                        result.where_expr.pop(); // Remove '(' token

                        // Remove ' AND ' or 'OR ' token if bracket is not inner bracket
                        // 'AND (' -> removed
                        // 'AND ((' -> reduced to 'AND ('
                        if result.where_expr.ends_with_literal(" AND ")
                            || result.where_expr.ends_with_literal(" OR ")
                        {
                            result.where_expr.pop();
                        }
                    } else {
                        result.where_expr.push_literal(")");
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn canonical_alias<'c>(&'c self, query_path: &'c FieldPath) -> Result<Cow<String>> {
        let root_alias = &self.root_mapper()?.canonical_table_alias;

        Ok(match query_path.is_empty() {
            false => Cow::Owned(query_path.prepend(&root_alias).to_string()),
            true => Cow::Borrowed(&root_alias),
        })
    }

    fn preparse_query<M>(
        &mut self,
        query: &Query<M>,
        build_context: &mut BuildContext,
        result: &mut BuildResult,
    ) -> Result<()> {
        result.unmerged_home_paths = self.selection_from_query(query, build_context)?;
        build_context.update_joins_from_selections();

        Ok(())
    }

    // Add recusivly all joins from a mapper to selected_paths
    fn add_all_joins_as_selected_paths(
        &self,
        mapper_name: &str,
        local_path: String,
        build_context: &mut BuildContext,
        unmerged_home_paths: &mut HashSet<String>,
    ) -> Result<()> {
        let mapper = self
            .table_mapper_registry
            .get(&mapper_name)
            .ok_or_else(|| ToqlError::MapperMissing(mapper_name.to_string()))?;
        for jm in &mapper.joins {
            let selected_path = FieldPath::from(&local_path).append(jm.0);
            // Resolve, if join is not yet resolved
            // Otherwise skip to avoid circular dependency
            if !build_context
                .local_selected_paths
                .contains(selected_path.as_str())
            {
                build_context
                    .local_selected_paths
                    .insert(selected_path.to_string());
                self.add_all_joins_as_selected_paths(
                    &jm.1.joined_mapper,
                    selected_path.to_string(),
                    build_context,
                    unmerged_home_paths,
                )?;
            }
        }
        for jm in &mapper.merges {
            let local_merge_path = FieldPath::from(&local_path).append(jm.0);
            let query_merge_path =
                FieldPath::from(&build_context.query_home_path).append(local_merge_path.as_str());
            // Resolve, if merge is not yet resolved
            // Otherwise skip to avoid circular dependency
            if !unmerged_home_paths.contains(query_merge_path.as_str()) {
                unmerged_home_paths.insert(query_merge_path.to_string());
                self.add_all_joins_as_selected_paths(
                    jm.0,
                    local_merge_path.to_string(),
                    build_context,
                    unmerged_home_paths,
                )?;
            }
        }
        Ok(())
    }

    fn build_order_clause(
        &mut self,
        query_aux_params: &HashMap<String, SqlArg>,
        build_context: &mut BuildContext,
        result: &mut BuildResult,
    ) -> Result<()> {
        let mut ordinals = Vec::with_capacity(build_context.ordering.len());
        for o in build_context.ordering.keys() {
            ordinals.push(o);
        }
        ordinals.sort();

        for n in ordinals {
            if let Some(orderings) = build_context.ordering.get(n) {
                for (ord, local_path_with_basename) in orderings {
                    let (local_path, field_name) =
                        FieldPath::split_basename(local_path_with_basename);
                    // Skip merge fields
                    if let Ok(mapper) = self.joined_mapper_for_local_path(&local_path) {
                        let struct_role_valid = mapper
                            .load_role_expr
                            .as_ref()
                            .map_or(true, |e| RoleValidator::is_valid(&self.roles, &e));
                        if !struct_role_valid {
                            return Err(SqlBuilderError::RoleRequired(
                                mapper
                                    .load_role_expr
                                    .as_ref()
                                    .map_or_else(|| String::new(), |e| e.to_string()),
                                FieldPath::from(&build_context.query_home_path)
                                    .append(&local_path)
                                    .to_string(),
                            )
                            .into());
                        }

                        let field_info = mapper
                            .field(field_name)
                            .ok_or_else(|| SqlBuilderError::FieldMissing(field_name.to_string()))?;

                        let role_valid =
                            if let Some(load_role_expr) = &field_info.options.load_role_expr {
                                RoleValidator::is_valid(&self.roles, load_role_expr)
                            } else {
                                true
                            };
                        if !role_valid {
                            let role_string = field_info
                                .options
                                .load_role_expr
                                .as_ref()
                                .map_or_else(|| String::new(), |e| e.to_string());

                            return Err(SqlBuilderError::RoleRequired(
                                role_string,
                                FieldPath::from(&build_context.query_home_path)
                                    .append(local_path_with_basename)
                                    .to_string(),
                            )
                            .into());
                        }
                        let p = [
                            &self.aux_params,
                            &field_info.options.aux_params,
                            query_aux_params,
                        ];
                        let aux_params = ParameterMap::new(&p);

                        let handler = field_info
                            .options
                            .field_handler
                            .as_ref()
                            .unwrap_or(&mapper.field_handler);
                        let select_expr =
                            handler.build_select(field_info.expression.clone(), &aux_params)?;
                        let canonical_alias = self.canonical_alias(&local_path)?;
                        if let Some(expr) = select_expr {
                            let resolver = Resolver::new().with_self_alias(&canonical_alias);
                            let expr = resolver.resolve(&expr)?;
                            result.order_expr.extend(expr);
                            result.order_expr.push_literal(match ord {
                                FieldOrder::Asc(_) => " ASC, ",
                                FieldOrder::Desc(_) => " DESC, ",
                            });
                        }
                    }
                }
            }
        }
        if !result.order_expr.is_empty() {
            result.order_expr.pop_literals(2); // Remove trailing ,
        }

        Ok(())
    }

    fn build_select_clause<M>(
        &mut self,
        query: &Query<M>,
        build_context: &mut BuildContext,
        result: &mut BuildResult,
    ) -> Result<()> {
        self.resolve_select(&FieldPath::default(), query, build_context, result)?;
        if result.select_expr.is_empty() {
            result.select_expr.push_literal("1");
        } else {
            result.select_expr.pop_literals(2); // Remove trailing ,
        }
        if result.select_expr.is_empty() {
            result.select_expr.push_literal("1");
        } else {
            result.select_expr.pop_literals(2); // Remove trailing ,
        }

        Ok(())
    }

    fn resolve_select<M>(
        &self,
        local_path: &FieldPath,
        query: &Query<M>,
        build_context: &mut BuildContext,
        result: &mut BuildResult,
    ) -> Result<()> {
        let mapper = self.joined_mapper_for_local_path(&local_path)?;

        let canonical_alias = self.canonical_alias(local_path)?;

        let path_selection = build_context
            .local_selected_paths
            .contains(local_path.as_str());

        for deserialization_type in &mapper.deserialize_order {
            match deserialization_type {
                DeserializeType::Field(field_name) => {
                    let local_field = if !local_path.is_empty() {
                        Cow::Owned(format!("{}_{}", local_path.as_str(), field_name))
                    } else {
                        Cow::Borrowed(field_name)
                    };

                    let mapped_field = mapper
                        .field(field_name)
                        .ok_or_else(|| SqlBuilderError::FieldMissing(field_name.to_string()))?;

                    let p = [
                        &self.aux_params,
                        &query.aux_params,
                        &mapped_field.options.aux_params,
                    ];
                    let aux_params = ParameterMap::new(&p);

                    let role_valid = mapped_field
                        .options
                        .load_role_expr
                        .as_ref()
                        .map_or(true, |e| RoleValidator::is_valid(&self.roles, e));

                    // If field is preselected
                    if mapped_field.options.preselect {
                        if !role_valid {
                            let role_string = mapped_field
                                .options
                                .load_role_expr
                                .as_ref()
                                .map_or_else(|| String::new(), |e| e.to_string());
                            return Err(SqlBuilderError::RoleRequired(
                                role_string,
                                FieldPath::from(&build_context.query_home_path)
                                    .append(&local_field)
                                    .to_string(),
                            )
                            .into());
                        }
                        let handler = mapped_field
                            .options
                            .field_handler
                            .as_ref()
                            .unwrap_or(&mapper.field_handler);
                        let select_expr =
                            handler.build_select(mapped_field.expression.clone(), &aux_params)?;

                        if let Some(expr) = select_expr {
                            let resolver = Resolver::new().with_self_alias(&canonical_alias);
                            let mut expr = resolver.resolve(&expr)?;

                            expr.push_literal(", ");

                            if local_path.is_empty()
                                || build_context
                                    .local_joined_paths
                                    .contains(local_path.as_str())
                            {
                                result.select_expr.extend(expr);
                                result.select_stream.push(Select::Preselect);
                            } else {
                                result.select_stream.push(Select::None);
                            }
                        } else {
                            // Column / expression is not selected
                            result.select_stream.push(Select::None);
                        }
                    }
                    // Field is selected through wildcard or explictit through field name
                    else if (path_selection && !mapped_field.options.skip_wildcard)
                        || build_context
                            .local_selected_fields
                            .contains(local_field.as_ref())
                    {
                        // If role is invalid raise error for explicit field and skip for wildcard selection
                        if !role_valid
                            && build_context
                                .local_selected_fields
                                .contains(local_field.as_ref())
                        {
                            let role_string = mapped_field
                                .options
                                .load_role_expr
                                .as_ref()
                                .map_or_else(|| String::new(), |e| e.to_string());
                            return Err(SqlBuilderError::RoleRequired(
                                role_string,
                                FieldPath::from(&build_context.query_home_path)
                                    .append(&local_field)
                                    .to_string(),
                            )
                            .into());
                        }

                        if role_valid {
                            let handler = mapped_field
                                .options
                                .field_handler
                                .as_ref()
                                .unwrap_or(&mapper.field_handler);
                            let select_expr = handler
                                .build_select(mapped_field.expression.clone(), &aux_params)?;
                            if let Some(expr) = select_expr {
                                // Fields with unresolved aux params that are selected through a wildcard are unselected
                                match expr.first_aux_param() {
                                    Some(p) if path_selection => {
                                        let query_field =
                                            FieldPath::from(build_context.query_home_path.as_str())
                                                .append(local_field.as_str());
                                        tracing::info!("Unselecting field `{}` in struct for table `{}` because aux param `{}` is missing", query_field.as_str(), &mapper.table_name, &p );
                                        result.select_stream.push(Select::None);
                                    }
                                    _ => {
                                        let resolver =
                                            Resolver::new().with_self_alias(&canonical_alias);
                                        let expr = resolver.resolve(&expr)?;
                                        result.select_expr.extend(expr);
                                        result.select_expr.push_literal(", ");
                                        result.select_stream.push(Select::Query);
                                        result.column_counter += 1;
                                    }
                                };
                            } else {
                                result.select_stream.push(Select::None);
                            }
                        } else {
                            result.select_stream.push(Select::None);
                        }
                    } else {
                        result.select_stream.push(Select::None);
                    }
                }
                DeserializeType::Join(join_name) => {
                    let mapped_join = mapper.join(join_name).ok_or_else(|| {
                        SqlBuilderError::JoinMissing(
                            join_name.to_string(),
                            mapper.table_name.to_string(),
                        )
                    })?;

                    let local_join_path = local_path.append(join_name);

                    let role_valid = mapped_join
                        .options
                        .load_role_expr
                        .as_ref()
                        .map_or(true, |e| RoleValidator::is_valid(&self.roles, e));

                    // If role is invalid raise error for explicit join
                    if build_context
                        .local_joined_paths
                        .contains(&local_join_path.to_string())
                    {
                        if !role_valid {
                            let role_string = mapped_join
                                .options
                                .load_role_expr
                                .as_ref()
                                .map_or_else(|| String::new(), |e| e.to_string());
                            return Err(SqlBuilderError::RoleRequired(
                                role_string,
                                FieldPath::from(&build_context.query_home_path)
                                    .append(&local_join_path)
                                    .to_string(),
                            )
                            .into());
                        }
                        result.select_stream.push(Select::Query); // Query selected join
                                                                  // join path is the same as to query path

                        // dbg!(&local_join_path);

                        // Seelect fields for this path
                        self.resolve_select(&local_join_path, query, build_context, result)?;
                    } else if mapped_join.options.preselect {
                        if !role_valid {
                            let role_string = if let Some(e) = &mapped_join.options.load_role_expr {
                                e.to_string()
                            } else {
                                String::from("")
                            };
                            return Err(SqlBuilderError::RoleRequired(
                                role_string,
                                FieldPath::from(&build_context.query_home_path)
                                    .append(&local_join_path)
                                    .to_string(),
                            )
                            .into());
                        }
                        //   dbg!(&local_join_path);
                        // Add preselected join to joined paths
                        build_context
                            .local_joined_paths
                            .insert(local_join_path.to_string());

                        result.select_stream.push(Select::Preselect); // Preselected join

                        self.resolve_select(&local_join_path, query, build_context, result)?;
                    } else {
                        result.select_stream.push(Select::None); // No Join
                    }
                }
                DeserializeType::Merge(merge_name) => {
                    let mapped_merge = mapper
                        .merge(merge_name)
                        .ok_or_else(|| SqlBuilderError::MergeMissing(merge_name.to_string()))?;

                    let query_field = FieldPath::from(build_context.query_home_path.as_str())
                        .append(local_path.as_str())
                        .append(merge_name.as_str());

                    if mapped_merge.options.preselect
                        || query.contains_path_starts_with(&query_field)
                    {
                        let role_valid = mapped_merge
                            .options
                            .load_role_expr
                            .as_ref()
                            .map_or(true, |e| RoleValidator::is_valid(&self.roles, e));

                        if !role_valid {
                            let role_string = mapped_merge
                                .options
                                .load_role_expr
                                .as_ref()
                                .map_or_else(|| String::new(), |e| e.to_string());
                            return Err(SqlBuilderError::RoleRequired(
                                role_string,
                                query_field.to_string(),
                            )
                            .into());
                        }
                        result.unmerged_home_paths.insert(query_field.to_string());
                    }
                }
            }
        }

        Ok(())
    }

    fn add_query_field(
        &self,
        query_field: &str,
        build_context: &mut BuildContext,
        unmerged_home_paths: &mut HashSet<String>,
        field_hidden: bool,
    ) -> Result<()> {
        let query_path = FieldPath::trim_basename(query_field);
        if !Self::home_contains(&build_context.query_home_path, &query_path) {
            return Ok(());
        }
        let local_path = match query_path.localize_path(&build_context.query_home_path) {
            Some(l) => l,
            None => return Ok(()),
        };

        if let Some(local_merge_path) = self.next_merge_path(&local_path)? {
            unmerged_home_paths.insert(
                FieldPath::from(&build_context.query_home_path)
                    .append(&local_merge_path)
                    .to_string(),
            );

            for path in FieldPath::from(&local_merge_path).step_up().skip(1) {
                build_context.local_joined_paths.insert(path.to_string());
            }
        } else {
            let query_field = FieldPath::from(&query_field);
            let local_field = match query_field.localize_path(&build_context.query_home_path) {
                Some(f) => f,
                None => return Ok(()),
            };
            if !field_hidden {
                build_context
                    .local_selected_fields
                    .insert(local_field.to_string());
            } else {
                // Insert path only for join (needed for hidden order, hidden filter)
                for path in FieldPath::from(&local_field).step_up().skip(1) {
                    build_context.local_joined_paths.insert(path.to_string());
                }
            }
        }

        Ok(())
    }

    fn resolve_custom_selection(
        &self,
        query_selection: &str,
        mut build_context: &mut BuildContext,
        mut unmerged_home_paths: &mut HashSet<String>,
    ) -> Result<()> {
        let (query_path, selection_name) = FieldPath::split_basename(query_selection);
        let mapper = self.mapper_for_query_path(&query_path)?;
        let selection = mapper
            .selections
            .get(selection_name)
            .ok_or_else(|| SqlBuilderError::SelectionMissing(selection_name.to_string()))?;
        for local_field_or_path in selection {
            // Path either ends with `*` or `_`
            if local_field_or_path.ends_with('*') || local_field_or_path.ends_with('_') {
                let query_path = FieldPath::from(query_path.as_str()).append(
                    local_field_or_path
                        .trim_end_matches('*')
                        .trim_end_matches('_'),
                );
                if let Some(local_path) = query_path.localize_path(&build_context.query_home_path) {
                    if let Some(merge_path) = self.next_merge_path(&local_path)? {
                        unmerged_home_paths.insert(
                            FieldPath::from(&build_context.query_home_path)
                                .append(&merge_path)
                                .to_string(),
                        );
                    } else {
                        build_context
                            .local_selected_paths
                            .insert(local_path.to_string());
                    }
                }
            } else {
                let query_field = FieldPath::from(query_path.as_str())
                    .append(local_field_or_path)
                    .to_string();
                let query_path = FieldPath::trim_basename(query_field.as_str());
                if let Some(local_path) = query_path.localize_path(&build_context.query_home_path) {
                    if let Some(merge_path) = self.next_merge_path(&local_path)? {
                        unmerged_home_paths.insert(
                            FieldPath::from(&build_context.query_home_path)
                                .append(&merge_path)
                                .to_string(),
                        );
                    } else {
                        self.add_query_field(
                            query_field.as_str(),
                            &mut build_context,
                            &mut unmerged_home_paths,
                            false,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn preparse_filter_joins<M>(
        &mut self,
        query: &Query<M>,
        build_context: &mut BuildContext,
        count_selection_only: bool,
    ) -> Result<()> {
        for token in &query.tokens {
            if let QueryToken::Field(field) = token {
                if field.filter.is_some() {
                    let query_path = FieldPath::from(&field.name);
                    if count_selection_only {
                        let root_mapper = self.root_mapper()?;
                        match root_mapper.selections.get("cnt") {
                            Some(selection) => {
                                let wildcard_path = format!("{}_*", field.name.as_str());
                                if !selection.contains(&field.name)
                                    && !selection.contains(&wildcard_path)
                                {
                                    continue;
                                }
                            }
                            None => continue,
                        }
                    }
                    if let Some(local_path_with_name) =
                        query_path.localize_path(&build_context.query_home_path)
                    {
                        let field_path = FieldPath::trim_basename(local_path_with_name.as_str());
                        if self.next_merge_path(&field_path)?.is_none() {
                            for path in field_path.step_up() {
                                build_context.local_joined_paths.insert(path.to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn selection_from_query<M>(
        &mut self,
        query: &Query<M>,
        mut build_context: &mut BuildContext,
    ) -> Result<HashSet<String>> {
        let mut unmerged_home_paths = HashSet::new();

        for token in &query.tokens {
            match token {
                QueryToken::Field(field) => {
                    self.add_query_field(
                        &field.name,
                        &mut build_context,
                        &mut unmerged_home_paths,
                        field.hidden,
                    )?;
                    if let Some(o) = &field.order {
                        let order = match o {
                            FieldOrder::Asc(o) => o,
                            FieldOrder::Desc(o) => o,
                        };
                        let query_path = FieldPath::from(&field.name);
                        if let Some(local_path_with_name) =
                            query_path.localize_path(&build_context.query_home_path)
                        {
                            build_context
                                .ordering
                                .entry(*order)
                                .or_insert_with(Vec::new)
                                .push((o.to_owned(), local_path_with_name.to_string()));
                        }
                    }
                }
                QueryToken::Wildcard(wildcard) => {
                    // TODO: Wildcard path may have ending _, check why and to remove
                    let query_path = FieldPath::from(&wildcard.path.trim_end_matches('_'));

                    if let Some(local_path) =
                        query_path.localize_path(&build_context.query_home_path)
                    {
                        // if !local_path.is_empty() {
                        // local_selected_paths.insert(local_path.to_string());
                        if let Some(local_merge_path) = self.next_merge_path(&local_path)? {
                            // insert full query path
                            unmerged_home_paths.insert(
                                FieldPath::from(&build_context.query_home_path)
                                    .append(&local_merge_path)
                                    .to_string(),
                            );
                            for path in FieldPath::from(&local_merge_path).step_up().skip(1) {
                                build_context.local_joined_paths.insert(path.to_string());
                            }
                        } else {
                            build_context
                                .local_selected_paths
                                .insert(local_path.to_string());
                        }
                        // }
                    }

                    //  relative_paths.insert(wildcard.path.to_string());
                }
                QueryToken::Selection(selection) => {
                    let (query_path, selection_name) = FieldPath::split_basename(&selection.name);

                    // Process only if selection path is a valid local path
                    if let Some(local_path) =
                        query_path.localize_path(&build_context.query_home_path)
                    {
                        if let Some(local_merge_path) = self.next_merge_path(&local_path)? {
                            // insert full query path
                            unmerged_home_paths.insert(
                                FieldPath::from(&build_context.query_home_path)
                                    .append(&local_merge_path)
                                    .to_string(),
                            );
                            for path in FieldPath::from(&local_merge_path).step_up().skip(1) {
                                build_context.local_joined_paths.insert(path.to_string());
                            }
                        } else {
                            let mapper = self.joined_mapper_for_local_path(&local_path)?;
                            match selection_name {
                                "all" => {
                                    // Add all fields explicitly. This will include fields that are skip_wildcard
                                    for deserialization_type in &mapper.deserialize_order {
                                        if let DeserializeType::Field(field_name) =
                                            deserialization_type
                                        {
                                            // Skip invliad load role restriction
                                            let f =
                                                mapper.fields.get(field_name).ok_or_else(|| {
                                                    SqlBuilderError::FieldMissing(
                                                        field_name.to_string(),
                                                    )
                                                })?;
                                            if let Some(role_expr) = &f.options.load_role_expr {
                                                if !RoleValidator::is_valid(&self.roles, role_expr)
                                                {
                                                    continue;
                                                }
                                            }

                                            let query_field = FieldPath::from(
                                                build_context.query_home_path.as_str(),
                                            )
                                            .append(local_path.as_str())
                                            .append(field_name)
                                            .to_string();
                                            self.add_query_field(
                                                query_field.as_str(),
                                                &mut build_context,
                                                &mut unmerged_home_paths,
                                                false,
                                            )?;
                                        }
                                    }
                                }
                                "mut" => {
                                    // Add all mutable fields on that path
                                    // (additionally keys and preselects will be added when building actual select expression)
                                    for deserialization_type in &mapper.deserialize_order {
                                        if let DeserializeType::Field(field_name) =
                                            deserialization_type
                                        {
                                            let f =
                                                mapper.fields.get(field_name).ok_or_else(|| {
                                                    SqlBuilderError::FieldMissing(
                                                        field_name.to_string(),
                                                    )
                                                })?;
                                            // Skip invalid load role restriction
                                            if let Some(role_expr) = &f.options.load_role_expr {
                                                if !RoleValidator::is_valid(&self.roles, role_expr)
                                                {
                                                    continue;
                                                }
                                            }
                                            // Skip invalid mut role restriction
                                            if let Some(role_expr) = &f.options.update_role_expr {
                                                if !RoleValidator::is_valid(&self.roles, role_expr)
                                                {
                                                    continue;
                                                }
                                            }
                                            if !f.options.skip_mut && !f.options.key {
                                                let query_field = FieldPath::from(
                                                    build_context.query_home_path.as_str(),
                                                )
                                                .append(local_path.as_str())
                                                .append(field_name)
                                                .to_string();
                                                self.add_query_field(
                                                    query_field.as_str(),
                                                    &mut build_context,
                                                    &mut unmerged_home_paths,
                                                    false,
                                                )?;
                                            }
                                        }
                                    }
                                }
                                "cnt" => {
                                    let selection = mapper.selections.get("cnt");
                                    if let Some(selection) = selection {
                                        // Select fields that are used for counting
                                        // Additionally to keys and preselects
                                        for deserialization_type in &mapper.deserialize_order {
                                            if let DeserializeType::Field(query_field_name) =
                                                deserialization_type
                                            {
                                                let wildcard_path =
                                                    format!("{}_*", query_field_name);
                                                if !selection.contains(&query_field_name)
                                                    && !selection.contains(&wildcard_path)
                                                {
                                                    continue;
                                                }

                                                self.add_query_field(
                                                    &query_field_name,
                                                    &mut build_context,
                                                    &mut unmerged_home_paths,
                                                    false,
                                                )?;
                                            }
                                        }
                                    } else {
                                        // If cnt selection is undefined, select keys and preselects
                                        for deserialization_type in &mapper.deserialize_order {
                                            if let DeserializeType::Field(field_name) =
                                                deserialization_type
                                            {
                                                let f = mapper.fields.get(field_name).ok_or_else(
                                                    || {
                                                        SqlBuilderError::FieldMissing(
                                                            field_name.to_string(),
                                                        )
                                                    },
                                                )?;
                                                if f.options.key || f.options.preselect {
                                                    let query_field = FieldPath::from(
                                                        build_context.query_home_path.as_str(),
                                                    )
                                                    .append(local_path.as_str())
                                                    .append(field_name)
                                                    .to_string();
                                                    self.add_query_field(
                                                        query_field.as_str(),
                                                        &mut build_context,
                                                        &mut unmerged_home_paths,
                                                        false,
                                                    )?;
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    self.resolve_custom_selection(
                                        &selection.name,
                                        &mut build_context,
                                        &mut unmerged_home_paths,
                                    )?;
                                }
                            }
                        }
                    } else {
                        // Evaluate selections that are not local to the selection path.
                        // `all` and custom selections may be defined above the actual query path
                        // Let's say selection is  `$users_all` and query_home_path is `users_memberships`
                        // A local selection had to start with `$users_memberships_all`, however we still have to
                        // evaluate `$users_all` because the selection goes down
                        if build_context
                            .query_home_path
                            .starts_with(query_path.as_str())
                        {
                            if selection_name == "all" {
                                let mapper =
                                    self.joined_mapper_for_local_path(&FieldPath::default())?;

                                build_context.local_selected_paths.insert("".to_string());
                                self.add_all_joins_as_selected_paths(
                                    &mapper.table_name,
                                    "".to_string(),
                                    &mut build_context,
                                    &mut unmerged_home_paths,
                                )?;
                            } else if selection_name != "cnt" && selection_name != "mut" {
                                self.resolve_custom_selection(
                                    &selection.name,
                                    &mut build_context,
                                    &mut unmerged_home_paths,
                                )?;
                            }
                        }
                    }
                }
                QueryToken::Predicate(predicate) => {
                    let query_path = FieldPath::trim_basename(&predicate.name);
                    if let Some(local_path) =
                        query_path.localize_path(&build_context.query_home_path)
                    {
                        // Skip local path if it contains a merged field
                        if self.joined_mapper_for_query_path(&local_path).is_ok() {
                            for partial_local_path in FieldPath::from(&local_path).step_up() {
                                // Skip predicate name
                                build_context
                                    .local_joined_paths
                                    .insert(partial_local_path.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(unmerged_home_paths)
    }

    fn root_mapper(&self) -> Result<&TableMapper> {
        self.table_mapper_registry
            .get(&self.home_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_string()))
    }
    fn next_merge_path(&self, local_path: &FieldPath) -> Result<Option<String>> {
        let mut current_mapper = self
            .table_mapper_registry
            .get(&self.home_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_string()))?;

        for (mapper_name, merge_path) in local_path.children().zip(local_path.step_down()) {
            if current_mapper.merged_mapper(mapper_name.as_str()).is_some() {
                return Ok(Some(merge_path.to_string()));
            } else if let Some(joined_mapper_name) =
                current_mapper.joined_mapper(mapper_name.as_str())
            {
                let m = self
                    .table_mapper_registry
                    .get(&joined_mapper_name)
                    .ok_or_else(|| ToqlError::MapperMissing(mapper_name.to_string()))?;
                current_mapper = m;
            } else {
                break;
            }
        }
        Ok(None)
    }

    fn home_contains(home_path: &str, query_path: &FieldPath) -> bool {
        let r = match (home_path.is_empty(), query_path.is_empty()) {
            (true, true) => true,
            (false, true) => false,
            (true, false) => true,
            (false, false) => query_path.as_str().starts_with(home_path),
        };

        r
    }
}
