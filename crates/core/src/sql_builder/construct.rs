use crate::sql_mapper::{Join, JoinType};
use std::collections::{HashMap, HashSet};
use crate::sql::{Sql, SqlArg};
use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::sql_builder::build_result::BuildResult;
use crate::sql_mapper::SqlTarget;
use crate::query::field_order::FieldOrder;
use crate::sql_builder::sql_target_data::SqlTargetData;

pub fn aux_param_values(
        aux_param_names: &Vec<String>,
        aux_params: &HashMap<String, SqlArg>,
    ) -> Result<Vec<SqlArg>, SqlBuilderError> {
        let mut params: Vec<SqlArg> = Vec::with_capacity(aux_param_names.len());
        for p in aux_param_names {
            let qp = aux_params
                .get(p)
                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
            params.push(qp.to_owned());
        }
        Ok(params)
    }

    pub fn resolve_query_params(
        expression: &str,
        aux_params: &HashMap<String, SqlArg>,
    ) -> Result<Sql, SqlBuilderError> {
        let (sql, params) = extract_query_params(expression);

        let mut resolved: Vec<SqlArg> = Vec::new();
        for p in params {
            let v = aux_params
                .get(&p)
                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
            resolved.push(v.to_owned());
        }

        Ok((sql, resolved))
    }

    pub fn extract_query_params(expression: &str) -> (String, Vec<String>) {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new(r"<([\w_]+)>").unwrap();
        }

        let mut query_params = Vec::new();
        let sql = REGEX.replace(expression, |e: &regex::Captures| {
            let name = &e[1];
            query_params.push(name.to_string());
            "?"
        });
        (sql.to_string(), query_params)
    }

    pub(crate) fn build_ordering(
        result: &mut BuildResult,
        query_aux_params: &HashMap<String, SqlArg>,
        sql_target_data: &HashMap<String, SqlTargetData>,
        sql_targets: &HashMap<String, SqlTarget>,
        ordinals: &HashSet<u8>,
        ordering: &HashMap<u8, Vec<(FieldOrder, String)>>,
    ) -> Result<(), SqlBuilderError> {
        // Build ordering clause
        for n in ordinals {
            if let Some(fields) = ordering.get(n) {
                for (ord, toql_field) in fields {
                    let o = match ord {
                        FieldOrder::Asc(_) => " ASC",
                        FieldOrder::Desc(_) => " DESC",
                    };
                    if let Some(_sql_target_data) = sql_target_data.get(toql_field.as_str()) {
                        if let Some(sql_target) = sql_targets.get(toql_field) {
                            let mut combined_aux_params: HashMap<String, SqlArg> = HashMap::new();
                            let aux_params = combine_aux_params(
                                &mut combined_aux_params,
                                query_aux_params,
                                &sql_target.options.aux_params,
                            );
                            let aux_param_values =  aux_param_values(&sql_target.sql_aux_param_names, aux_params)?;
                            if let Some(s) = sql_target.handler.build_select(
                                (sql_target.expression.to_owned(), aux_param_values),
                                aux_params,
                            )? {
                                result.order_clause.push_str(&s.0);
                                result.order_params.extend_from_slice(&s.1);
                            }
                        }
                    }
                    result.order_clause.push_str(o);
                    result.order_clause.push_str(", ");
                }
            }
        }
        result.order_clause = result.order_clause.trim_end_matches(", ").to_string();
        Ok(())
    }

    pub(crate) fn build_count_select_clause(
        result: &mut BuildResult,
        query_aux_params: &HashMap<String, SqlArg>,
        sql_targets: &HashMap<String, SqlTarget>,
        field_order: &Vec<String>,
    ) -> Result<(), SqlBuilderError> {
        let mut any_selected = false;
        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                // For selected fields there exists target data
                if sql_target.options.count_select {
                    let mut combined_aux_params: HashMap<String, SqlArg> = HashMap::new();
                    let aux_params = combine_aux_params(
                        &mut combined_aux_params,
                        query_aux_params,
                        &sql_target.options.aux_params,
                    );

                    let aux_param_values =  aux_param_values(&sql_target.sql_aux_param_names, aux_params)?;
                    if let Some(sql_field) = sql_target.handler.build_select(
                        (sql_target.expression.to_owned(), aux_param_values),
                        aux_params,
                    )? {
                        result.select_clause.push_str(&sql_field.0);
                        result.select_params.extend_from_slice(&sql_field.1);
                        result.select_clause.push_str(", ");
                        any_selected = true;
                    }
                }
            }
        }
        result.any_selected = any_selected;
        if any_selected {
            // Remove last ,
            result.select_clause = result.select_clause.trim_end_matches(", ").to_string();
        } else {
            result.select_clause = "1".to_string();
        }
        Ok(())
    }

    pub(crate) fn build_select_clause(
        result: &mut BuildResult,
        query_aux_params: &HashMap<String, SqlArg>,
        sql_targets: &HashMap<String, SqlTarget>,
        sql_target_data: &HashMap<String, SqlTargetData>,
        field_order: &Vec<String>,
        //  used_paths: &HashSet<String>,
        selected_paths: &HashSet<String>,
        // joins: &HashMap<String, Join>,
    ) -> Result<(), SqlBuilderError> {
        // Build select clause
        let mut any_selected = false;

        for toql_field in field_order {
            if let Some(sql_target) = sql_targets.get(toql_field) {
                // Skip fields that must not appear in select statement
                

                let path: &str = toql_field
                    .trim_end_matches(|c| c != '_')
                    .trim_end_matches('_');
                // For selected fields there exists target data
                // For always selected fields, check if path is used by query
                let selected = (/*join_selected
                ||*/sql_target.options.preselect
                        //&& (path.is_empty() || used_paths.contains(&path)))
                        && (path.is_empty() || selected_paths.contains(path)))
                    || sql_target_data
                        .get(toql_field.as_str())
                        .map_or(false, |d| d.selected);

                if selected {
                    let mut combined_aux_params: HashMap<String, SqlArg> = HashMap::new();
                    let aux_params = combine_aux_params(
                        &mut combined_aux_params,
                        query_aux_params,
                        &sql_target.options.aux_params,
                    );

                    let params =  aux_param_values(&sql_target.sql_aux_param_names, aux_params)?;
                    if let Some(sql_field) = sql_target
                        .handler
                        .build_select((sql_target.expression.to_owned(), params), aux_params)?
                    {
                        result.select_clause.push_str(&sql_field.0);
                        result.select_params.extend_from_slice(&sql_field.1);

                        /*  // Replace query params with actual values
                        for p in &sql_target.sql_query_params {
                            let qp = query_params
                                .get(p)
                                .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;
                            result.select_params.push(qp.to_string());
                        } */

                        any_selected = true;
                    } else {
                        result.select_clause.push_str("null");
                    }
                } else {
                    result.select_clause.push_str("null");
                }
                result.select_clause.push_str(", ");
            }
        }
        result.any_selected = any_selected;
        // Remove last ,
        result.select_clause = result.select_clause.trim_end_matches(", ").to_string();
        Ok(())
    }
    pub(crate) fn build_join_clause(
        join_root: &Vec<String>,
        join_tree: &HashMap<String, Vec<String>>,
        selected_paths: &mut HashSet<String>,
        sql_joins: &HashMap<String, Join>,
        aux_params: &HashMap<String, SqlArg>,
        result: &mut BuildResult,
    ) -> Result<(), SqlBuilderError> {
        fn build_join_start(join: &Join) -> String {
            let mut result = String::from(match join.join_type {
                JoinType::Inner => "JOIN (",
                JoinType::Left => "LEFT JOIN (",
            });
            result.push_str(&join.aliased_table);
            result
        }
        /* fn build_join_end(join: &Join) -> String {
            let mut result = String::from(") ON (");
            result.push_str(&join.on_predicate);
            result.push_str(") ");
            result
        } */
        pub(crate) fn build_joins(
            joins: &Vec<String>,
            selected_paths: &mut HashSet<String>,
            sql_joins: &HashMap<String, Join>,
            aux_params: &HashMap<String, SqlArg>,
            result: &mut BuildResult,
            join_tree: &HashMap<String, Vec<String>>,
        ) -> Result<(), SqlBuilderError>{
            for join in joins {
                // Construct join if
                // - join is left join and selected
                // - join is inner join (must always be selected)
                let join_data = sql_joins.get(join.as_str());
                if let Some(join_data) = join_data {
                    // If join is used in query

                    // Add path for preselected join
                    if join_data.options.preselect {
                        selected_paths.insert(join.to_owned());
                    }
                    // Construction rules for joins:
                    // - Preselected and Inner Joins always
                    // - Left Joins only on demand
                    let construct = join_data.options.preselect
                        || match join_data.join_type {
                            JoinType::Inner => true,
                            JoinType::Left => selected_paths.contains(join.as_str()),
                        };
                    if construct {
                        if let Some(t) = sql_joins.get(join) {
                            result.join_clause.push_str(build_join_start(&t).as_str());
                            result.join_clause.push(' ');
                            // Ressolve nested joins
                            resolve_nested(&join, selected_paths, sql_joins, aux_params,result, join_tree)?;
                            result.join_clause.pop(); // remove last whitespace

                           result.join_clause.push_str(") ON (");
                            
            
                          
                            
                             // Combine aux params from query and local join params
                             let mut combined_aux_params: HashMap<String, SqlArg> =
                                        HashMap::new();
                                    let temp_aux_params = combine_aux_params(
                                        &mut combined_aux_params,
                                        &aux_params,
                                        &join_data.options.aux_params,
                                    );
                                                       
                            let params = aux_param_values(&join_data.sql_aux_param_names, &temp_aux_params)?;
                            match &t.options.join_handler{
                                Some(h) => {
                                    let (on, pa) = h.build_on_predicate((t.on_predicate.to_owned(),params), aux_params)?;
                                    result.join_clause.push_str(&on);
                                    result.join_clause.push_str(") ");
                                    result.join_params.extend_from_slice(&pa);
                                }
                                None => {
                                    result.join_clause.push_str(&t.on_predicate);
                                    result.join_clause.push_str(") ");
                                    result.join_params.extend_from_slice(&params);
                                }
                            }; 
                                
                            
                            
                        }
                    }
                }
            }
            Ok(())
        }
        fn resolve_nested(
            path: &str,
            selected_paths: &mut HashSet<String>,
            sql_joins: &HashMap<String, Join>,
            aux_params: &HashMap<String, SqlArg>,
            result: &mut BuildResult,
            join_tree: &HashMap<String, Vec<String>>,
        ) -> Result<(), SqlBuilderError>{
            if join_tree.contains_key(path) {
                let joins = join_tree.get(path).unwrap();
                build_joins(&joins, selected_paths, sql_joins, aux_params,result, join_tree)?;
            }
            Ok(())
        }

        //println!("Selected joins {:?}", selected_paths);
        // Process top level joins
        build_joins(join_root, selected_paths, sql_joins, aux_params, result, join_tree)?;

        // Process all fields with subpaths from the query
        /*for (k, v) in sql_join_data  {
            // If not yet joined, check if subpath should be optionally joined
            if !v.joined {
                // For every subpath, check if there is JOIN data available
                if let Some(t) = sql_joins.get(*k) {
                    // If there is JOIN data available, use it to construct join
                    // Join data can be missing for directly typed join

                    result.join_clause.push_str(&t.join_clause);
                    result.join_clause.push(' ');
                }
                v.joined = true; // Mark join as processed
            }
        } */

        // Remove last whitespace
        if result
            .join_clause
            .chars()
            .rev()
            .next()
            .unwrap_or('A')
            .is_whitespace()
        {
            result.join_clause = result.join_clause.trim_end().to_string();
        }
        Ok(())
    }

 pub(crate)  fn path_ignored(ignored_paths: &Vec<String>, field_name: &str) -> bool {
        for path in ignored_paths {
            if field_name.starts_with(path) {
                return true;
            }
        }
        false
    }

 pub(crate)   fn combine_aux_params<'b>(
        combined_aux_params: &'b mut HashMap<String, SqlArg>,
        query_aux_params: &'b HashMap<String, SqlArg>,
        sql_target_aux_params: &'b HashMap<String, SqlArg>,
    ) -> &'b HashMap<String, SqlArg> {
        if sql_target_aux_params.is_empty() {
            query_aux_params
        } else {
            for (k, v) in query_aux_params {
                combined_aux_params.insert(k.clone(), v.clone());
            }
            for (k, v) in sql_target_aux_params {
                combined_aux_params.insert(k.clone(), v.clone());
            }
            combined_aux_params
        }
    }

    
pub(crate) fn insert_paths(field_with_path:&str, paths: &mut HashSet<String>) {
            let mut path = field_with_path
                                    .trim_end_matches(|c| c != '_')
                                    .trim_end_matches('_');
                while !path.is_empty() {
                    let exists = !paths.insert(path.to_owned());
                    if exists {
                        break;
                    }
                    path =
                        path.trim_end_matches(|c| c != '_').trim_end_matches('_');
                }

    }