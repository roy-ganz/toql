
use crate::backend::update::plan_update_order;
use  crate::backend::update::build_update_sql;
use crate::backend::insert::build_insert_sql;
use crate::query::field_path::FieldPath;
use crate::alias_translator::AliasTranslator;
use crate::sql_builder::SqlBuilder;


use crate::tree::tree_identity::IdentityAction;
use crate::tree::{
    tree_identity::TreeIdentity,  tree_map::TreeMap, tree_predicate::TreePredicate, tree_update::TreeUpdate, tree_insert::TreeInsert
   
};
use crate::{
    sql_expr::{resolver::Resolver, PredicateColumn},
    sql_mapper::mapped::Mapped, 
};

 use crate::sql_expr::SqlExpr;
 use crate::sql_arg::SqlArg;
use crate::sql::Sql;
use  crate::backend::{paths::Paths, fields::Fields};

use crate::sql_mapper_registry::SqlMapperRegistry;
use crate::alias_format::AliasFormat;
use crate::error::ToqlError;
use crate::result::Result;
use crate::parameter_map::ParameterMap;

use std::collections::{HashMap, HashSet};
use std::borrow::BorrowMut;
 use std::borrow::Borrow;
use super::conn::Conn;
use crate::backend::context::Context;





pub trait Insert {

   fn insert<T>(&mut self, mut entities: &mut [T], paths: Paths) ->Result<u64> where 
    T: Mapped + TreeInsert + TreeIdentity + TreeIdentity + TreeUpdate + TreePredicate +TreeMap,
     Self: Conn
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
            let aux_params = [&self.context().aux_params];
            let aux_params = ParameterMap::new(&aux_params);
            let home_path = FieldPath::default();

            crate::backend::insert::build_insert_sql::<T, _>(
                &self.registry().mappers,
                self.context().alias_format.clone(),
                &aux_params,
                entities,
                &self.context().roles,
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
                    let aux_params = [&self.context().aux_params];
                    let aux_params = ParameterMap::new(&aux_params);
                    crate::backend::insert::build_insert_sql::<T, _>(
                        &self.registry().mappers,
                        self.context().alias_format.clone(),
                        &aux_params,
                        entities,
                        &self.context().roles,
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
                let aux_params = [&self.context().aux_params];
                let aux_params = ParameterMap::new(&aux_params);
                crate::backend::insert::build_insert_sql::<T, _>(
                    &self.registry().mappers,
                    self.context().alias_format.clone(),
                    &aux_params,
                    entities,
                    &self.context().roles,
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