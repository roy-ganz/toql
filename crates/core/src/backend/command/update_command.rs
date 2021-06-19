
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
use  crate::backend::fields::Fields;

use crate::sql_mapper_registry::SqlMapperRegistry;
use crate::alias_format::AliasFormat;
use crate::error::ToqlError;
use crate::result::Result;
use crate::parameter_map::ParameterMap;

use std::collections::{HashMap, HashSet};
use std::borrow::BorrowMut;
 use std::borrow::Borrow;





/// Database backends can use this command to implement their specific update strategy.
pub struct UpdateCommand<'a> {
    registry: &'a SqlMapperRegistry,
    execute_sql: &'a mut dyn FnMut(Sql) -> Result<()>,
    roles: HashSet<String>,
    alias_format: AliasFormat,
    aux_params: HashMap<String, SqlArg>
}

impl<'a> UpdateCommand<'a> {

    pub fn new(registry: &'a SqlMapperRegistry, execute_sql: &'a mut dyn FnMut(Sql)-> Result<()>) -> Self{
        Self {
            registry,
            execute_sql,
            alias_format: AliasFormat::Canonical,
            roles: HashSet::new(),
            aux_params: HashMap::new()
        }
    }


pub fn run<T>(&mut self, entities: &mut [T], fields: Fields) ->Result<()> where 
T: Mapped + TreeInsert + TreeIdentity + TreeIdentity + TreeUpdate + TreePredicate
{
      
        // TODO should be possible to impl with &str
        let mut joins: HashMap<String, HashSet<String>> = HashMap::new();
        let mut merges: HashMap<String, HashSet<String>> = HashMap::new();

       

        plan_update_order::<T, _>(
            &self.registry.mappers,
            fields.list.as_ref(),
            &mut joins,
            &mut merges,
        )?;

        for (path, fields) in joins {
            let sqls = {
                let field_path = FieldPath::from(&path);
                build_update_sql::<T, _>(
                    self.alias_format.clone(),
                    entities,
                    &field_path,
                    &fields,
                    &self.roles,
                    "",
                    "",
                )
            }?;

            // Update joins
            for sql in sqls {
                (self.execute_sql)(sql)?;
            }
        }

        // Delete existing merges and insert new merges

        for (path, fields) in merges {
            // Build delete sql
            dbg!(&path);
            dbg!(&fields);

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
                    
                    let mut sql_builder = SqlBuilder::new(&type_name, &self.registry)
                        .with_aux_params(self.aux_params.clone()) // todo ref
                        .with_roles(self.roles.clone()); // todo ref
                    let delete_expr =
                        sql_builder.build_merge_delete(&merge_path, key_predicate.to_owned())?;

                    let mut alias_translator = AliasTranslator::new(self.alias_format.clone());
                    let resolver = Resolver::new();
                    resolver
                        .to_sql(&delete_expr, &mut alias_translator)
                        .map_err(ToqlError::from)?
                };

                //dbg!(sql.to_unsafe_string());
                (self.execute_sql)(sql)?;
                

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
                let aux_params = [&self.aux_params];
                let aux_params = ParameterMap::new(&aux_params);
                let sql = build_insert_sql(
                    &self.registry.mappers,
                    self.alias_format.clone(),
                    &aux_params,
                    entities,
                    &self.roles,
                    &merge_path,
                    "",
                    "",
                )?;
                if let Some(sql) = sql {
                   (self.execute_sql)(sql)?;
                }
            }
        }

        Ok(())
    }
}

