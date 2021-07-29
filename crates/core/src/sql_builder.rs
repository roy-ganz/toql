//!
//! The SQL Builder turns a [Query](../query/struct.Query.html) with the help of a [SQL Mapper](../table_mapper/struct.TableMapper.html)
//! into a [SQL Builder Result](../sql_builder_result/BuildResult.html)
//! The result hold the different parts of an SQL query and can be turned into an SQL query that can be sent to the database.
//!
//! ## Example
//!
//! ``` ignore
//!
//! let  query = Query::wildcard().and(Field::from("foo").eq(5));
//! let mapper::new("Bar b").map_field("foo", "b.foo");
//! let builder_result = QueryBuilder::new().build_query(&mapper, &query);
//! assert_eq!("SELECT b.foo FROM Bar b WHERE b.foo = ?", builder_result.to_sql());
//! assert_eq!(["5"], builder_result.params());
//! ```
//!
//! The SQL Builder can also add joins if needed. Joins must be registered on the SQL Mapper for this.
//!
//! ### Count queries
//! Besides normal queries the SQL Builder can als build count queries.
//!
//! Let's assume you have a grid view with books and the user enters a search term to filter your grid.
//! The normal query will get 50 books, but you will only display 10 books. Toql calls those 50 _the filtered count_.
//! To get the unfilted count, Toql must issue another query with different filter settings. Typically to get
//! the number of all books only that user has access to. Toql calls this _the total count_.
//!
//! ### Paths
//! The SQL Builder can also ignore paths to skip paths in the query that are not mapped in the mapper.
//! This is needed for structs that contain collections, as these collections must be querried with a different mapper.
//!
//! Let's assume a struct *user* had a collection of *phones*.
//! The Toql query may look like:  `username, phones_number`.
//! The SQL Builder needs 2 passes to resolve that query:
//!  - The first pass will query all users with the user mapper and will ignore the path *phones_*.
//!  - The second pass will only build the query for the path *phones_* with the help of the phone mapper.
//!
pub mod select_stream;
pub mod sql_builder_error;
pub mod wildcard_scope;
pub mod build_result;

pub(crate) mod build_context;
pub(crate) mod path_tree;

use super::sql_builder::build_context::BuildContext;
use super::sql_builder::build_result::BuildResult;
use crate::error::ToqlError;
use crate::query::Query;
use crate::query::QueryToken;
use crate::result::Result;
use crate::sql_builder::sql_builder_error::SqlBuilderError;

use crate::table_mapper::{join_type::JoinType, DeserializeType, TableMapper};

use std::collections::HashMap;
use std::collections::HashSet;

use crate::table_mapper_registry::TableMapperRegistry;

use crate::sql_arg::SqlArg;
use crate::{
    parameter_map::ParameterMap,
    query::{concatenation::Concatenation, field_order::FieldOrder, field_path::FieldPath},
    role_validator::RoleValidator,
    sql_expr::{resolver::Resolver, SqlExpr},
};
use path_tree::PathTree;
use select_stream::Select;
use std::borrow::Cow;

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

    extra_joins: HashSet<String>, // Use this joins
}

impl<'a> SqlBuilder<'a> {
    /// Create a new SQL Builder
    pub fn new(base_type: &'a str, table_mapper_registry: &'a TableMapperRegistry) -> Self {
        SqlBuilder {
            root_mapper: base_type.to_string(),
            home_mapper: base_type.to_string(),
            table_mapper_registry,
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            extra_joins: HashSet::new(),
        }
    }

