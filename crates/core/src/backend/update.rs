
use std::{borrow::{Borrow, BorrowMut}, collections::{HashSet, HashMap}};
use crate::{alias::AliasFormat, sql_mapper::{mapped::Mapped, SqlMapper}, parameter::ParameterMap, tree::tree_update::TreeUpdate, query::field_path::FieldPath};
use crate::{sql_expr::resolver::Resolver, sql::Sql, alias_translator::AliasTranslator, error::ToqlError, sql_builder::sql_builder_error::SqlBuilderError};
use crate::error::Result;

pub fn build_update_sql<T, Q>( alias_format: AliasFormat, 
  
    entities: &[Q], 
    path: &FieldPath, 
    fields: &HashSet<String>,
    roles: &HashSet<String>,
    modifier: &str, 
    extra: &str) -> Result<Vec<Sql>>
    where
        T: Mapped + TreeUpdate,
        Q: Borrow<T>,
    {
        let mut alias_translator = AliasTranslator::new(alias_format);

        let mut update_sqls = Vec::new();

        let mut exprs = Vec::new();
        for e in entities.iter() {
            let mut descendents = path.descendents();
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
pub fn plan_update_order<T, S: AsRef<str>>(
        mappers: &HashMap<String, SqlMapper>,
        paths: &[S],
        fields: &mut HashMap<String, HashSet<String>>, // paths that refer to fields 
        merges: &mut HashMap<String, HashSet<String>>, // paths that refer to merges
    ) -> Result<()>
    where
        T: Mapped,
    {
        let ty = <T as Mapped>::type_name();
        for path in paths {
            let (field, field_path) = FieldPath::split_basename(path.as_ref().trim_end_matches("_"));
            
            let field_path =  field_path.unwrap_or_default();
            
            let children = field_path.children();
            
          
            let mut current_mapper: String = ty.to_owned();
          

            // Get mapper for path
            for c in children {
                  let mapper =
                mappers
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
            let mapper =
                mappers
                .get(&current_mapper)
                .ok_or(ToqlError::MapperMissing(current_mapper))?;

                // Triage field
                // Join, convert to wildcard
                if  mapper.joined_mapper(field).is_some() {
                   fields.entry(path.as_ref().trim_end_matches("_").to_string())
                   .or_insert( HashSet::new())
                   .insert("*".to_string()) ;
                } 
                // Merged field
                else if mapper.merged_mapper(field).is_some() {
                    merges.entry(field_path.to_string())
                   .or_insert( HashSet::new())
                   .insert(field.to_string());
                } 
                // Normal field
                else {
                   fields.entry(field_path.to_string())
                   .or_insert( HashSet::new())
                   .insert(field.to_string());
                }

          
            
        }
        Ok(())
    }