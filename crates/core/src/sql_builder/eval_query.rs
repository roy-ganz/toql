
use crate::sql_builder::construct::path_ignored;
use crate::sql_builder::construct::insert_paths;
use crate::sql_builder::construct::aux_param_values;
use crate::sql_builder::construct::combine_aux_params;
use std::collections::{HashSet, HashMap};
use super::sql_target_data::SqlTargetData;
use super::build_result::BuildResult;
use crate::query::{Query, concatenation::Concatenation, field_order::FieldOrder};

use crate::query::QueryToken;
use super::BuildMode;
use crate::sql_mapper::SqlMapper;
use super::sql_builder_error::SqlBuilderError;
use super::wildcard_scope::WildcardScope;
use crate::sql::SqlArg;
use crate::query::field_filter::FieldFilter;
use crate::query::assert_roles;


pub(crate) fn eval_query<M>( 
    build_aux_params: &HashMap<String, SqlArg>,
    on_params: &mut HashMap<std::string::String,SqlArg>,
    roles: &HashSet<String>, 
    wildcard_scope: &WildcardScope,
    sql_mapper: &SqlMapper, 
    ignored_paths: &Vec<String>, 
    subpath: &String, 
    mode: &BuildMode, 
    query: &Query<M>,
    sql_target_data: &mut HashMap<String, SqlTargetData>, 
    mut selected_paths: &mut HashSet<String>, 
    ordinals: &mut HashSet<u8>,
    ordering: &mut HashMap<u8, Vec<(FieldOrder,String)>>,
    result : &mut BuildResult )->Result<(), SqlBuilderError> {

 

  let mut need_where_concatenation = false;
        let mut need_having_concatenation = false;
        let mut pending_where_parens_concatenation: Option<Concatenation> = None;
        let mut pending_having_parens_concatenation: Option<Concatenation> = None;
        let mut pending_where_parens: u8 = 0;
        let mut pending_having_parens: u8 = 0;

        for t in &query.tokens {
            {
                match t {
                    QueryToken::LeftBracket(ref concatenation) => {
                        pending_where_parens += 1;
                        pending_having_parens += 1;
                        pending_having_parens_concatenation = Some(concatenation.clone());
                        pending_where_parens_concatenation = Some(concatenation.clone());
                    }
                    QueryToken::RightBracket => {
                        if pending_where_parens > 0 {
                            pending_where_parens -= 1;
                        } else {
                            result.where_clause.push_str(")");
                            need_where_concatenation = true;
                        }
                        if pending_having_parens > 0 {
                            pending_having_parens -= 1;
                        } else {
                            result.having_clause.push_str(")");
                            need_having_concatenation = true;
                        }
                    }

                    QueryToken::Wildcard(wildcard) => {
                        // Skip wildcard for count queries
                      /*   if self.count_query {
                            continue;
                        } */
                        // Wildcard is only evaluated for nomral queries
                        if mode != &BuildMode::SelectQuery {
                            continue;
                        }
                        // Skip field from other path

                        if !(wildcard.path.starts_with(subpath)
                            || subpath.starts_with(&wildcard.path))
                        {
                            continue;
                        }

                        let wildcard_path = wildcard
                            .path
                            .trim_start_matches(subpath)
                            .trim_end_matches('_');

                        // Skip ignored path
                        if ignored_paths
                            .iter()
                            .any(|p| wildcard_path.starts_with(p))
                        {
                            continue;
                        }
                        // Ensure user has load roles for path
                        let mut path = wildcard_path;
                        while !path.is_empty() {
                            if let Some(join) = sql_mapper.joins.get(path) {
                                assert_roles(&roles, &join.options.roles)
                                    .map_err(|role| SqlBuilderError::RoleRequired(role))?;
                            } else {
                                return Err(SqlBuilderError::FieldMissing(path.to_owned()));
                            }
                            path = path.trim_end_matches(|c| c != '_').trim_end_matches('_');
                        }

                        // Cache vars to speed up validation
                        let mut last_validated_path = ("", true); 
                        let mut last_validated_scope_wildcard = ("", "", false);

                        for (field_name, sql_target) in &sql_mapper.fields {
                            
                            if !sql_target.options.query_select {
                                continue;
                            }


                            let field_path = field_name
                                .trim_end_matches(|c| c != '_')
                                .trim_end_matches('_');

                            // Check if field is in wildcard scope
                            let wildcard_in_scope = if last_validated_scope_wildcard
                                == (&subpath, &field_path, true)
                            {
                                true
                            } else {
                                // check path
                                let mut temp_scope: String;
                                let path_for_scope_test = if subpath.is_empty() {
                                    field_path
                                } else {
                                    temp_scope = subpath.clone();
                                    if !temp_scope.ends_with('_') {
                                        temp_scope.push('_');
                                    }
                                    temp_scope.push_str(field_path);
                                    temp_scope.as_str()
                                };

                                if wildcard_scope
                                    .contains_all_fields_from_path(path_for_scope_test)
                                {
                                    last_validated_scope_wildcard =
                                        (&subpath, &field_path, true);
                                    true
                                } else {
                                    let field_for_scope_test = if subpath.is_empty() {
                                        field_name
                                    } else {
                                        temp_scope = subpath.clone();
                                        if !temp_scope.ends_with('_') {
                                            temp_scope.push('_');
                                        }
                                        temp_scope.push_str(field_name);
                                        temp_scope.as_str()
                                    };
                                    //println!("Test {} in {:?}", field_for_scope_test, self.wildcard_scope);
                                    wildcard_scope.contains_field(&field_for_scope_test)
                                }
                            };

                            if !wildcard_in_scope {
                               //    println!("Skipped {:?}", field_name);
                                continue;
                            } else {
                               // println!("Included {:?}", field_name);
                            }

                            // Skip field if it doesn't belong to wildcard path
                            if !field_path.starts_with(wildcard_path) {
                                continue;
                            }

                            // Skip ignored paths, they belong typically to merged fields and are handled by another build() call
                            if ignored_paths.iter().any(|p| field_name.starts_with(p)) {
                                continue;
                            }

                            if sql_target.options.skip_wildcard {
                                continue;
                            }

                            // Skip fields with missing role
                            if assert_roles(&roles, &sql_target.options.roles).is_err() {
                                continue;
                            }

                            // Skip field paths, that are marked with ignore wildcard or have missing roles
                            if !field_path.is_empty() {
                                if field_path != last_validated_path.0 {
                                    let mut path = field_path;
                                    // Remember successful validation to speed up next validation of the same path
                                    last_validated_path.0 = field_path;
                                    while !path.is_empty() {
                                        // Validate path only up to wildcard path
                                        if path == wildcard_path {
                                            last_validated_path = (path, true);
                                            break;
                                        }

                                        //if ignore wildcard, roles missing validated_path = (path, false)
                                        if let Some(join) = sql_mapper.joins.get(path) {
                                            if join.options.skip_wildcard {
                                                last_validated_path = (path, false);
                                                break;
                                            }
                                        }

                                        //println!("PATH={}", path);
                                        // Next path
                                        path = path
                                            .trim_end_matches(|c| c != '_')
                                            .trim_end_matches('_');
                                    }
                                }

                                // Skip any field on path with failed validation
                                if last_validated_path.1 != true {
                                    // println!("Path {} is invalid!", last_validated_path.0);
                                    continue;
                                }
                                /*  else {
                                    //println!("Path {} is valid!", last_validated_path.0);
                                } */
                            }

                            // Select all fields on wildcard path
                            // including joins with preselected fields only

                            //let select = (field_path == wildcard_path) || field_path.starts_with(wildcard_path) && sql_target.options.preselect;

                            // let select = field_path.starts_with(wildcard_path);
                            //println!( "field {}: field_path={}, wildcard_path ={}, select={}",&field_name, field_path, &wildcard.path, select);

                            /* if (wildcard.path.is_empty())  && ! sql_target.subfields
                            || (field_name.starts_with(&wildcard.path)
                                && field_name.rfind("_").unwrap_or(field_name.len())
                                    < wildcard.path.len()) */
                            /*   if select
                            { */
                            let f = sql_target_data.entry(field_name.to_string()).or_default();

                            f.selected = true; // Select field
                            f.used = true;      // Used field because selected

                            // Ensure all parent paths are selected
                            if sql_target.subfields {
                                let mut path = field_name
                                    .trim_end_matches(|c| c != '_')
                                    .trim_end_matches('_');
                                while !path.is_empty() {
                                    let exists = !selected_paths.insert(path.to_owned());
                                    if exists {
                                        break;
                                    }
                                    path =
                                        path.trim_end_matches(|c| c != '_').trim_end_matches('_');
                                }

                                /* for subfield in field_name.split('_').rev().skip(1) {
                                     let exists= selected_paths.insert(subfield);
                                if exists { break;}
                                } */
                            }
                            // }
                        }
                    }
                    QueryToken::Field(query_field) => {
                        // Ignore field if name does not start with path
                        // E.g "user_id" has path "user"
                        if !subpath.is_empty() && !query_field.name.starts_with(subpath)
                        {
                            continue;
                        }
                        if ignored_paths
                            .iter()
                            .any(|p| query_field.name.starts_with(p))
                        {
                            continue;
                        }

                        let field_name = if subpath.is_empty() {
                            &query_field.name
                        } else {
                            query_field
                                .name
                                .trim_start_matches(subpath)
                                .trim_start_matches('_')
                        };

                        match sql_mapper.fields.get(field_name) {
                            Some(sql_target) => {
                                // Verify user role and skip field role mismatches
                                assert_roles(&roles, &sql_target.options.roles)
                                    .map_err(|role| SqlBuilderError::RoleRequired(role))?;

                                // Skip filtering and ordering in count queries for unfiltered fields
                                if mode == &BuildMode::CountFiltered && !sql_target.options.count_filter {
                                    continue;
                                }

                                // Skip field that cannot neither be selected and filtered
                                if !sql_target.options.query_select {
                                    continue;
                                }

                                /* if self.count_query == true && !sql_target.options.count_filter {
                                    continue;
                                } */

                                // Select sql target if field is not hidden
                                let data = sql_target_data.entry(field_name.to_string()).or_default();

                                // Add parent Joins
                                if sql_target.subfields {
                                    insert_paths(field_name, &mut selected_paths);
                                    /* let mut path = field_name
                                        .trim_end_matches(|c| c != '_')
                                        .trim_end_matches('_');
                                    while !path.is_empty() {
                                        let exists = !selected_paths.insert(path.to_owned());
                                        if exists {
                                            break;
                                        }
                                        path = path
                                            .trim_end_matches(|c| c != '_')
                                            .trim_end_matches('_');
                                    } */
                                }
                                //println!("{:?}", selected_paths);

                                data.selected = match mode {
                                    BuildMode::CountFiltered => sql_target.options.count_select,
                                    BuildMode::DeleteQuery => false,
                                    BuildMode::SelectAll => true,
                                    BuildMode::SelectMut => sql_target.options.mut_select,
                                    BuildMode::SelectQuery => !query_field.hidden
                                };
                              /*   if self.count_query {
                                    sql_target.options.count_select
                                } else {
                                    !query_field.hidden
                                }; */

                                // Target is used if it's selected
                                data.used = data.selected;  

                               // data.used = !query_field.hidden;

                                // TODO fix bug
                                // Resolve query params in sql expression
                                /* for p in &sql_target.sql_query_params {
                                    let qp = query
                                        .params
                                        .get(p)
                                        .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))?;

                                    if query_field.aggregation == true {
                                        result.having_params.push(qp.to_string());
                                    } else {
                                        result.where_params.push(qp.to_string());
                                    }
                                } */

                                if let Some(f) = &query_field.filter {
                                    // Combine aux params from query and target
                                    let mut combined_aux_params: HashMap<String, SqlArg> =
                                        HashMap::new();
                                    let aux_params = combine_aux_params(
                                        &mut combined_aux_params,
                                        &build_aux_params,
                                        &sql_target.options.aux_params,
                                    );

                                    let aux_param_values = aux_param_values(&sql_target.sql_aux_param_names, aux_params)?;
                                        

                                    let expression = sql_target
                                        .handler
                                        .build_select(
                                            (sql_target.expression.to_owned(), aux_param_values),
                                            aux_params,
                                        )?
                                        .unwrap_or(("null".to_string(), vec![]));

                                    if let Some(f) = sql_target.handler.build_filter(
                                        expression, 
                                        &f, aux_params
                                    )? {

                                        // Valid filter on target makes it used
                                         data.used = data.used | true;

                                        if query_field.aggregation == true {
                                            if need_having_concatenation == true {
                                                if pending_having_parens > 0 {
                                                    BuildResult::push_concatenation(
                                                        &mut result.having_clause,
                                                        &pending_having_parens_concatenation,
                                                    );
                                                } else {
                                                    BuildResult::push_concatenation(
                                                        &mut result.having_clause,
                                                        &Some(query_field.concatenation.clone()), // OPTIMISE
                                                    );
                                                }
                                            }

                                            BuildResult::push_pending_parens(
                                                &mut result.having_clause,
                                                &pending_having_parens,
                                            );

                                            BuildResult::push_filter(
                                                &mut result.having_clause,
                                                &f.0,
                                            );
                                            if query_field.aggregation == true {
                                                result.having_params.extend_from_slice(&f.1);
                                            } else {
                                                result.where_params.extend_from_slice(&f.1);
                                            }

                                            need_having_concatenation = true;
                                            pending_having_parens = 0;
                                        } else {
                                            if need_where_concatenation == true {
                                                if pending_where_parens > 0 {
                                                    BuildResult::push_concatenation(
                                                        &mut result.where_clause,
                                                        &pending_where_parens_concatenation,
                                                    );
                                                } else {
                                                    BuildResult::push_concatenation(
                                                        &mut result.where_clause,
                                                        &Some(query_field.concatenation.clone()), // IMPROVE
                                                    );
                                                }
                                            }
                                            BuildResult::push_pending_parens(
                                                &mut result.where_clause,
                                                &pending_where_parens,
                                            );
                                            BuildResult::push_filter(
                                                &mut result.where_clause,
                                                &f.0,
                                            );
                                            if query_field.aggregation == true {
                                                result.having_params.extend_from_slice(&f.1);
                                            } else {
                                                result.where_params.extend_from_slice(&f.1);
                                            }

                                            pending_where_parens = 0;
                                            need_where_concatenation = true;
                                        }
                                    }

                                // Add Parameters for on clauses
                                if let FieldFilter::Eq(a) | FieldFilter::Ne(a) = f {
                                    for n in &sql_target.options.on_params {
                                        on_params.insert(n.to_string(), a.to_owned());
                                    }
                                }
                                
                                    // TODO Test correct aux_params provided
                                   // Sql Target aux params, join aux params?
                                   /*  if let Some((j, p)) =
                                        sql_target.handler.build_join(aux_params)?
                                    {
                                        result.join_clause.push_str(&j);
                                        result.join_clause.push_str(" ");
                                        result.join_params.extend_from_slice(&p);
                                    }  */
                                }
                                if let Some(o) = &query_field.order {
                                    let num = match o {
                                        FieldOrder::Asc(num) => num,
                                        FieldOrder::Desc(num) => num,
                                    };
                                    ordinals.insert(*num);
                                    let l = ordering.entry(*num).or_insert(Vec::new());
                                    l.push((o.clone(), query_field.name.clone()));
                                    // OPTIMISE
                                }
                            }
                            None => {
                                // If field has path, validate to known paths
                                if !query_field.name.contains("_")
                                    || !path_ignored(&ignored_paths, &query_field.name)
                                {
                                    return Err(SqlBuilderError::FieldMissing(
                                        query_field.name.clone(),
                                    ));
                                }
                            }
                        }
                    },
                     QueryToken::Predicate(query_predicate) => {
                         
                         // Predicates work only on base entity
                        if !subpath.is_empty()
                        {
                            continue;
                        }
                          

                         match sql_mapper.predicates.get(&query_predicate.name) {
                            Some(predicate) => {

                                if mode == &BuildMode::CountFiltered && !predicate.options.count_filter {
                                    continue;
                                }

                                fn predicate_param_values(aux_param_names: &Vec<String>, aux_params: &HashMap<String, SqlArg>, predicate_args: &Vec<SqlArg>, predicate_name:&str ) -> Result<Vec<SqlArg>, SqlBuilderError>{
                                      let mut params: Vec<SqlArg> = Vec::with_capacity(aux_param_names.len());
                                      let mut i = 0usize;
                                    for p in aux_param_names {
                                       let value =  if p == "?" {
                                            (predicate_args.get(i).ok_or(SqlBuilderError::PredicateArgumentMissing(predicate_name.to_string())),
                                            i = i + 1).0
                                        } else {
                                        aux_params
                                            .get(p)
                                            .ok_or(SqlBuilderError::QueryParamMissing(p.to_string()))
                                        }?;
                                        params.push(value.to_owned());
                                    };
                                    Ok(params)
                                }

                                
                                
                                 let mut combined_aux_params: HashMap<String, SqlArg> =
                                        HashMap::new();
                                    let aux_params = combine_aux_params(
                                        &mut combined_aux_params,
                                        &build_aux_params,
                                        &predicate.options.aux_params,
                                    );

                                let args = predicate_param_values(&predicate.sql_aux_param_names, &aux_params,  &query_predicate.args,&query_predicate.name )?;
                                if let Some((expr, args)) = predicate.handler.build_predicate(( predicate.expression.to_owned(), args),&query_predicate.args, aux_params)? {

                                   if need_where_concatenation == true {
                                        if pending_where_parens > 0 {
                                            BuildResult::push_concatenation(
                                                &mut result.where_clause,
                                                &pending_where_parens_concatenation,
                                            );
                                        } else {
                                            BuildResult::push_concatenation(
                                                &mut result.where_clause,
                                                &Some(query_predicate.concatenation.clone()), // IMPROVE
                                            );
                                        }
                                    }
                                    BuildResult::push_pending_parens(
                                        &mut result.where_clause,
                                        &pending_where_parens,
                                    );
                                    BuildResult::push_filter(
                                        &mut result.where_clause,
                                        &expr,
                                    );
                                   
                                    result.where_params.extend_from_slice(&args);
                                    
                                    pending_where_parens = 0;
                                    need_where_concatenation = true;
                                }

                                // Add On Parameters for on clauses 
                                for (i,n) in &predicate.options.on_params {
                                    let a = query_predicate.args.get(*i as usize).ok_or(SqlBuilderError::PredicateArgumentMissing(query_predicate.name.to_string()))?;
                                    on_params.insert(n.to_string(), a.to_owned());
                                }

                                
                            


                            },
                            None => {
                                 return Err(SqlBuilderError::PredicateMissing(
                                        query_predicate.name.clone(),
                                    ));
                            }
                         }
                     }
                }
            }
            
        }
        Ok(())

}