    pub fn with_roles(mut self, roles: HashSet<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_aux_params(mut self, aux_params: HashMap<String, SqlArg>) -> Self {
        self.aux_params = aux_params;
        self
    }

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
                        columns_expr.push_literal(" AND ");
                    }
                    columns_expr.extend(resolver.resolve(&field.expression)?);
                }
                DeserializeType::Join(name) => {
                    let join = mapper
                        .join(&name)
                        .ok_or_else(|| SqlBuilderError::JoinMissing(name.to_string()))?;
                    if !join.options.key {
                        return Ok(());
                    }
                    let other_alias = FieldPath::from(alias).append(&mapper.canonical_table_alias);
                    let resolver = Resolver::new()
                        .with_self_alias(alias)
                        .with_other_alias(other_alias.as_str());

                    join_expr.push_literal("JOIN ");
                    join_expr.extend(resolver.resolve(&join.table_expression)?);
                    join_expr.push_literal(" ON (");
                    join_expr.extend(resolver.resolve(&join.on_expression)?);
                    join_expr.push_literal(") ");

                    let joined_query_path = FieldPath::from(query_path).append(&name);

                    self.resolve_columns_expr(
                        joined_query_path.as_str(),
                        other_alias.as_str(),
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
        let (basename, query_path) = FieldPath::split_basename(query_field_path);
        let mapper = self.mapper_for_query_path(&query_path)?;
        /* let mut current_mapper = self
            .table_mapper_registry
            .get(&self.root_mapper)
            .ok_or(ToqlError::MapperMissing(self.root_mapper.to_string()))?; // set root ty

        self.mapper_or_merge_for_path(local_path)
        if !query_path.is_empty() {
            for d in query_path.children() {
                dbg!(&d);

                if let Some(j) = current_mapper.merge(d.as_str()) {
                    current_mapper = self
                        .table_mapper_registry
                        .get(&j.merged_mapper)
                        .ok_or(ToqlError::MapperMissing(j.merged_mapper.to_string()))?;
                } else {
                    return Err(ToqlError::MapperMissing(d.as_str().to_string()));
                }
            }
        } */

        // Get merge join statement and on predicate
        let merge = mapper.merge(basename).ok_or(ToqlError::NotFound)?;

        Ok((
            merge.merge_join.to_owned(),
            merge.merge_predicate.to_owned(),
        ))
    }

    pub fn build_delete<M>(&mut self, query: &Query<M>) -> Result<BuildResult> {
        let mut context = BuildContext::new();
        let root_mapper = self
            .table_mapper_registry
            .mappers
            .get(&self.home_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.home_mapper.to_owned()))?;

        if let Some(role_expr) = &root_mapper.delete_role_expr {
            if !RoleValidator::is_valid(&self.roles, role_expr) {
                return Err(SqlBuilderError::RoleRequired(role_expr.to_string()).into());
            }
        }

        let mut result = BuildResult::new(SqlExpr::literal("DELETE"));

        result.set_from(
            root_mapper.table_name.to_owned(),
            root_mapper.canonical_table_alias.to_owned(),
        );
        self.preparse_filter_joins(&query, &mut context)?;
        self.build_where_clause(&query, &mut context, false, &mut result)?;
        self.build_join_clause(&mut context, &mut result, true)?;

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

        let (merge_field, base_path) = FieldPath::split_basename(merge_path.as_str());

        let base_mapper = self.joined_mapper_for_local_path(&base_path)?;

        let root_path = FieldPath::from(&root_mapper.canonical_table_alias);
        let path = if base_path.is_empty() {
            root_path
        } else {
            base_path
        };
        let (self_field, _) = FieldPath::split_basename(path.as_str());

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

        let merge_field = format!("{}_{}", self_field, merge_field);
        let resolver = Resolver::new()
            .with_self_alias(self_field)
            .with_other_alias(&merge_field);

        resolver.resolve(&delete_expr).map_err(ToqlError::from)
    }

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

        let mut result = BuildResult::new(SqlExpr::literal("SELECT"));
        result.set_from(
            mapper.table_name.to_owned(),
            mapper.canonical_table_alias.to_owned(),
        );

        self.preparse_query(&query, &mut context, &mut result)?;
        self.build_where_clause(&query, &mut context, false, &mut result)?;
        self.build_select_clause(&query, &mut context, &mut result)?;
        self.build_join_clause(&mut context, &mut result, false)?;
        self.build_order_clause(&mut context, &mut result)?;

        Ok(result)
    }

    pub fn build_count<M>(
        &mut self,
        query_root_path: &str,
        query: &Query<M>,
        count_selection_only: bool,
    ) -> Result<BuildResult> {
        let mut build_context = BuildContext::new();
        build_context.query_home_path = query_root_path.to_string();
        let root_mapper = self.root_mapper()?; // self.joined_mapper_for_path(&Self::root_field_path(root_path))?;

        let mut result = BuildResult::new(SqlExpr::literal("SELECT COUNT(*)"));

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

        let _unmerged_home_paths = self.selection_from_query(query, &mut build_context)?;

        if count_selection_only {
            // Strip path and fields that are not in count selection
            let mut default_count_selection = Vec::new();
            default_count_selection.push("*".to_string());
            let root_mapper = self.root_mapper()?;

            for s in root_mapper
                .selections
                .get("cnt")
                .unwrap_or(&default_count_selection)
            {
                if s.ends_with("_*") {
                    //selected_paths.remove(s.trim_end_matches("_*"));
                    build_context
                        .local_selected_paths
                        .remove(s.trim_end_matches("_*"));
                }
            }
            // all_fields = false;
        }

        // build_context.local_selected_paths = selected_paths;
        //  build_context.all_fields_selected = all_fields;

        self.build_join_clause(&mut build_context, &mut result, true)?;

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
                //   println!("Getting join for name {}", p.as_str());
                if let Some(join) = current_mapper.joins.get(p.as_str()) {
                    current_mapper = self
                        .table_mapper_registry
                        .get(&join.joined_mapper)
                        .ok_or_else(|| ToqlError::MapperMissing(join.joined_mapper.to_string()))?;
                } else {
                    return Err(SqlBuilderError::JoinMissing(p.as_str().to_string()).into());
                }
            }
        }

        Ok(current_mapper)
    }

    fn mapper_for_query_path(&self, query_path: &FieldPath) -> Result<&TableMapper> {
        let mut current_mapper = self
            .table_mapper_registry
            .get(&self.root_mapper)
            .ok_or_else(|| ToqlError::MapperMissing(self.root_mapper.to_string()))?;

        if !query_path.is_empty() {
            for p in query_path.children() {
                //   println!("Getting join for name {}", p.as_str());
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
                    return Err(ToqlError::MapperMissing(p.as_str().to_string()));
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
                    .ok_or_else(|| ToqlError::MapperMissing(p.as_str().to_string()))?;
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
        mut build_context: &mut BuildContext,
        result: &mut BuildResult,
        enforce_inner_joins : bool
    ) -> Result<()> {
        // Build join tree for all selected paths
        // This allows to nest joins properly
        // Eg [user] = [user_address, user_folder]
        // [user_folder] = [ user_folder_owner]
        // [user_folder_owner] =[]
        // [user address] =[]

        let mut join_tree = PathTree::new();
        /* let home_path = self.home_mapper.to_mixed_case(); */

        for local_path in &build_context.local_joined_paths {
            // let query_path = FieldPath::from(&local_path).prepend(&build_context.query_home_path);
            // join_tree.insert(&query_path);
            join_tree.insert(&FieldPath::from(&local_path));
        }

        // Build join

        let expr: SqlExpr = self.resolve_join(
            FieldPath::default(),
            &join_tree,
            &join_tree.roots(),
            &mut build_context,
            enforce_inner_joins
        )?;
        result.join_expr.extend(expr);
        result.join_expr.pop_literals(1); // Remove trailing whitespace

        Ok(())
    }
    fn resolve_join(
        &self,
        _local_path: FieldPath,
        join_tree: &PathTree,
        nodes: &HashSet<String>,
        build_context: &mut BuildContext,
        enforce_inner_joins: bool
    ) -> Result<SqlExpr> {
        let mut join_expr = SqlExpr::new();

        for local_path_with_join in nodes {
            let (join_name, local_path) = FieldPath::split_basename(local_path_with_join);

            let local_mapper = self.joined_mapper_for_local_path(&local_path)?;
            let join = local_mapper
                .join(join_name)
                .ok_or_else(|| SqlBuilderError::JoinMissing(join_name.to_string()))?;

            let p = [&self.aux_params, &join.options.aux_params];
            let aux_params = ParameterMap::new(&p);

            let canonical_self_alias = self.canonical_alias(&local_path)?.to_string();
            let canonical_other_alias = self
                .canonical_alias(&FieldPath::from(local_path_with_join))?
                .to_string();
            let resolver = Resolver::new()
                .with_self_alias(&canonical_self_alias)
                .with_other_alias(&canonical_other_alias);

            

            join_expr.push_literal( if enforce_inner_joins {
                "JOIN ("
            } else {
                match &join.join_type {
                JoinType::Inner => "JOIN (",
                JoinType::Left => "LEFT JOIN (",
            }});
            let join_e = resolver.resolve(&join.table_expression)?;
            join_expr.extend(join_e);
            join_expr.push_literal(" ");

            if let Some(subnodes) = join_tree.nodes(local_path_with_join) {
                if !subnodes.is_empty() {
                    let subjoin_expr = self.resolve_join(
                        local_path.append(local_path_with_join),
                        join_tree,
                        &subnodes,
                        build_context,
                        enforce_inner_joins
                    )?;
                    if !subjoin_expr.is_empty() {
                        join_expr.extend(subjoin_expr);
                    }
                }
            }

            join_expr.push_literal(") ON (".to_string());

            let on_expr = resolver.resolve(&join.on_expression)?;

            let on_expr = match &join.options.join_handler {
                Some(handler) => handler.build_on_predicate(on_expr, &aux_params)?,
                None => on_expr,
            };

            // Skip left joins with unresolved aux params
            let on_expr = match on_expr.first_aux_param() {
                Some(p) if join.join_type == JoinType::Left => {
                    let query_path_with_join = FieldPath::from(&build_context.query_home_path)
                        .append(local_path_with_join);
                    log::info!("Setting condition of left join `{}` to `false`, because aux param `{}` is missing", query_path_with_join.as_str(), &p );
                    SqlExpr::literal("false")
                }
                _ => on_expr,
            };

            /* let on_expr = match &join.options.join_handler {
                Some(handler) => {
                     // Skip Join, if join is optional (LEFT Join) and aux param is missing
                       match handler.build_on_predicate(on_expr, &aux_params) {
                           Err(SqlBuilderError::QueryParamMissing(_)) if join.join_type == JoinType::Left => SqlExpr::literal("false"),
                           Ok(e) => e,
                           Err(e) => return Err(e.into())
                       }
                        },
                None => on_expr,
            };
            */
            //   println!("{:?}", &on_expr);
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

        // println!("token: {:?}", &query.tokens);
        for token in &query.tokens {
            match token {
                QueryToken::Field(field) => {
                    // Continue if field is not filtered
                    if field.filter.is_none() {
                        continue;
                    }
                    let (field_name, query_path) = FieldPath::split_basename(&field.name);

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

                            let select_expr = mapped_field
                                .handler
                                .build_select(mapped_field.expression.clone(), &aux_params)?
                                .unwrap_or_default();

                            // Does filter apply
                            if let Some(expr) = mapped_field.handler.build_filter(
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
                    let (basename, query_path) = FieldPath::split_basename(&predicate.name);

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

                            if let Some(role) = &mapped_predicate.options.load_role_expr {
                                if !RoleValidator::is_valid(&self.roles, role) {
                                    return Err(
                                        SqlBuilderError::RoleRequired(role.to_string()).into()
                                    );
                                }
                            }
                            // Build canonical alias

                            let canonical_alias = self.canonical_alias(&local_path)?;

                            let resolver = Resolver::new()
                                .with_self_alias(&canonical_alias)
                                .with_arguments(&predicate.args);

                            let expr = resolver.resolve(&mapped_predicate.expression)?;
                            if let Some(expr) = mapped_predicate.handler.build_predicate(
                                expr,
                                &predicate.args,
                                &aux_params,
                            )? {
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
                            }
                        }
                        MapperOrMerge::Merge(_merge_path) => {}
                    }
                }
                QueryToken::LeftBracket(concatenation) => {
                    if !result.where_expr.is_empty() {
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
                        result.where_expr.pop(); // Remove ' AND ' or 'OR ' token
                    } else {
                        result.where_expr.push_literal(")");
                    }
                }
                _ => {}
            }
        }

        /*  if !result.where_expr.is_empty() {
            result.where_expr.pop_literals(if last_concatenation == Concatenation::And {5} else {4}); // Remove trailing ' AND ' resp ' OR '
        } */
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
                //  println!("Adding `{}` to merge paths", query_merge_path.as_str());

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
                    let (field_name, local_path) =
                        FieldPath::split_basename(local_path_with_basename);
                    let mapper = self.joined_mapper_for_local_path(&local_path)?;
                    let field_info = mapper
                        .field(field_name)
                        .ok_or_else(|| SqlBuilderError::FieldMissing(field_name.to_string()))?;

                    let role_valid =
                        if let Some(load_role_expr) = &field_info.options.load_role_expr {
                            !RoleValidator::is_valid(&self.roles, load_role_expr)
                        } else {
                            true
                        };
                    if !role_valid {
                        let role_string = if let Some(e) = &field_info.options.load_role_expr {
                            e.to_string()
                        } else {
                            String::from("")
                        };
                        return Err(SqlBuilderError::RoleRequired(role_string).into());
                    }
                    let p = [&self.aux_params, &field_info.options.aux_params];
                    let aux_params = ParameterMap::new(&p);

                    let select_expr = field_info
                        .handler
                        .build_select(field_info.expression.clone(), &aux_params)?;
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

        for deserialization_type in &mapper.deserialize_order {
            match deserialization_type {
                DeserializeType::Field(field_name) => {
                    let path_selection = build_context
                        .local_selected_paths
                        .contains(local_path.as_str());

                    let local_field = if !local_path.is_empty() {
                        Cow::Owned(format!("{}_{}", local_path.as_str(), field_name))
                    } else {
                        Cow::Borrowed(field_name)
                    };
                    let field_selection = build_context
                        .local_selected_fields
                        .contains(local_field.as_ref());

                    let query_selection =
                        path_selection || field_selection /* || build_context.all_fields_selected*/;

                    let field_info = mapper
                        .field(field_name)
                        .ok_or_else(|| SqlBuilderError::FieldMissing(field_name.to_string()))?;

                    let p = [
                        &self.aux_params,
                        &query.aux_params,
                        &field_info.options.aux_params,
                    ];
                    let aux_params = ParameterMap::new(&p);

                    let role_valid =
                        if let Some(load_role_expr) = &field_info.options.load_role_expr {
                            !RoleValidator::is_valid(&self.roles, load_role_expr)
                        } else {
                            true
                        };

                    // Field is explicitly selected in query
                    if query_selection {
                        // If role is invalid raise error for explicit field and skip for path
                        if !role_valid && build_context.local_selected_fields.contains(field_name) {
                            let role_string = if let Some(e) = &field_info.options.load_role_expr {
                                e.to_string()
                            } else {
                                String::from("")
                            };
                            return Err(SqlBuilderError::RoleRequired(role_string).into());
                        }

                        let select_expr = field_info
                            .handler
                            .build_select(field_info.expression.clone(), &aux_params)?;

                        if role_valid {
                            // Do not select field, if field is selected through path and aux param is missing

                            if let Some(expr) = select_expr {
                                // Fields with unresolved aux params that are selected through a path are unselected
                                match expr.first_aux_param() {
                                    Some(p) if path_selection => {
                                        let query_field =
                                            FieldPath::from(build_context.query_home_path.as_str())
                                                .append(local_field.as_str());
                                        log::info!("Unselecting field `{}`  because aux param `{}` is missing", query_field.as_str(), &p );
                                        result.selection_stream.push(Select::None);
                                    }
                                    _ => {
                                        let resolver =
                                            Resolver::new().with_self_alias(&canonical_alias);
                                        let expr = resolver.resolve(&expr)?;
                                        result.select_expr.extend(expr);
                                        result.select_expr.push_literal(", ");
                                        result.selection_stream.push(Select::Query);
                                        result.column_counter += 1;
                                    }
                                };
                            } else {
                                result.selection_stream.push(Select::None);
                            }
                        } else {
                            result.selection_stream.push(Select::None);
                        }
                    }
                    // Field may be preselected (implicit selection)
                    else if field_info.options.preselect {
                        if !role_valid {
                            let role_string = if let Some(e) = &field_info.options.load_role_expr {
                                e.to_string()
                            } else {
                                String::from("")
                            };
                            return Err(SqlBuilderError::RoleRequired(role_string).into());
                        }

                        // let alias = self.alias_translator.translate(&canonical_alias);

                        let select_expr = field_info
                            .handler
                            .build_select(field_info.expression.clone(), &aux_params)?;

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
                                result.selection_stream.push(Select::Preselect);
                            } else {
                                result.selection_stream.push(Select::None);
                            }
                        } else {
                            // Column / expression is not selected
                            result.selection_stream.push(Select::None);
                        }
                    } else {
                        result.selection_stream.push(Select::None);
                    }
                }
                DeserializeType::Join(join_name) => {
                    let join_info = mapper
                        .join(join_name)
                        .ok_or_else(|| SqlBuilderError::JoinMissing(join_name.to_string()))?;

                    if let Some(load_role_expr) = &join_info.options.load_role_expr {
                        return Err(
                            SqlBuilderError::RoleRequired(load_role_expr.to_string()).into()
                        );
                    };

                    let local_join_path = local_path.append(join_name);
                    if build_context
                        .local_joined_paths
                        .contains(&local_join_path.to_string())
                    {
                        result.selection_stream.push(Select::Query); // Query selected join
                                                                     // join path is the same as to query path

                        // dbg!(&local_join_path);

                        // Seelect fields for this path
                        self.resolve_select(&local_join_path, query, build_context, result)?;
                    } else if join_info.options.preselect {
                        //   dbg!(&local_join_path);
                        // Add preselected join to joined paths
                        build_context
                            .local_joined_paths
                            .insert(local_join_path.to_string());

                        result.selection_stream.push(Select::Preselect); // Preselected join

                        self.resolve_select(&local_join_path, query, build_context, result)?;
                    } else {
                        result.selection_stream.push(Select::None); // No Join
                    }
                }
                DeserializeType::Merge(_) => {}
            }
        }

        Ok(())
    }

    fn add_query_field(
        &self,
        query_field: &str,
        build_context: &mut BuildContext,
        unmerged_home_paths: &mut HashSet<String>,
        /*local_selected_fields: &mut HashSet<String>, */
    ) -> Result<()> {
        let (_, query_path) = FieldPath::split_basename(query_field);
        if !Self::home_contains(&build_context.query_home_path, &query_path) {
            return Ok(());
        }
        let local_path = match query_path.localize_path(&build_context.query_home_path) {
            Some(l) => l,
            None => return Ok(()),
        };
        //  if !local_path.is_empty() {
        //local_selected_paths.insert(local_path.to_string());
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
            build_context
                .local_selected_fields
                .insert(local_field.to_string());
        }

        Ok(())
    }

    fn resolve_custom_selection(
        &self,
        query_selection: &str,
        mut build_context: &mut BuildContext,
        mut unmerged_home_paths: &mut HashSet<String>,
    ) -> Result<()> {
        let (selection_name, query_path) = FieldPath::split_basename(query_selection);
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
                let (_, query_path) = FieldPath::split_basename(query_field.as_str());
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
    ) -> Result<()> {
        for token in &query.tokens {
            match token {
                QueryToken::Field(field) => {
                    if field.filter.is_some() {
                        let query_path = FieldPath::from(&field.name);
                        if let Some(local_path_with_name) =
                            query_path.localize_path(&build_context.query_home_path)
                        {
                            let (_, field_path) =
                                FieldPath::split_basename(local_path_with_name.as_str());
                            if self.next_merge_path(&field_path)?.is_none() {
                                for path in field_path.step_up() {
                                    build_context.local_joined_paths.insert(path.to_string());
                                }
                            }
                        }
                    }
                }
                _ => {}
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
                        &mut unmerged_home_paths, /*&mut local_selected_fields, */
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
                    let query_path = FieldPath::from(&wildcard.path);

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
                    let (selection_name, query_path) = FieldPath::split_basename(&selection.name);

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
                                    build_context
                                        .local_selected_paths
                                        .insert(local_path.to_string());
                                    self.add_all_joins_as_selected_paths(
                                        &mapper.table_name,
                                        local_path.to_string(),
                                        &mut build_context,
                                        &mut unmerged_home_paths,
                                    )?;
                                }
                                "mut" => {
                                    // Add all mutable fields on that path
                                    // (additionally keys and preselects will be added when building actual select expression)
                                    for deserialization_type in &mapper.deserialize_order {
                                        match deserialization_type {
                                            DeserializeType::Field(field_name) => {
                                                let f = mapper.fields.get(field_name).ok_or_else(
                                                    || {
                                                        SqlBuilderError::FieldMissing(
                                                            field_name.to_string(),
                                                        )
                                                    },
                                                )?;
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
                                                    )?;
                                                }
                                            }
                                            DeserializeType::Join(join_name) => {
                                                let f = mapper.joins.get(join_name).ok_or_else(
                                                    || {
                                                        SqlBuilderError::JoinMissing(
                                                            join_name.to_string(),
                                                        )
                                                    },
                                                )?;
                                                if !(f.options.skip_mut || f.options.key) {
                                                    // Add keys from that join
                                                    let local_path = local_path.append(join_name);
                                                    self.add_nested_keys(
                                                        &f.joined_mapper,
                                                        &local_path,
                                                        build_context,
                                                    )?;
                                                }
                                            }
                                            DeserializeType::Merge(_) => {}
                                        }
                                    }
                                }
                                "cnt" => {
                                    // Select fields that are used for counting
                                    // Additionally to keys and preselects
                                    for deserialization_type in &mapper.deserialize_order {
                                        if let DeserializeType::Field(query_field_name) =
                                            deserialization_type
                                        {
                                            let f = mapper
                                                .fields
                                                .get(query_field_name)
                                                .ok_or_else(|| {
                                                    SqlBuilderError::FieldMissing(
                                                        query_field_name.to_string(),
                                                    )
                                                })?;
                                            if f.options.count_select {
                                                self.add_query_field(
                                                    &query_field_name,
                                                    &mut build_context,
                                                    &mut unmerged_home_paths,
                                                )?;
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
                        /*  println!(
                            "Path `{}` is not local to `{}`",
                            query_path.as_str(),
                            &build_context.query_home_path
                        ); */
                        if build_context
                            .query_home_path
                            .starts_with(query_path.as_str())
                        {
                            if selection_name == "all" {
                                let mapper =
                                    self.joined_mapper_for_local_path(&FieldPath::default())?;
                                // println!("Resolving all");
                                build_context.local_selected_paths.insert("".to_string());
                                self.add_all_joins_as_selected_paths(
                                    &mapper.table_name,
                                    "".to_string(),
                                    &mut build_context,
                                    &mut unmerged_home_paths,
                                )?;
                            } else if selection_name != "cnt" && selection_name != "mut" {
                                //println!("Resolving `{}`", selection_name);

                                self.resolve_custom_selection(
                                    &selection.name,
                                    &mut build_context,
                                    &mut unmerged_home_paths,
                                )?;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(unmerged_home_paths)
    }

    fn add_nested_keys(
        &self,
        joined_mapper_name: &str,
        local_path: &FieldPath,
        build_context: &mut BuildContext,
    ) -> Result<()> {
        let joined_mapper = self
            .table_mapper_registry
            .get(&joined_mapper_name)
            .ok_or_else(|| ToqlError::MapperMissing(joined_mapper_name.to_string()))?;
        for deserialization_type in &joined_mapper.deserialize_order {
            match deserialization_type {
                DeserializeType::Field(field_name) => {
                    let field = joined_mapper.field(field_name).ok_or_else(|| {
                        SqlBuilderError::FieldMissing(joined_mapper_name.to_string())
                    })?;
                    if field.options.key {
                        let local_field = local_path.append(field_name);
                        build_context
                            .local_selected_fields
                            .insert(local_field.to_string());
                    }
                }
                DeserializeType::Join(query_join_name) => {
                    let join = joined_mapper.join(query_join_name).ok_or_else(|| {
                        SqlBuilderError::JoinMissing(joined_mapper_name.to_string())
                    })?;
                    if join.options.key {
                        let local_path = local_path.append(query_join_name);
                        self.add_nested_keys(&join.joined_mapper, &local_path, build_context)?;
                    }
                }
                DeserializeType::Merge(_) => {}
            }
        }
        Ok(())
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
        /*   println!(
            "Test if query path  {:?} has home {:?}",
            &query_path, &home_path
        ); */

        let r = match (home_path.is_empty(), query_path.is_empty()) {
            (true, true) => true,
            (false, true) => false,
            (true, false) => true,
            (false, false) => query_path.as_str().starts_with(home_path),
        };
        //  println!("Result {:?}", r);

        r
    }
}
