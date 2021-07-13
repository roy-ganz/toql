//pub mod api;
pub mod context;
pub mod context_builder;
pub mod fields;
pub mod insert;
pub mod paths;
pub mod update;


mod map;

//pub mod ops;

use async_trait::async_trait;

use crate::{
    error::ToqlError,
    from_row::FromRow,
    keyed::Keyed,
    sql_mapper::mapped::Mapped,
    tree::{
        tree_identity::{IdentityAction, TreeIdentity}, tree_index::TreeIndex, tree_insert::TreeInsert,
        tree_map::TreeMap, tree_merge::TreeMerge, tree_predicate::TreePredicate,
        tree_update::TreeUpdate,
    }, sql_mapper_registry::SqlMapperRegistry, alias_format::AliasFormat, sql_arg::SqlArg, sql::Sql, query::{Query, field_path::FieldPath}, sql_expr::{SqlExpr, PredicateColumn, resolver::Resolver}, sql_builder::{build_result::BuildResult, SqlBuilder}, alias_translator::AliasTranslator, parameter_map::ParameterMap, page::Page,
};
use std::{borrow::Borrow, collections::{HashMap, HashSet}, sync::{RwLockWriteGuard, RwLockReadGuard}};
use fields::Fields;
use paths::Paths;


pub trait Load<R, E>:
    Keyed
    + Mapped
    + TreeMap
    + FromRow<R, E>
    + TreePredicate
    + TreeIndex<R, E>
    + TreeMerge<R, E>
    + std::fmt::Debug
where
    <Self as Keyed>::Key: FromRow<R, E>,
    E: std::convert::From<ToqlError>,
{
}

pub trait Insert: TreeInsert + Mapped + TreeIdentity +TreeMap{}
pub trait Update: TreeUpdate + Mapped + TreeIdentity + TreePredicate + TreeInsert + TreeMap{}

pub trait Count: Keyed + Mapped + std::fmt::Debug {}

pub trait Delete: Mapped + TreeMap + std::fmt::Debug {}


