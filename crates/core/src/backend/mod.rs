//pub mod api;
pub mod context;
pub mod context_builder;
pub mod fields;
pub mod insert;
pub mod paths;
pub mod update;

//pub mod ops;



use crate::{
    result::Result,
    error::ToqlError,
    from_row::FromRow,
    keyed::Keyed,
    sql_mapper::mapped::Mapped,
    tree::{
        tree_identity::{IdentityAction, TreeIdentity}, tree_index::TreeIndex, tree_insert::TreeInsert,
        tree_map::TreeMap, tree_merge::TreeMerge, tree_predicate::TreePredicate,
        tree_update::TreeUpdate,
    }, sql_mapper_registry::SqlMapperRegistry, alias_format::AliasFormat, sql_arg::SqlArg, sql::Sql, query::field_path::FieldPath, sql_expr::{SqlExpr, PredicateColumn, resolver::Resolver}, sql_builder::SqlBuilder, alias_translator::AliasTranslator, parameter_map::ParameterMap,
};
use std::collections::{HashMap, HashSet};
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


pub trait Backend {
   fn registry(&self) -> &SqlMapperRegistry;
   fn registry_mut(&mut self) -> &mut SqlMapperRegistry;
   fn roles(&self) -> &HashSet<String>;
   fn alias_format(&self) -> AliasFormat;
   fn aux_params(&self) -> &HashMap<String, SqlArg>;

   fn execute_sql(&mut self, sql:Sql) -> Result<()>;
   fn insert_sql(&mut self, sql:Sql) -> Result<Vec<SqlArg>>; // New keys


   fn update<T>(&mut self, entities: &mut [T], fields: Fields) ->Result<()> where 
    T: Update 
    {
        use update::{plan_update_order, build_update_sql};
        use insert::build_insert_sql;
        use std::borrow::{Borrow, BorrowMut};
        

         // TODO should be possible to impl with &str
            let mut joined_or_merged_fields: HashMap<String, HashSet<String>> = HashMap::new();
            let mut merges: HashMap<String, HashSet<String>> = HashMap::new();

            // Ensure entity is mapped
            if !self.registry().mappers.contains_key(<T as Mapped>::type_name().as_str()){
                <T as TreeMap>::map(&mut self.registry_mut())?;
            }


            plan_update_order::<T, _>(
                &self.registry().mappers,
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
                    self.execute_sql(sql)?;
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
                        
                        let mut sql_builder = SqlBuilder::new(&type_name, &self.registry())
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
                    self.execute_sql(sql)?;
                    

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
                        &self.registry().mappers,
                        self.alias_format().clone(),
                        &aux_params,
                        entities,
                        self.roles(),
                        &merge_path,
                        "",
                        "",
                    )?;
                    if let Some(sql) = sql {
                        self.execute_sql(sql)?;

                        // TODO read auto keys and assign

                    }
                }
            }

            Ok(())
        }
         fn insert<T>(&mut self, mut entities: &mut [T], paths: Paths) ->Result<u64> where 
            T: Insert,
    {
        
     
        if !self.registry().mappers.contains_key(<T as Mapped>::type_name().as_str()){
            <T as TreeMap>::map(&mut self.registry_mut())?;
        }
        

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
            &self.registry().mappers,
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
                &self.registry().mappers,
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
            let ids= self.insert_sql(sql)?;

            let mut descendents = home_path.children();
            crate::backend::insert::set_tree_identity2(
                ids ,
                &mut entities,
                &mut descendents,
            )?;
         }else {
            self.execute_sql(sql)?;
         }


        // Insert joins
        for l in (0..joins.len()).rev() { // TEST not rev
            for p in joins.get(l).unwrap() {
                let mut path = FieldPath::from(&p);

                let sql = {
                    let aux_params = [self.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    crate::backend::insert::build_insert_sql::<T, _>(
                        &self.registry().mappers,
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
                let ids= self.insert_sql(sql)?;

                let mut descendents = home_path.children();
                crate::backend::insert::set_tree_identity2(
                    ids ,
                    &mut entities,
                    &mut descendents,
                )?;
             } else {
                self.execute_sql(sql)?;
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
                    &self.registry().mappers,
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
            self.execute_sql(sql)?;

        }

        Ok(0)
    }
    }

