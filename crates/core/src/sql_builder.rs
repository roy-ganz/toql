//!
//! The SQL Builder turns a [Query](../query/struct.Query.html) with the help of a [SQL Mapper](../sql_mapper/struct.SqlMapper.html)
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
pub mod sql_builder_error;
pub mod wildcard_scope;
pub mod select_stream;

pub(crate) mod build_context;
pub(crate) mod build_result;
pub(crate) mod path_tree;
//pub(crate) mod sql_with_placeholders;

use super::sql_builder::build_context::BuildContext;
use super::sql_builder::build_result::BuildResult;
use crate::error::{Result, ToqlError};
use crate::query::Query;
use crate::query::QueryToken;
use crate::sql_builder::sql_builder_error::SqlBuilderError;

use crate::sql_mapper::{DeserializeType, SqlMapper};

use std::collections::HashMap;
use std::collections::HashSet;

use crate::sql_mapper_registry::SqlMapperRegistry;

use crate::sql_arg::SqlArg;
use crate::{
    parameter::ParameterMap,
    query::field_path::FieldPath,
    sql_expr::{resolver::Resolver, SqlExpr},
};
use path_tree::PathTree;
use std::borrow::Cow;
use select_stream::Select;

enum MapperOrMerge<'a> {
    Mapper(&'a SqlMapper),
    Merge(String),
}

/// The Sql builder to build normal queries and count queries.
pub struct SqlBuilder<'a> {
    root_mapper: String,      // root type 
    home_mapper: String,    // home mapper, depends on query root
    sql_mapper_registry: &'a SqlMapperRegistry,
    roles: HashSet<String>,
    aux_params: HashMap<String, SqlArg>, // Aux params used for all queries with this builder instance, contains typically config or auth data

    extra_joins: HashSet<String>, // Use this joins
}

impl<'a> SqlBuilder<'a> {
    /// Create a new SQL Builder
    pub fn new(base_type: &'a str, sql_mapper_registry: &'a SqlMapperRegistry) -> Self {
        SqlBuilder {
            root_mapper: base_type.to_string(),
            home_mapper: base_type.to_string(),
            sql_mapper_registry,
            roles: HashSet::new(),
            aux_params: HashMap::new(),
            extra_joins: HashSet::new(),
        }
    }

    /* pub fn change_root_for_path(&mut self, root: &str, path: &str) -> Result<()> {
        let path = if path.is_empty() {
            None
        } else {
            Some(FieldPath::from(path))
        };
        self.root_mapper = String::from(root);
        let mapper = self.mapper_for_path(&path)?;
        self.root_mapper = String::from(&mapper.table_name);
        Ok(())
    } */

    pub fn with_roles(mut self, roles: HashSet<String>) -> Self {
        self.roles = roles;
        self
    }
    pub fn with_aux_params(mut self, aux_params: HashMap<String, SqlArg>) -> Self {
        // TODO use Parameter Map
        self.aux_params = aux_params;
        self
    }

    pub fn with_extra_join<T: Into<String>>(mut self, join: T) -> Self {
        self.extra_joins.insert(join.into());
        self
    }