#[async_trait]
pub trait Backend<R,E> 
where  E: From<ToqlError>
{ // <R,E> for row error
   fn registry(&self) ->Result<RwLockReadGuard<'_, SqlMapperRegistry>, ToqlError>;
   fn registry_mut(&mut self) -> Result<RwLockWriteGuard<'_, SqlMapperRegistry>, ToqlError>;
   fn roles(&self) -> &HashSet<String>;
   fn alias_format(&self) -> AliasFormat;
   fn aux_params(&self) -> &HashMap<String, SqlArg>;

   async fn select_sql(&mut self, sql:Sql) -> Result<Vec<R>, E>;
   fn prepare_page(&self, result: &mut BuildResult, start:u64, number_of_records: u16); // Modify result, so that page with unlimited page size can be loaded
   async fn select_max_page_size_sql(&mut self, sql:Sql) -> Result<u32, E>; // Load page and number of records without page limitation
   async fn select_count_sql(&mut self, sql:Sql) -> Result<u32, E>; // Load single value

   async fn execute_sql(&mut self, sql:Sql) -> Result<(), E>;
   async fn insert_sql(&mut self, sql:Sql) -> Result<Vec<SqlArg>, E>; // New keys

async fn load_count<T, B>(
    &mut self,
    query: &B,
) -> Result<u32, E>
where
    T: Load<R, E>,
    B: Borrow<Query<T>> + Send + Sync,
    <T as Keyed>::Key: FromRow<R, E>, 
  
 {
   
           
            let ty = <T as Mapped>::type_name();
            //let sql = build_load_count_sql(self.alias_format(), registry, ty)?;

            let alias_format = self.alias_format();
            let mut alias_translator = AliasTranslator::new(alias_format);
            let aux_params = [self.aux_params()];
            let aux_params = ParameterMap::new(&aux_params);

            let sql ={
                let registry =  &*self.registry()?;
                let mut builder = SqlBuilder::new(&ty, registry)
                    .with_aux_params(self.aux_params().clone()) // todo ref
                    .with_roles(self.roles().clone()); // todo ref;
                let result = builder.build_count("", query.borrow(), true)?;
                let sql = result
                    .to_sql(&aux_params, &mut alias_translator)
                    .map_err(ToqlError::from)?;
                sql
            };
        

            log_sql!(&sql);
            
            let page_count = self.select_count_sql(sql).await?;
            /* let Sql(sql_stmt, args) = sql;
            let args = crate::sql_arg::values_from_ref(&args);
            let query_results = mysql.conn.prep_exec(sql_stmt, args)?;
            query_results
                .into_iter()
                .next()
                .unwrap()
                .unwrap()
                .get(0)
                .unwrap()
        };
        Some((unpaged_count, unfiltered_count)) */
    
    Ok(page_count)
}

async fn load_and_merge<T, B>(
    &mut self,
    query: &B,
    entities: &mut Vec<T>,
    unmerged_home_paths: &HashSet<String>
) -> std::result::Result<HashSet<String>, E>
where
    T: Keyed
        + Mapped
        + FromRow<R, E>
        + TreePredicate
        + TreeIndex<R, E>
        + TreeMerge<R, E>
        + Send
        + std::fmt::Debug,

    B: Borrow<Query<T>> + Sync,
    <T as crate::keyed::Keyed>::Key: FromRow<R, E>, 
    E: From<ToqlError>
    
{
    

    let ty = <T as Mapped>::type_name();
    let mut pending_home_paths = HashSet::new();

    let canonical_base = {
        let registry = self.registry()?;
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
            let registry = self.registry()?;
            let mut builder = SqlBuilder::new(&ty, &*registry)
                .with_aux_params(self.aux_params().clone()) // todo ref
                .with_roles(self.roles().clone()); // todo ref// Add alias format or translator to constructor
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
            let registry = self.registry()?;
            let builder = SqlBuilder::new(&ty, &*registry)
                .with_aux_params(self.aux_params().clone()) // TODO ref
                .with_roles(self.roles().clone());
            builder.merge_expr(&home_path)?
        };

        let merge_join_predicate = merge_resolver.resolve(&merge_join_predicate).map_err(ToqlError::from)?;

        // Get key columns 
        let (merge_join, key_select_expr) = {
           let parent_home_path = parent_home_path.unwrap_or_default(); 
            let registry = self.registry()?;
            let builder = SqlBuilder::new(&ty, &*registry); // No aux params for key
            let (key_select_expr, key_join) =
                builder.columns_expr(parent_home_path.as_str(), &merge_base_alias)?;

            let merge_join = if key_join.is_empty() {
                &merge_join_sql_expr
            } else {
                merge_join_sql_expr.push_literal(" ").extend(key_join)
            };
           
            (merge_resolver.resolve(merge_join).map_err(ToqlError::from)?, key_select_expr)
          
        };

        result.set_preselect(key_select_expr); // Select key columns for indexing
        result.push_join(merge_join);
        result.push_join(SqlExpr::literal(" ON ("));
        result.push_join(merge_join_predicate);
      
        // Get ON predicate from entity keys
        let mut predicate_expr = SqlExpr::new();
        let (_field, ancestor_path) = FieldPath::split_basename(home_path.as_str());
        // let ancestor_path = ancestor_path.unwrap_or(FieldPath::from(""));
        //let mut d = ancestor_path.descendents();
        let mut d = ancestor_path.children();

        let columns =
            TreePredicate::columns(entities.get(0).unwrap(), &mut d).map_err(ToqlError::from)?;

        let mut args = Vec::new();
        //let mut d = ancestor_path.descendents();
        let mut d = ancestor_path.children();
        for e in entities.iter() {
            TreePredicate::args(e, &mut d, &mut args).map_err(ToqlError::from)?;
        }
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

        let mut alias_translator = AliasTranslator::new(self.alias_format());
        let aux_params = [self.aux_params()];
        let aux_params = ParameterMap::new(&aux_params);
        let sql = result
            .to_sql(&aux_params, &mut alias_translator)
            .map_err(ToqlError::from)?;
        log_sql!(sql);
        
        //let Sql(sql, args) = sql;
       /*  dbg!(&sql);
        dbg!(&args); */

        // Load from database
        let rows = self.select_sql(sql).await?; // Default vector size
       /*  let args = crate::sql_arg::values_from_ref(&args);
        let query_results = mysql.conn.prep_exec(sql, args)?; */

        // Build index
        let mut index: HashMap<u64, Vec<usize>> = HashMap::new(); //hashed key, array positions

        let (field, ancestor_path) = FieldPath::split_basename(home_path.as_str());

        // TODO Batch process rows
        // TODO Introduce traits that do not need to copy into vec
      /*   let mut rows = Vec::with_capacity(100);

        for q in query_results {
            rows.push(Row(q?)); // Stream into Vec because we need random access
        } */

        let row_offset = 0; // key must be first columns in row

        let (_, ancestor2_path) = FieldPath::split_basename(ancestor_path.as_str());
        //let mut d = ancestor2_path.descendents();
        let mut d = ancestor2_path.children();
        <T as TreeIndex<R, E>>::index(&mut d, &rows, row_offset, &mut index)?;

        //let mut d = ancestor_path.descendents();
    
        // Merge into entities
      //  dbg!(result.selection_stream());

        for e in entities.iter_mut() {
            let mut d = ancestor_path.children();
            <T as TreeMerge<_, E>>::merge(
                e,
                &mut d,
                field,
                &rows,
                row_offset,
                &index,
                result.selection_stream(),
            )?;
        }
    }
    Ok(pending_home_paths)
}

