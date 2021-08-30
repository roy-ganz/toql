use crate::{
    alias_format::AliasFormat,
    alias_translator::AliasTranslator,
    error::ToqlError,
    query::field_path::FieldPath,
    result::Result,
    sql::Sql,
    sql_builder::{SqlBuilder, sql_builder_error::SqlBuilderError},
    sql_expr::{PredicateColumn, resolver::Resolver, SqlExpr},
    table_mapper::{mapped::Mapped, TableMapper},
    tree::{tree_identity::{IdentityAction, TreeIdentity}, tree_update::TreeUpdate, tree_predicate::TreePredicate, tree_map::TreeMap}, parameter_map::ParameterMap,
};

use std::{
    borrow::{BorrowMut, Borrow},
    collections::{HashMap, HashSet},
};
use super::{map, Backend, insert::build_insert_sql};

use crate::toql_api::{update::Update,fields::Fields};


pub async fn update<B, Q, T, R, E>(backend: &mut B, entities: &mut [Q], fields: Fields) ->std::result::Result<(), E> where 
    T: Update,
       Q: BorrowMut<T>,
       B: Backend<R, E>, 
       E: From<ToqlError>
    {
       
        //use insert::build_insert_sql;
        
        

         // TODO should be possible to impl with &str
            let mut joined_or_merged_fields: HashMap<String, HashSet<String>> = HashMap::new();
            let mut merges: HashMap<String, HashSet<String>> = HashMap::new();

            // Ensure entity is mapped
             {
                let registry = &mut *backend.registry_mut()?;
                map::map::<T>(registry)?;
            }


          /*   if !self.registry()?.mappers.contains_key(<T as Mapped>::type_name().as_str()){
                 let registry = &mut *self.registry_mut()?;
                <T as TreeMap>::map( self.registry_mut()?)?;
            } */


            plan_update_order::<T, _>(
                &backend.registry()?.mappers,
                fields.list.as_ref(),
                &mut joined_or_merged_fields,
                &mut merges,
            )?;

            for (path, fields) in joined_or_merged_fields {
                let sqls = {
                    let field_path = FieldPath::from(&path);
                    build_update_sql::<T, _>(
                        backend.alias_format(),
                        entities,
                        &field_path,
                        &fields,
                        &backend.roles(),
                        "",
                        "",
                    )
                }?;

                // Update joins
                for sql in sqls {
                    backend.execute_sql(sql).await?;
                }
            }

            // Delete existing merges and insert new merges

            for (path, fields) in merges {
                // Build delete sql
               /*  dbg!(&path);
                dbg!(&fields); */

                let parent_path = FieldPath::from(&path);
                let entity = entities.get(0).unwrap().borrow();
                let columns = <T as TreePredicate>::columns(&mut parent_path.children())?;
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
                        let registry =  &*backend.registry()?;
                        let mut sql_builder = SqlBuilder::new(&type_name,registry)
                            .with_aux_params(backend.aux_params().clone()) // todo ref
                            .with_roles(backend.roles().clone()); // todo ref
                        let delete_expr =
                            sql_builder.build_merge_delete(&merge_path, key_predicate.to_owned())?;

                        let mut alias_translator = AliasTranslator::new(backend.alias_format().clone());
                        let resolver = Resolver::new();
                        resolver
                            .to_sql(&delete_expr, &mut alias_translator)
                            .map_err(ToqlError::from)?
                    };

                    //dbg!(sql.to_unsafe_string());
                    backend.execute_sql(sql).await?;
                    

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
                    let aux_params = [backend.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    let sql = build_insert_sql(
                        &backend.registry()?.mappers,
                        backend.alias_format().clone(),
                        &aux_params,
                        entities,
                        backend.roles(),
                        &merge_path,
                        "",
                        "",
                    )?;
                    if let Some(sql) = sql {
                        backend.execute_sql(sql).await?;

                        // TODO read auto keys and assign

                    }
                }
            }

            Ok(())
        }

 fn build_update_sql<T, Q>(
    alias_format: AliasFormat,

    entities: &[Q],
    path: &FieldPath,
    fields: &HashSet<String>,
    roles: &HashSet<String>,
    _modifier: &str,
    _extra: &str,
) -> Result<Vec<Sql>>
where
    T: Mapped + TreeUpdate,
    Q: Borrow<T>,
{
    let mut alias_translator = AliasTranslator::new(alias_format);

    let mut update_sqls = Vec::new();

    let mut exprs = Vec::new();
    for e in entities.iter() {
        //let mut descendents = path.descendents();
        let mut descendents = path.children();
        TreeUpdate::update(e.borrow(), &mut descendents, fields, roles, &mut exprs)?;
    }

    // Resolve to Sql

    let resolver = Resolver::new();

    for sql_expr in exprs {
        let update_sql = resolver
            .to_sql(&sql_expr, &mut alias_translator)
            .map_err(ToqlError::from)?;
        update_sqls.push(update_sql);
    }

    Ok(update_sqls)
}

// separate out fields, that refer to merged entities
// E.g on struct user "userLanguage_order" will update all orders in userLanguages
// "userLanguage" refers to merges -> will replace rows
fn plan_update_order<T, S: AsRef<str>>(
    mappers: &HashMap<String, TableMapper>,
    query_paths: &[S],
    fields: &mut HashMap<String, HashSet<String>>, // paths that refer to fields
    merges: &mut HashMap<String, HashSet<String>>, // paths that refer to merges
) -> Result<()>
where
    T: Mapped,
{
    let ty = <T as Mapped>::type_name();
    for path in query_paths {
        let (descendent_name, ancestor_path) =
            FieldPath::split_basename(path.as_ref().trim_end_matches('_'));

        let children = ancestor_path.children();

        let mut current_mapper: String = ty.to_owned();

        // Get mapper for path
        for c in children {
            let mapper = mappers
                .get(&current_mapper)
                .ok_or(ToqlError::MapperMissing(current_mapper))?;

            if let Some(j) = mapper.joined_mapper(c.as_str()) {
                current_mapper = j.to_string();
            } else if let Some(m) = mapper.merged_mapper(c.as_str()) {
                current_mapper = m.to_string();
            } else {
                return Err(SqlBuilderError::JoinMissing(c.as_str().to_owned()).into());
            }
        }
        let mapper = mappers
            .get(&current_mapper)
            .ok_or(ToqlError::MapperMissing(current_mapper))?;

        // Triage field
        // Join use as normal field (this will insert keys of the join)
       /*  if mapper.joined_mapper(descendent_name).is_some() {
            fields
                .entry(ancestor_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(descendent_name.to_string());
          /*   fields
                .entry(path.as_ref().trim_end_matches('_').to_string())
                .or_insert_with(HashSet::new)
                .insert("*".to_string()); */
        } */
        // Merged field
        if mapper.merged_mapper(descendent_name).is_some() {
            merges
                .entry(ancestor_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(descendent_name.to_string());
        }
        // Joins and normal field
        else {
            fields
                .entry(ancestor_path.to_string())
                .or_insert_with(HashSet::new)
                .insert(descendent_name.to_string());
        }
    }
    Ok(())
}