    /*  pub fn join_sql(&self, field_path: &str) -> Result<Sql> {
        let mut context = BuildContext::new();
        let mut sql = Sql::new();
        let mut current_mapper = self.root_mapper()?;
        let field_path = FieldPath::from(field_path);
        let mut alias = std::iter::once(FieldPath::from("")).chain(field_path.descendents()); // build canonical aliases
        let canonical_table_alias = &self.root_mapper()?.canonical_table_alias;
        let p = [&self.aux_params];
        let aux_params = ParameterMap::new(&p);

        let mut alias1 = alias.next();
        let mut alias2 = alias.next();

        for d in field_path.children() {
            dbg!(&d);

            let self_alias = self.alias_translator.translate(
                alias1
                    .unwrap_or(FieldPath::from("")) // Should never be used
                    .prefix(&canonical_table_alias)
                    .as_str(),
            );

            let other_alias = self.alias_translator.translate(
                alias2
                    .as_ref()
                    .unwrap_or(&FieldPath::from("")) // Should never be used
                    .prefix(&canonical_table_alias)
                    .as_str(),
            );

            // Rotate aliases
            alias1 = alias2;
            alias2 = alias.next();

            if let Some(j) = current_mapper.join(d.as_str()) {
                current_mapper = self
                    .sql_mapper_registry
                    .get(&j.joined_mapper)
                    .ok_or(ToqlError::MapperMissing(j.joined_mapper.to_string()))?;

                let join_sql =
                    j.join_expression
                        .resolve(&self_alias, Some(&other_alias), &aux_params, &[])?;
                sql.append(&join_sql);
                sql.push_literal(" ON ");

                let join_on_sql =
                    j.on_expression
                        .resolve(&self_alias, Some(&other_alias), &aux_params, &[])?;
                sql.append(&join_on_sql);
                sql.push_literal(" ");
            } else if let Some(m) = current_mapper.merge(d.as_str()) {
                current_mapper = self
                    .sql_mapper_registry
                    .get(&m.merged_mapper)
                    .ok_or(ToqlError::MapperMissing(m.merged_mapper.to_string()))?;

                let merge_sql =
                    m.merge_predicate
                        .resolve(&self_alias, Some(&other_alias), &aux_params, &[])?;
                sql.append(&merge_sql);
                sql.push_literal(" ");
            } else {
                return Err(ToqlError::MapperMissing(d.as_str().to_string()));
            }
        }
        sql.pop_literals(1); // Remove trailing space
        Ok(sql)
    } */
    pub fn merge_expr(&self, field_path: &str) -> Result<(SqlExpr, SqlExpr)> {
        // let mut sql = SqlExpr::new();
        let mut current_mapper = self.sql_mapper_registry.get(&self.root_mapper)
        .ok_or(ToqlError::MapperMissing(self.root_mapper.to_string()))?; // set root ty
        let (basename, ancestor_path) = FieldPath::split_basename(field_path);

        let canonical_table_alias = &current_mapper.canonical_table_alias;
        /*  let p = [&self.aux_params];
        let aux_params = ParameterMap::new(&p); */

        if let Some(p) = ancestor_path.as_ref() {
            for d in p.children() {
                dbg!(&d);

                if let Some(j) = current_mapper.merge(d.as_str()) {
                    current_mapper = self
                        .sql_mapper_registry
                        .get(&j.merged_mapper)
                        .ok_or(ToqlError::MapperMissing(j.merged_mapper.to_string()))?;
                } else {
                    return Err(ToqlError::MapperMissing(d.as_str().to_string()));
                }
            }
        }

        // Build canonical alias
      /*   let self_alias = ancestor_path
            .unwrap_or(FieldPath::from(""))
            .prefix(&canonical_table_alias);

        let other_alias = FieldPath::from(field_path).prefix(&canonical_table_alias); */

        // Get merge join statement and on predicate
        let merge = current_mapper.merge(basename).ok_or(ToqlError::NotFound)?;

       /*  let resolver = Resolver::new()
            .with_self_alias(&self_alias.as_str())
            .with_other_alias(&other_alias.as_str()); */

      /*   let join_expr = resolver.resolve(&merge.merge_join)?;
        let on_expr = resolver.resolve(&merge.merge_predicate)?;

        Ok((join_expr, on_expr)) */
         Ok((merge.merge_join.to_owned(), merge.merge_predicate.to_owned()))

        /* let join_clause_sql =
            merge
                .merge_join
                .resolve(&self_alias, Some(&other_alias), &aux_params, &[])?;
        sql.append(&join_clause_sql);
        sql.push_literal(" ON ");

        let on_clause_sql =
            merge
                .merge_predicate
                .resolve(&self_alias, Some(&other_alias), &aux_params, &[])?;
        sql.append(&on_clause_sql);
        // return two sql
        // TODO translator in sql builder to have same translations
        // Oder anstatt Alias Format -> Translator übergeben
        // Rückgabe 2 expressions -> resolve in hauptfunktion
        Ok(sql)  */
    }

    pub fn build_delete<M>(&mut self, query: &Query<M>) -> Result<BuildResult> {
        let mut context = BuildContext::new();
        let root_mapper = self
            .sql_mapper_registry
            .mappers
            .get(&self.home_mapper)
            .ok_or(ToqlError::MapperMissing(self.home_mapper.to_owned()))?;
        /* let alias_table = self
        .alias_translator
        .translate(&root_mapper.canonical_table_alias); */
        // let aliased_table = format!("{} {}", root_mapper.table_name, alias_table);
        let mut result = BuildResult::new(SqlExpr::literal("DELETE"));

        result.set_from(
            root_mapper.table_name.to_owned(),
            root_mapper.canonical_table_alias.to_owned(),
        );

        self.build_where_clause(&query, &mut context, &mut result)?;
        self.build_join_clause(&mut context, &mut result)?;

        Ok(result)
    }
    /*  pub fn build_delete_sql<M>(
        &mut self,
        query: &Query<M>,
        modified: &str,
        extra: &str,
    ) -> Result<Sql> {
        let result = self.build_delete_result(query)?;

       // result.delete_sql().map_err( ToqlError::from)
        result.to_sql().map_err( ToqlError::from)
    } */

