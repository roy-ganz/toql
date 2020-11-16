

use crate::{tree::{tree_insert::TreeInsert, tree_identity::TreeIdentity}, query::field_path::{FieldPath, Descendents}, sql::Sql, sql_mapper::{SqlMapper, mapped::Mapped}, error::ToqlError, alias_translator::AliasTranslator, alias::AliasFormat, sql_expr::resolver::Resolver, sql_builder::sql_builder_error::SqlBuilderError};
use std::{collections::HashMap, borrow::BorrowMut};
use crate::{paths::Paths, parameter::ParameterMap, fields::Fields};

use crate::error::Result;
use std::collections::HashSet;


    pub fn set_tree_identity<T, Q>(first_id: u64, entities: &mut[Q], path: &mut Descendents) -> Result<()>
    where
        T:  TreeIdentity,
        Q: BorrowMut<T>,
    {
            use crate::tree::tree_identity::IdentityAction;
            use crate::sql_arg::SqlArg;

          if <T as TreeIdentity>::auto_id() {
            let mut id: u64 =  first_id; 
            let home_path = FieldPath::default();
            let mut descendents= home_path.descendents();
            for  e in entities.iter_mut() {
                {
                let e_mut = e.borrow_mut();
                <T as TreeIdentity>::set_id( e_mut, &mut descendents,  IdentityAction::Set(vec![SqlArg::U64(id)]))?;
                }
                id += 1;
            }
        }
        Ok(())
    }
    pub fn build_insert_sql<T, Q>( mappers: &HashMap<String, SqlMapper>, 
        alias_format: AliasFormat, aux_params: &ParameterMap, entities: &[Q], 
            path: &FieldPath, modifier: &str, extra: &str) 
            -> Result<Option<Sql>>
    where
        T: Mapped + TreeInsert,
        Q: BorrowMut<T>,
    {
        use crate::sql_expr::SqlExpr;

        let ty = <T as Mapped>::type_name();
      
        let mut values_expr = SqlExpr::new();
        let mut d = path.descendents();
        let columns_expr = <T as TreeInsert>::columns(&mut d)?;
        for e in entities {
             let mut d = path.descendents();
            <T as TreeInsert>::values(e.borrow(), &mut d, &mut values_expr)?;
        }
        if values_expr.is_empty() {
            return Ok(None);
        }
        
        let mut mapper = 
            mappers
            .get(&ty)
            .ok_or(ToqlError::MapperMissing(ty.to_owned()))?;
        let mut alias_translator = AliasTranslator::new(alias_format);

        // Walk down mappers
        for d in path.descendents(){
            let mapper_name = 
                 mapper.joined_mapper(d.as_str()).or( mapper.merged_mapper(d.as_str()));
           let mapper_name =  mapper_name.ok_or(ToqlError::MapperMissing(d.as_str().to_owned()))?;
            mapper = 
                mappers
                .get(&mapper_name)
                .ok_or(ToqlError::MapperMissing(mapper_name.to_owned()))?;
        }
      

        let resolver = Resolver::new()
            .with_aux_params(&aux_params)
            .with_self_alias(&mapper.canonical_table_alias);
        let columns_sql = resolver
            .to_sql(&columns_expr, &mut alias_translator)
            .map_err(ToqlError::from)?;
        let values_sql = resolver
            .to_sql(&values_expr, &mut alias_translator)
            .map_err(ToqlError::from)?;

        let mut insert_stmt = String::from("INSERT INTO ");
        insert_stmt.push_str(&mapper.table_name);
            insert_stmt.push_str(" ");
        insert_stmt.push_str(&columns_sql.0);
        insert_stmt.push_str(" VALUES ");
        insert_stmt.push_str(&values_sql.0);

        insert_stmt.pop(); // Remove ', '
        insert_stmt.pop();

        Ok(Some(Sql(insert_stmt, values_sql.1)))
    }
    pub fn split_basename( fields: &[&str], path_basenames:  &mut HashMap<String, HashSet<String>>, paths: &mut Vec<String>) {
        for f in fields {
            let (base, path )  = FieldPath::split_basename(f);
            let p = path.unwrap_or_default();
            if path_basenames.get(p.as_str()).is_none() {
                path_basenames.insert(p.as_str().to_string(), HashSet::new());
            }
            path_basenames.get_mut(p.as_str()).unwrap().insert(base.to_string());
           
            paths.push(p.to_string());
           
        }

    }



    pub fn build_insert_tree<T, S: AsRef<str>>(
        mappers: &HashMap<String, SqlMapper>,
        paths: &[S],
        joins: &mut Vec<HashSet<String>>,
        merges: &mut HashSet<String>,
    ) -> Result<()>
    where
        T: Mapped,
    {
        let ty = <T as Mapped>::type_name();
        for path in paths {
            let field_path = FieldPath::from(path.as_ref());
            let steps = field_path.step();
            let children = field_path.children();
            let mut level = 0;
            let mut mapper =
                mappers
                .get(&ty)
                .ok_or(ToqlError::MapperMissing(ty.to_owned()))?;

            for (d, c) in steps.zip(children) {
                if let Some(j) = mapper.joined_mapper(c.as_str()) {
                    if joins.len() <= level {
                        joins.push(HashSet::new());
                    }

                    // let rel = d.relative_path(home_path.as_str()).unwrap_or(FieldPath::default());
                    joins.get_mut(level).unwrap().insert(d.as_str().to_string());
                    level += 1;
                    mapper = mappers
                        .get(&j)
                        .ok_or(ToqlError::MapperMissing(j.to_owned()))?;
                } else if let Some(m) = mapper.merged_mapper(c.as_str()) {
                    level = 0;
                    merges.insert(d.as_str().to_string());
                    mapper = mappers
                        .get(&m)
                        .ok_or(ToqlError::MapperMissing(m.to_owned()))?;
                } else {
                    return Err(SqlBuilderError::JoinMissing(c.as_str().to_owned()).into());
                }
            }
        }
        Ok(())
    }