async fn load_top<T, B>(
    &mut self,
    query: &B,
    page: Option<Page>,
) -> std::result::Result<(Vec<T>, HashSet<String>, Option<(u32, u32)>), E>
where
    T: Load<R, E> + Send + FromRow<R, E>,
    B: Borrow<Query<T>> + Sync + Send,
    <T as crate::keyed::Keyed>::Key: FromRow<R, E>, 
    E: From<ToqlError>
{
   
    let alias_format = self.alias_format();

    let ty = <T as Mapped>::type_name();

    let mut result = {
        let registry = &*self.registry()?;
        let mut builder = SqlBuilder::new(&ty, registry)
            .with_aux_params(self.aux_params().clone()) // todo ref
            .with_roles(self.roles().clone()); // todo ref;
        builder.build_select("", query.borrow())?
    };

    let unmerged = result.unmerged_home_paths().clone();
    let mut alias_translator = AliasTranslator::new(alias_format);
    let aux_params = [self.aux_params()];
    let aux_params = ParameterMap::new(&aux_params);
/* 
    let extra = match page {
        Some(Page::Counted(start, number_of_records)) => {
            self.prepare_page(&mut result, start, number_of_records);
            //Cow::Owned(format!("LIMIT {}, {}", start, number_of_records))
        }
        Some(Page::Uncounted(start, number_of_records)) => {
            self.prepare_page(&mut result, start, number_of_records);
            //Cow::Owned(format!("LIMIT {}, {}", start, number_of_records))
        }
        None => {}//Cow::Borrowed(""),
    };

    let modifier = if let Some(Page::Counted(_, _)) = page {
        "SQL_CALC_FOUND_ROWS"
    } else {
        ""
    };
 */
    // 
    


    let sql = {
        result
            .to_sql(
                &aux_params,
                &mut alias_translator,
            )
        /* result
            .to_sql_with_modifier_and_extra(
                &aux_params,
                &mut alias_translator,
                modifier,
                extra.borrow(),
            ) */
            .map_err(ToqlError::from)?
    };

    log_sql!(&sql);
    
    let entities=  {
        let rows = self.select_sql(sql).await?;
            let mut entities = Vec::with_capacity(rows.len());

            for r in rows {
                let mut iter = result.selection_stream().iter();
                let mut i = 0usize;
                if let Some(e) =
                    <T as FromRow<R, E>>::from_row(&r, &mut i, &mut iter)?
                {
                    entities.push(e);
                }
            }
        entities
    };

    
    
    
    let page_count = match page {
        Some(Page::Counted(start,number_of_records))=> {
                let count_sql = Sql::new(); // TODO
                let unfiltered_page_size = self.select_count_sql(count_sql).await?;
                self.prepare_page(&mut result, start, number_of_records);
                let max_page_size_sql = result.to_count_sql(&mut alias_translator).map_err(|e|e.into())?;
                let max_page_size =  self.select_max_page_size_sql(max_page_size_sql).await?;
                Some((unfiltered_page_size, max_page_size))
        },
        _ => None,
    };
    

   // let Sql(sql_stmt, args) = sql;
   
    /* let mut entities: Vec<T> = Vec::with_capacity(capactity.into());
    for r in query_results {
        let r = Row(r?);
      //  dbg!(result.selection_stream());
        let mut iter = result.selection_stream().iter();
        let mut i = 0usize;
        if let Some(e) =
            <T as toql::from_row::FromRow<Row, ToqlMySqlError>>::from_row(&r, &mut i, &mut iter)?
        {
            entities.push(e);
        }
    } */

    // Retrieve count information
  /*   let page_count = if let Some(Page::Counted(_, _)) = page {
        Some((count, self.load_count(query)?))
    } else {
        None
    }; */

    Ok((entities, unmerged, page_count))
}