    pub fn build_select<M>(
        &mut self,
        query_home_path: &str,
        query: &Query<M>,
    ) -> Result<BuildResult> {
        let mut context = BuildContext::new();
        context.query_home_path = query_home_path.to_string();

        // let root_mapper = self.root_mapper()?;
        let query_home_path = if query_home_path.is_empty() {
            None
        } else {
            Some(FieldPath::from(query_home_path))
        };
        dbg!(&query_home_path);

        self.set_home_mapper_for_path(&query_home_path)?;

        let mapper = self
            .sql_mapper_registry
            .get(&self.home_mapper)
            .ok_or(ToqlError::MapperMissing(self.home_mapper.to_string()))?;
        /*  let alias_table = self
            .alias_translator
            .translate(&root_mapper.canonical_table_alias);

        let aliased_table = format!("{} {}", root_mapper.table_name, alias_table); */
        let mut result = BuildResult::new(SqlExpr::literal("SELECT"));
        result.set_from(
            mapper.table_name.to_owned(),
            mapper.canonical_table_alias.to_owned(),
        );

        self.build_where_clause(&query, &mut context, &mut result)?;
        self.build_select_clause(&query, &mut context, &mut result)?;
        self.build_join_clause(&mut context, &mut result)?;

        Ok(result)
    }

    /*  pub fn build_select<M>(
        &mut self,
        query_root_path: &str,
        query: &Query<M>,
    ) -> Result<BuildResult> {
       self.build_select_result(query_root_path, query)?;

        Ok(result)
    } */

    pub fn build_count<M>(
        &mut self,
        query_root_path: &str,
        query: &Query<M>,
    ) -> Result<BuildResult> {
        let mut context = BuildContext::new();
        context.query_home_path = query_root_path.to_string();
        let root_mapper = self.root_mapper()?; // self.mapper_for_path(&Self::root_field_path(root_path))?;

        let mut result = BuildResult::new(SqlExpr::literal("SELECT COUNT(*)"));

        result.set_from(
            root_mapper.table_name.to_owned(),
            root_mapper.canonical_table_alias.to_owned(),
        );

        self.build_where_clause(&query, &mut context, &mut result)?;
        todo!();
        //self.build_select_clause(&query, &mut context, &mut result)?;
        self.build_join_clause(&mut context, &mut result)?;

        Ok(result)
    }
    /*  pub fn build_count_sql<M>(
        &mut self,
        query_root_path: &str,
        query: &Query<M>,
        modified: &str,
        extra: &str,
        mut alias_translator: &mut AliasTranslator,
    ) -> Result<Sql> {
        let result = self.build_count_result(query_root_path, query, alias_translator)?;

       result.count_sql(alias_translator).map_err( ToqlError::from)
    } */

    fn mapper_for_path(&self, path: &Option<FieldPath>) -> Result<&SqlMapper> {
        let mut current_mapper = self
            .sql_mapper_registry
            .get(&self.home_mapper)
            .ok_or(ToqlError::MapperMissing(self.home_mapper.to_string()))?;

        if let Some(path) = path {
            for p in path.children() {
                if let Some(join) = current_mapper.joins.get(p.as_str()) {
                    current_mapper = self
                        .sql_mapper_registry
                        .get(&join.joined_mapper)
                        .ok_or(ToqlError::MapperMissing(join.joined_mapper.to_string()))?;
                } else {
                    //else if let Some(merge) = current_mapper.merges.get(p.as_str()){
                    /* current_mapper = self.sql_mapper_registry.get(&merge.merged_mapper).ok_or(ToqlError::MapperMissing(self.root_mapper.to_string()))?;
                    } else { */
                    return Err(SqlBuilderError::JoinMissing(p.as_str().to_string()).into());
                }
            }
        }

        Ok(current_mapper)
    }

    fn mapper_or_merge_for_path(
        &'a self,
        path: &'a Option<FieldPath>,
    ) -> Result<MapperOrMerge<'a>> {
        let mut current_mapper = self
            .sql_mapper_registry
            .get(&self.home_mapper)
            .ok_or(ToqlError::MapperMissing(self.home_mapper.to_string()))?;