async fn load<T, B>(
    &mut self,
    query: B,
    page: Option<Page>
) -> std::result::Result<(Vec<T>, Option<(u32, u32)>), E>
where
    E: From<ToqlError>,
    T: Load<R, E> + Send,
    B: Borrow<Query<T>> + Sync + Send,
    <T as Keyed>::Key: FromRow<R, E>,
{
    {
        let registry = &mut *self.registry_mut()?;
        map::map::<T>(registry)?;
    }

    let (mut entities, unmerged_paths, counts) = self.load_top(&query, page).await?;
    let mut pending_paths = unmerged_paths;
    loop {
        pending_paths = self.load_and_merge( &query, &mut entities, &pending_paths).await?;

        // Quit, if all paths have been merged
        if pending_paths.is_empty() {
            break;
        }

        // Select and merge next paths
        // unmerged_paths.extend(pending_paths.drain());
    }

    Ok((entities, counts))
}

   async fn update<T>(&mut self, entities: &mut [T], fields: Fields) ->Result<(), E> where 
    T: Update + Send + Sync,
    {
        use update::{plan_update_order, build_update_sql};
        use insert::build_insert_sql;
        use std::borrow::{Borrow, BorrowMut};
        

         // TODO should be possible to impl with &str
            let mut joined_or_merged_fields: HashMap<String, HashSet<String>> = HashMap::new();
            let mut merges: HashMap<String, HashSet<String>> = HashMap::new();

            // Ensure entity is mapped
             {
                let registry = &mut *self.registry_mut()?;
                map::map::<T>(registry)?;
            }


          /*   if !self.registry()?.mappers.contains_key(<T as Mapped>::type_name().as_str()){
                 let registry = &mut *self.registry_mut()?;
                <T as TreeMap>::map( self.registry_mut()?)?;
            } */


            plan_update_order::<T, _>(
                &self.registry()?.mappers,
                fields.list.as_ref(),
                &mut joined_or_merged_fields,
                &mut merges,
            )?;

            for (path, fields) in joined_or_merged_fields {
                let sqls = {
                    let field_path = FieldPath::from(&path);
                    build_update_sql::<T, _>(
                        self.alias_format(),
                        entities,
                        &field_path,
                        &fields,
                        &self.roles(),
                        "",
                        "",
                    )
                }?;

                // Update joins
                for sql in sqls {
                    self.execute_sql(sql).await?;
                }
            }

            // Delete existing merges and insert new merges

            for (path, fields) in merges {
                // Build delete sql
               /*  dbg!(&path);
                dbg!(&fields); */

                let parent_path = FieldPath::from(&path);
                let entity = entities.get(0).unwrap().borrow();
                let columns = <T as TreePredicate>::columns(entity, &mut parent_path.children())?;
                let mut args = Vec::new();
                for e in entities.iter() {
                    <T as TreePredicate>::args(e.borrow(), &mut parent_path.children(), &mut args)?;
                }
                let columns = columns
                    .into_iter()
                    .map(|c| PredicateColumn::SelfAliased(c))
                    .collect::<Vec<_>>();

                // Construct sql
                let mut key_predicate: SqlExpr = SqlExpr::new();
                key_predicate.push_predicate(columns, args);

                for merge in fields {
                    let merge_path = FieldPath::from(&merge);
                    let sql = {
                        let type_name = <T as Mapped>::type_name();
                        let registry =  &*self.registry()?;
                        let mut sql_builder = SqlBuilder::new(&type_name,registry)
                            .with_aux_params(self.aux_params().clone()) // todo ref
                            .with_roles(self.roles().clone()); // todo ref
                        let delete_expr =
                            sql_builder.build_merge_delete(&merge_path, key_predicate.to_owned())?;

                        let mut alias_translator = AliasTranslator::new(self.alias_format().clone());
                        let resolver = Resolver::new();
                        resolver
                            .to_sql(&delete_expr, &mut alias_translator)
                            .map_err(ToqlError::from)?
                    };

                    //dbg!(sql.to_unsafe_string());
                    self.execute_sql(sql).await?;
                    

                    // Ensure primary keys of collection are valid
                    // This is needed, if entities have been added to the collection
                    // without valid primary keys
                    for e in entities.iter_mut() {
                        let mut descendents = parent_path.children();
                        <T as TreeIdentity>::set_id(
                            e.borrow_mut(),
                            &mut descendents,
                            &IdentityAction::Refresh,
                        )?;
                    } 

                    // Insert
                    let aux_params = [self.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    let sql = build_insert_sql(
                        &self.registry()?.mappers,
                        self.alias_format().clone(),
                        &aux_params,
                        entities,
                        self.roles(),
                        &merge_path,
                        "",
                        "",
                    )?;
                    if let Some(sql) = sql {
                        self.execute_sql(sql).await?;

                        // TODO read auto keys and assign

                    }
                }
            }

            Ok(())
        }
         async fn insert<T>(&mut self, mut entities: &mut [T], paths: Paths) ->Result<u64, E> where 
            T: Insert + Send,
    {
        
         {
                let registry = &mut *self.registry_mut()?;
                map::map::<T>(registry)?;
            }
     /* 
        if !self.registry()?.mappers.contains_key(<T as Mapped>::type_name().as_str()){
            <T as TreeMap>::map(self.registry_mut()?)?;
        } */
        

        // Build up execution tree
        // Path `a_b_merge1_c_d_merge2_e` becomes
        // [0] = [a, c, e]
        // [1] = [a_b, c_d]
        // [m] = [merge1, merge2]
        // Then execution order is [1], [0], [m]


        // TODO should be possible to impl with &str
        let mut joins: Vec<HashSet<String>> = Vec::new();
        let mut merges: HashSet<String> = HashSet::new();



        crate::backend::insert::plan_insert_order::<T, _>(
            &self.registry()?.mappers,
            paths.list.as_ref(),
            &mut joins,
            &mut merges,
        )?;

        // Insert root
        let sql = {
            let aux_params = [self.aux_params()];
            let aux_params = ParameterMap::new(&aux_params);
            let home_path = FieldPath::default();

            crate::backend::insert::build_insert_sql::<T, _>(
                &self.registry()?.mappers,
                self.alias_format(),
                &aux_params,
                entities,
                &self.roles(),
                &home_path,
                "",
                "",
            )
        }?;
        if sql.is_none() {
            return Ok(0);
        }
        let sql = sql.unwrap();

        let home_path = FieldPath::default();
        let mut descendents = home_path.children();
        // check if base has auto keys
        if <T as TreeIdentity>::auto_id(&mut descendents)? {
            let ids= self.insert_sql(sql).await?;

            let mut descendents = home_path.children();
            crate::backend::insert::set_tree_identity2(
                ids ,
                &mut entities,
                &mut descendents,
            )?;
         }else {
            self.execute_sql(sql).await?;
         }


        // Insert joins
        for l in (0..joins.len()).rev() { // TEST not rev
            for p in joins.get(l).unwrap() {
                let mut path = FieldPath::from(&p);

                let sql = {
                    let aux_params = [self.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    crate::backend::insert::build_insert_sql::<T, _>(
                        &self.registry()?.mappers,
                        self.alias_format(),
                        &aux_params,
                        entities,
                        &self.roles(),
                        &mut path,
                        "",
                        "",
                    )
                }?;
                if sql.is_none() {
                    break;
                }
                let sql = sql.unwrap();

            let mut descendents = path.children();
            if <T as TreeIdentity>::auto_id(&mut descendents)? {
                let ids= self.insert_sql(sql).await?;

                let mut descendents = home_path.children();
                crate::backend::insert::set_tree_identity2(
                    ids ,
                    &mut entities,
                    &mut descendents,
                )?;
             } else {
                self.execute_sql(sql).await?;
             }
            }
        }

        // Insert merges
        for p in merges {
            let path = FieldPath::from(&p);

            let sql = {
                let aux_params = [self.aux_params()];
                let aux_params = ParameterMap::new(&aux_params);
                crate::backend::insert::build_insert_sql::<T, _>(
                    &self.registry()?.mappers,
                    self.alias_format(),
                    &aux_params,
                    entities,
                    &self.roles(),
                    &path,
                    "",
                    "",
                )
            }?;
            if sql.is_none() {
                break;
            }
            let sql = sql.unwrap();

            // Merges must not contain auto value as identity, skip set_tree_identity
            self.execute_sql(sql).await?;

        }

        Ok(0)
    }
    }