        if let Some(path) = path {
            for (p, a) in path.children().zip(path.step()) {
                dbg!(&a);
                if current_mapper.merges.contains_key(p.as_str()) {
                    return Ok(MapperOrMerge::Merge(a.to_string()));
                }
                let join = current_mapper
                    .joins
                    .get(p.as_str())
                    .ok_or(ToqlError::MapperMissing(p.as_str().to_string()))?;
                current_mapper = self
                    .sql_mapper_registry
                    .get(&join.joined_mapper)
                    .ok_or(ToqlError::MapperMissing(self.home_mapper.to_string()))?;
            }
        }

        Ok(MapperOrMerge::Mapper(current_mapper))
    }
    fn set_home_mapper_for_path(&mut self, path: &Option<FieldPath>) -> Result<()> {
      
        if let Some(path) = path {
            let mut current_type: &str = &self.root_mapper;
            let mut current_mapper = self
                .sql_mapper_registry
                .get(current_type)
                .ok_or(ToqlError::MapperMissing(current_type.to_string()))?;

            for p in path.children() {
                dbg!(&p);
                if let Some(merge) = current_mapper.merges.get(p.as_str()) {
                    current_mapper = self
                        .sql_mapper_registry
                        .get(&merge.merged_mapper)
                        .ok_or(ToqlError::MapperMissing(merge.merged_mapper.to_string()))?;
                    current_type = &merge.merged_mapper;
                } else if let Some(join) = current_mapper.joins.get(p.as_str()) {
                    current_mapper = self
                        .sql_mapper_registry
                        .get(&join.joined_mapper)
                        .ok_or(ToqlError::MapperMissing(join.joined_mapper.to_string()))?;
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
    ) -> Result<()> {
        use heck::MixedCase;
        // Build join tree for all selected paths
        // This allows to nest joins properly
        // Eg [user] = [user_address, user_folder]
        // [user_folder] = [ user_folder_owner]
        // [user_folder_owner] =[]
        // [user address] =[]

        let mut join_tree = PathTree::new();
        let home_path = self.home_mapper.to_mixed_case(); 

        for selectect_path in &build_context.joined_paths {
           
            // Add home path for tree
            let canonical_path = FieldPath::from(&selectect_path).prepend(&home_path);
            join_tree.insert(&canonical_path);
        }
        dbg!(&join_tree);

        // Build join
        for r in join_tree.roots() {
            // Remove again home path
            /* let path = FieldPath::from(&r);
            let default_path= FieldPath::default();
            let path = path.relative_path(&home_path).unwrap_or(default_path); */

            let expr: SqlExpr =
                self.resolve_join(FieldPath::from(&r), &join_tree, &mut build_context)?;
            result.join_expr.extend(expr);
            result.join_expr.pop_literals(1); // Remove trailing whitespace
        }

        Ok(())
    }
    fn resolve_join(
        &self,
        canonical_path: FieldPath,
        join_tree: &PathTree,
        build_context: &mut BuildContext,
    ) -> Result<SqlExpr> {
        
        //  let mapper_name= canonical_path.ancestor().unwrap_or(canonical_path.basename());
        use heck::MixedCase;

        let mut join_expr = SqlExpr::new();
        let home_path = self.home_mapper.to_mixed_case(); 
        let relative_path =  canonical_path.relative_path(&home_path);
        let mapper = self.mapper_for_path(&relative_path)?;
        for nodes in join_tree.nodes(canonical_path.as_str()) {
            for n in nodes {
                //let mapper = self.mapper_from_path(canonical_path)?;
               

                let (basename, _) = FieldPath::split_basename(n);

                let join = mapper
                    .join(basename)
                    .ok_or(SqlBuilderError::JoinMissing(n.to_string()))?;

                let p = [&self.aux_params, &join.options.aux_params];
                let aux_params = ParameterMap::new(&p);

                /*  let self_alias = self
                    .alias_translator
                    .translate(canonical_path.as_str());
                let other_alias = self.alias_translator.translate(n.as_str()); */
                let resolver = Resolver::new()
                    .with_self_alias(canonical_path.as_str())
                    .with_other_alias(n.as_str());

                let join_e = resolver.resolve(&join.join_expression)?;
                join_expr.extend(join_e);

                let subjoin_expr =
                    self.resolve_join(FieldPath::from(n.as_str()), join_tree, build_context)?;
                if !subjoin_expr.is_empty() {
                    join_expr.push_literal(" (".to_string());
                    join_expr.extend(subjoin_expr);
                    join_expr.push_literal(")".to_string());
                }

                join_expr.push_literal(" ON (".to_string());

                let on_expr = resolver.resolve(&join.on_expression)?;

                let on_expr = match &join.options.join_handler {
                    Some(handler) => handler.build_on_predicate(on_expr, &aux_params)?,
                    None => on_expr,
                };
                join_expr.extend(on_expr);

                join_expr.push_literal(") ");
            }
        }

        Ok(join_expr)
    }

    /* fn mapper_from_path(&self, canonical_path: &FieldPath) -> Result<&SqlMapper> {
        //let path = canonical_path.trim_start_matches(self.root_path);
        let path = canonical_path;
        let mut mapper: Option<&SqlMapper> = None;
        for c in path.children() {
            mapper = Some(
                self.sql_mapper_registry
                    .get(c.as_str())
                    .ok_or(ToqlError::MapperMissing(c.as_str().to_string()))?,
            );
        }

        mapper.ok_or(ToqlError::MapperMissing("".to_ascii_lowercase()))
    } */

    fn build_where_clause<M>(
        &mut self,
        query: &Query<M>,
        build_context: &mut BuildContext,
        result: &mut BuildResult,
    ) -> Result<()> {
        let p = [&self.aux_params, &query.aux_params];
        let aux_params = ParameterMap::new(&p);

        println!("token: {:?}", &query.tokens);
        for token in &query.tokens {
            match token {
                QueryToken::Field(field) => {
                    let (basename, path) = FieldPath::split_basename(&field.name);

                    // skip if field path is not relative to root path
                    if !Self::root_contains(&build_context.query_home_path, &path) {
                        continue;
                    }

                    // Get relative path
                    //
                    let relative_path = path
                        .as_ref()
                        .and_then(|p| p.relative_path(&build_context.query_home_path));
                    dbg!(&path);
                    dbg!(&relative_path);
                    dbg!(&build_context.query_home_path);

                    let mapper_or_merge = self.mapper_or_merge_for_path(&relative_path)?;

                    match mapper_or_merge {
                        MapperOrMerge::Mapper(mapper) => {
                            let mapped_field = mapper
                                .fields
                                .get(basename)
                                .ok_or(SqlBuilderError::FieldMissing(basename.to_string()))?;

                            // Continue if field is not filtered
                            if field.filter.is_none() {
                                continue;
                            }

                            if let Some(role) = self.missing_role(
                                &mapped_field.options.roles, /*mapper.load_roles(field)*/
                            ) {
                                return Err(SqlBuilderError::RoleRequired(role.to_string()).into());
                            }
                            let canonical_alias = self.canonical_alias(&relative_path)?;
                            let resolver = Resolver::new().with_self_alias(&canonical_alias);

                            let field_expr = resolver.resolve(&mapped_field.expression)?;

                            let select_expr = mapped_field
                                .handler
                                .build_select(field_expr, &aux_params)?
                                .unwrap_or(SqlExpr::new());

                            // Does filter apply
                            if let Some(expr) = mapped_field.handler.build_filter(
                                select_expr,
                                field.filter.as_ref().unwrap(),
                                &aux_params,
                            )? {
                                result.where_expr.extend(expr);

                                if let (_, Some(path)) = FieldPath::split_basename(&field.name) {
                                    build_context.joined_paths.insert(path.as_str().to_string());
                                }
                            }
                        }
                        MapperOrMerge::Merge(merge_path) => {
                            result.unmerged_paths.insert(merge_path);
                        }
                    }
                }

                QueryToken::Predicate(predicate) => {
                    let (basename, path) = FieldPath::split_basename(&predicate.name);

                    // skip if field path is not relative to root path
                    if !Self::root_contains(&build_context.query_home_path, &path) {
                        continue;
                    }

                    let relative_path = path
                        .as_ref()
                        .and_then(|p| p.relative_path(&build_context.query_home_path));

                    let mapper_or_merge = self.mapper_or_merge_for_path(&relative_path)?;

                    match mapper_or_merge {
                        MapperOrMerge::Mapper(mapper) => {
                            let mapped_predicate = mapper
                                .predicates
                                .get(basename)
                                .ok_or(SqlBuilderError::PredicateMissing(basename.to_string()))?;

                            if let Some(role) = self.missing_role(&mapped_predicate.options.roles) {
                                return Err(SqlBuilderError::RoleRequired(role.to_string()).into());
                            }
                            // Build canonical alias

                            let canonical_alias = self.canonical_alias(&relative_path)?;
                            /*  let canonical_alias = match &relative_path {
                                Some(p) => Cow::Owned(format!(
                                    "{}_{}",
                                    &self.root_mapper()?.canonical_table_alias,
                                    p.as_str()
                                )),
                                None => Cow::Borrowed(&mapper.canonical_table_alias),
                            }; */
                            let resolver = Resolver::new().with_self_alias(&canonical_alias);

                            let expr = resolver.resolve(&mapped_predicate.expression)?;

                            /*   let alias = self.alias_translator.translate(&canonical_alias);
                            let sql = mapped_predicate.expression.resolve(
                                &alias,
                                None,
                                &aux_params,
                                &[],
                            )?; */

                            result.where_expr.extend(expr);

                            if let Some(p) = path {
                                build_context.joined_paths.insert(p.as_str().to_string());
                            }
                        }
                        MapperOrMerge::Merge(merge_path) => {
                            result.unmerged_paths.insert(merge_path);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn canonical_alias<'c>(&'c self, optional_path: &'c Option<FieldPath>) -> Result<Cow<String>> {
        let root_alias = &self.root_mapper()?.canonical_table_alias;

        Ok(match optional_path {
            Some(p) => Cow::Owned(p.prepend(&root_alias).to_string()),
            None => Cow::Borrowed(&root_alias),
        })
    }

    fn build_select_clause<M>(
        &mut self,
        query: &Query<M>,
        build_context: &mut BuildContext,
        result: &mut BuildResult,
    ) -> Result<()> {
        let (selected_fields, selected_paths, all_fields) =
            self.selection_from_query(query, build_context)?;
        build_context.selected_fields = selected_fields;
        build_context.selected_paths = selected_paths;
        build_context.all_fields_selected = all_fields;
        dbg!(&build_context.selected_paths);

        self.resolve_select(&None, query, build_context, result, 0)?;

        /*  result.select_expr = build_context
        .select_sql
        .into_sql(&build_context.selected_placeholders); */
        if result.select_expr.is_empty() {
            result.select_expr.push_literal("1");
        } else {
            result.select_expr.pop_literals(2); // Remove trailing ,
        }

        Ok(())
    }

    fn resolve_select<M>(
        &self,
        join_path: &Option<FieldPath>,
        query: &Query<M>,
        build_context: &mut BuildContext,
        result: &mut BuildResult,
        ph_index : u16
    ) -> Result<()> {
        
        use heck::MixedCase;

        let mapper = self.mapper_for_path(&join_path)?;

        let p = [&self.aux_params, &query.aux_params];
        let aux_params = ParameterMap::new(&p);
        let canonical_alias = self.canonical_alias(join_path)?;
        /*  let canonical_alias = match join_path {
                   Some(p) => Cow::Owned(format!("{}_{}", &self.root_mapper, p.as_str())),
                   None => Cow::Borrowed(&self.root_mapper),
               };
        */
       

        let mut any_selected = false;

        dbg!(&join_path);
        // relativ_path

        for deserialization_type in &mapper.deserialize_order {
            match deserialization_type {
                DeserializeType::Field(field) => {
                    let absolute_path = FieldPath::from(field);
                    let relative_path = absolute_path.relative_path(&build_context.query_home_path);

                    let explicit_selection =
                        if let Some(a) = relative_path.as_ref().and_then(|p| p.ancestor()) {
                            build_context.selected_paths.contains(a.as_str())
                        } else {
                            false
                        } || {
                            let selection_name = if let Some(j) = join_path {
                                Cow::Owned(format!("{}_{}", j.as_str(), field))
                            } else {
                                Cow::Borrowed(field)
                            };
                            build_context
                                .selected_fields
                                .contains(selection_name.as_ref())
                        };

                    let field_info = mapper
                        .field(field)
                        .ok_or(SqlBuilderError::FieldMissing(field.to_string()))?;

                    // Field is explicitly selected in query
                    if explicit_selection {
                        let resolver = Resolver::new().with_self_alias(&canonical_alias);

                        let select_expr = resolver.resolve(&field_info.expression)?;

                        let select_expr =
                            field_info.handler.build_select(select_expr, &aux_params)?;

                        if let Some(expr) = select_expr {
                            result.select_expr.extend(expr);
                            result.select_expr.push_literal(", ");
                            result.selection_stream.push(Select::Explicit);
                            any_selected = true;
                        } else {
                            result.selection_stream.push(Select::None);
                        }
                        result.column_counter += 1;
                    }
                    // Field may be preselected (implicit selection)
                    // Add those fields with placeholder number into expression.
                    // If any other field is explicitly selected, select also placeholder number to include 
                    // expression in final Sql.
                    else {
                        if field_info.options.preselect {
                            let resolver = Resolver::new().with_self_alias(&canonical_alias);

                            // let alias = self.alias_translator.translate(&canonical_alias);
                            let select_expr = resolver.resolve(&field_info.expression)?;
                            let select_expr =
                                field_info.handler.build_select(select_expr, &aux_params)?;
                            
                            if let Some(mut expr) = select_expr {
                                expr.push_literal(", ");
                                result.select_expr.push_placeholder(ph_index, expr, result.selection_stream.len() );
                            }
                            result.column_counter += 1;
                        }
                        result.selection_stream.push(Select::None);
                    }
                }
                DeserializeType::Join(join) => {
                    //   let new_join_path= format!("{}_{}", &canonical_alias, &join);

                     let join_info = mapper
                        .join(join)
                        .ok_or(SqlBuilderError::JoinMissing(join.to_string()))?;
                    
 
                    if build_context.selected_paths.contains(join) {
                        result.selection_stream.push(Select::Explicit);  // Query selected join
                        // self.resolve_select(&Some(FieldPath::from(&new_join_path)), query, build_context, result)?;
                         let new_join_path = FieldPath::from(&canonical_alias).append(join);
                        self.resolve_select(
                            &Some(new_join_path),
                            query,
                            build_context,
                            result,
                            ph_index + 1
                        )?;
                    } else if join_info.options.preselect {
                         //   let new_join_path= format!("{}_{}", &canonical_alias, &join);
                         let path = join.to_mixed_case();
                         let next_join_path = join_path.as_ref().unwrap_or(&FieldPath::default()).append(path.as_str());
                        dbg!(&next_join_path);
                        build_context.joined_paths.insert(next_join_path.to_string());
                        result.selection_stream.push(Select::Implicit); // Preselected join
                        result.selected_placeholders.insert(ph_index + 1); // Select placeholder for preselected join

                         self.resolve_select(
                            &Some(next_join_path),
                            query,
                            build_context,
                            result,
                            ph_index + 1 
                        )?;

                    }
                    
                    else {
                        result.selection_stream.push(Select::None); // No Join 
                    }
                }
                DeserializeType::Merge(merge) => {
                    if build_context.selected_paths.contains(merge) {
                        result.unmerged_paths.insert(merge.to_owned());
                    }
                }
            }
        }

        if any_selected {
            // TODO Maybe evaluate placeholders here
            result.selected_placeholders.insert(ph_index);
        }
        Ok(())
    }
    /* fn resolve_select_none(
           &self,
           join_path: &Option<FieldPath>,
           result: &mut BuildResult,
       ) -> Result<()> {
           use crate::sql_mapper::DeserializeType;
           let mapper = self.mapper_for_path(&join_path)?;
           let canonical_alias = join_path
               .as_ref()
               .map(|j| j.as_str())
               .unwrap_or(&self.root_mapper);

           for deserialization_type in &mapper.deserialize_order {
               match deserialization_type {
                   DeserializeType::Field(_) => {
                       result.selection_stream.push(false);
                   }
                   DeserializeType::Join(join) => {
                       let new_join_path = format!("{}_{}", &canonical_alias, &join);
                       self.resolve_select_none(&Some(FieldPath::from(&join)), result)?;
                   }
                   DeserializeType::Merge(_) => {}
               }
           }
           Ok(())
       }
    */
    fn selection_from_query<M>(
        &mut self,
        query: &Query<M>,
        build_context: &BuildContext,
    ) -> Result<(HashSet<String>, HashSet<String>, bool)> {
        let mut relative_fields = HashSet::new();
        let mut relative_paths = HashSet::new();
        let mut all_fields = false;

        for token in &query.tokens {
            match token {
                QueryToken::Field(field) => {
                    let (_, absolute_path) = FieldPath::split_basename(&field.name);
                    // TODO validate roles and raise error

                    if !Self::root_contains(&build_context.query_home_path, &absolute_path) {
                        continue;
                    }

                    if let Some(absolute_path) = absolute_path {
                        let relative_path =
                            absolute_path.relative_path(&build_context.query_home_path);
                        if let Some(relative_path) = relative_path {
                            for p in relative_path.step() {
                                relative_paths.insert(p.to_string());
                            }
                        }
                    }
                    let field = FieldPath::from(&field.name);
                    let relative_field =
                        field.relative_path(&build_context.query_home_path).unwrap();
                    relative_fields.insert(relative_field.to_string());
                }
                QueryToken::Wildcard(wildcard) => {
                    let (_, absolute_path) = FieldPath::split_basename(&wildcard.path);
                    if let Some(absolute_path) = absolute_path {
                        for p in absolute_path.descendents() {
                            // Todo validate roles and skip
                            relative_paths.insert(p.to_string());
                        }
                    }

                    //  relative_paths.insert(wildcard.path.to_string());
                }
                QueryToken::Selection(selection) => match selection.name.as_str() {
                    "all" => all_fields = true,
                    "mut" => {
                        let m = self.root_mapper()?;
                        for deserialization_type in &m.deserialize_order {
                            if let DeserializeType::Field(field_name) = deserialization_type {
                                let f = m
                                    .fields
                                    .get(field_name)
                                    .ok_or(SqlBuilderError::FieldMissing(field_name.to_string()))?;
                                if f.options.mut_select {
                                    let field = FieldPath::from(&field_name);
                                    let relative_field = field
                                        .relative_path(&build_context.query_home_path)
                                        .unwrap();
                                    relative_fields.insert(relative_field.to_string());
                                }
                            }
                        }
                    }
                    "cnt" => {
                        let m = self.root_mapper()?;
                        for deserialization_type in &m.deserialize_order {
                            if let DeserializeType::Field(field_name) = deserialization_type {
                                let f = m
                                    .fields
                                    .get(field_name)
                                    .ok_or(SqlBuilderError::FieldMissing(field_name.to_string()))?;
                                if f.options.count_select {
                                    let field = FieldPath::from(&field_name);
                                    let relative_field = field
                                        .relative_path(&build_context.query_home_path)
                                        .unwrap();
                                    relative_fields.insert(relative_field.to_string());
                                }
                            }
                        }
                    }
                    name @ _ => {
                        let m = self.root_mapper()?;
                        let selection = m
                            .selections
                            .get(name)
                            .ok_or(SqlBuilderError::SelectionMissing(name.to_string()))?;
                        for s in selection {
                            if s.ends_with("*") {
                                let path =
                                    FieldPath::from(s.trim_end_matches("*").trim_end_matches("_"));
                                let relative_path =
                                    path.relative_path(&build_context.query_home_path).unwrap();
                                relative_paths.insert(relative_path.to_string());
                            } else {
                                let field = FieldPath::from(s);
                                let relative_field =
                                    field.relative_path(&build_context.query_home_path).unwrap();
                                relative_paths.insert(relative_field.to_string());
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        Ok((relative_fields, relative_paths, all_fields))
    }

    fn root_mapper(&self) -> Result<&SqlMapper> {
        self.sql_mapper_registry
            .get(&self.home_mapper)
            .ok_or(ToqlError::MapperMissing(self.home_mapper.to_string()))
    }
    fn missing_role<'c>(&'c self, roles: &'c HashSet<String>) -> Option<&'c str> {
        let s = roles.difference(&self.roles).next();
        s.map(|r| r.as_str())
    }
    fn root_contains(root_path: &str, absolute_path: &Option<FieldPath>) -> bool {
        println!(
            "Test absolute path  {:?} has root {:?}",
            &absolute_path, &root_path
        );
        let root: Option<&str> = if root_path.is_empty() {
            None
        } else {
            Some(root_path)
        };
        let r = match (root, absolute_path) {
            (None, None) => true,
            (None, Some(_)) => true,
            (Some(_), None) => false,
            (Some(r), Some(a)) => a.as_str().starts_with(r),
        };
        println!("Result {:?}", r);

        r
    }
    /*   fn root_field_path(root_path: &str) -> Option<FieldPath> {
           if root_path.is_empty() {
               None
           } else {
               Some(FieldPath::from(root_path))
           }
       }
    */
    /* pub fn merge_path(&self, path: &FieldPath) -> Result<Option<&str>> {


        let mut mapper = self.root_mapper()?;
        let mp = path.children();

        for p in path.descendents() {
            match mapper.join(p.as_str()) {
                Some(m) => {mapper = self.mm.joined_mapper },
                None= {
                    if mapper.merges.contains(p) {
                        Ok(Some(mp))
                    }
                }
            }

        }
        Ok(None)

    } */
}
