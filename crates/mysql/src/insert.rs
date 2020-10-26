

use toql_core::{tree::{tree_insert::TreeInsert, tree_identity::TreeIdentity}, query::field_path::{FieldPath, Descendents}, sql::Sql, sql_mapper::{SqlMapper, mapped::Mapped}, error::ToqlError, alias_translator::AliasTranslator, alias::AliasFormat, sql_expr::resolver::Resolver, sql_builder::sql_builder_error::SqlBuilderError};
use std::{collections::HashMap, borrow::BorrowMut};
use toql_core::{paths::Paths, parameter::ParameterMap};
use crate::sql_arg::values_from;
use crate::error::Result;
use std::collections::HashSet;
use mysql::{prelude::GenericConnection};
use crate::MySql;

 /* /// Insert one struct.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id.
    pub fn insert_many<C, T, Q>(  mysql: &mut MySql<C>, paths: Paths, mut entities: &mut [Q]) ->Result<u64>
    where
      C: GenericConnection,
        T: TreeInsert + Mapped + TreeIdentity,
        Q: BorrowMut<T>,
    {
      

       
        //let sql = <T as InsertSql>::insert_one_sql(entity, &self.roles, "", "")?;
        //execute_insert_sql(sql, self.conn)

        // Build up execution tree
        // Path `a_b_merge1_c_d_merge2_e` becomes
        // [0] = [a, c, e]
        // [1] = [a_b, c_d]
        // [m] = [merge1, merge2]
        // Then execution order is [1], [0], [m]

        // TODO should be possible to impl with &str
        let mut joins: Vec<HashSet<String>> = Vec::new();
        let mut merges: Vec<String> = Vec::new();

        mysql.build_insert_tree::<T>(&paths.0, &mut joins, &mut merges)?;

        let home_path = FieldPath::default();
        let mut home_descendents = home_path.descendents();

        // Execute root
        
        let Sql(insert_stmt, insert_values) = {
            let aux_params = [mysql.aux_params()];
            let aux_params = ParameterMap::new(&aux_params);
            build_insert_sql::<T, _>(&mysql.registry().mappers, mysql.alias_format(), &aux_params, entities, &mut home_descendents, "", "")
        }?;

        let params = values_from(insert_values);
        let mut stmt = {mysql.conn().prepare(&insert_stmt)}?;
        let res = stmt.execute(params)?;

        if res.affected_rows() == 0 {
            return Ok(0);
        }

        let mut home_descendents = home_path.descendents();
        set_tree_identity( res.last_insert_id(), &mut entities, &mut home_descendents)?;

      

      

        // Execute joins
        for l in (0..joins.len()).rev() {
            for p in joins.get(l).unwrap() {
                dbg!(p);
                let path = FieldPath::from(&p);
                let mut descendents = path.descendents();
                let Sql(insert_stmt, insert_values) = 
                {
                    let aux_params = [mysql.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    build_insert_sql::<T, _>(&mysql.registry().mappers, mysql.alias_format(), &aux_params, entities, &mut descendents, "", "")
                }?;
                
                // Execute
                let params = values_from(insert_values);
                let mut stmt = mysql.conn().prepare(&insert_stmt)?;
                let res = stmt.execute(params)?;

                // set keys
                let path = FieldPath::from(&p);
               let mut descendents = path.descendents();
               set_tree_identity( res.last_insert_id(), &mut entities, &mut descendents)?;
            }
        }

        // Execute merges
        for p in merges {
                dbg!(p);
                let path = FieldPath::from(&p);
                let mut descendents = path.descendents();
                let Sql(insert_stmt, insert_values) = {
                    let aux_params = [mysql.aux_params()];
                    let aux_params = ParameterMap::new(&aux_params);
                    build_insert_sql::<T, _>(&mysql.registry().mappers, mysql.alias_format(), &aux_params, entities, &mut descendents, "", "")
                    }?;
                
                // Execute
                let res = stmt.execute(params)?;

                // Merges must not contain auto value as identity, skip set_tree_identity
        }

        Ok(0)
    }

    /// Insert a collection of structs.
    ///
    /// Skip fields in struct that are auto generated with `#[toql(skip_inup)]`.
    /// Returns the last generated id
    /* pub fn insert_many<T, Q>(&mut self, entities: &[Q]) -> Result<u64>
    where
        T: InsertSql,
        Q: Borrow<T>,
    {
        let sql = <T as InsertSql>::insert_many_sql(&entities, &self.roles, "", "")?;

        Ok(if let Some(sql) = sql {
            execute_insert_sql(sql, self.conn)?
        } else {
            0
        })
    } */
 */

    pub(crate) fn set_tree_identity<T, Q>(first_id: u64, entities: &mut[Q], path: &mut Descendents) -> toql_core::error::Result<()>
    where
        T:  TreeIdentity,
        Q: BorrowMut<T>,
    {
          if <T as toql_core::tree::tree_identity::TreeIdentity>::auto_id() {
            let mut id: u64 =  first_id; 
            let home_path = FieldPath::default();
            let mut descendents= home_path.descendents();
            for  e in entities.iter_mut() {
                {
                let e_mut = e.borrow_mut();
                <T as toql_core::tree::tree_identity::TreeIdentity>::set_id( e_mut, &mut descendents,  id)?;
                }
                id += 1;
            }
        }
        Ok(())
    }
    pub(crate) fn build_insert_sql<T, Q>( mappers: &HashMap<String, SqlMapper>, alias_format: AliasFormat, aux_params: &ParameterMap, entities: &[Q], path: &FieldPath, modifier: &str, extra: &str) -> toql_core::error::Result<Sql>
    where
        T: Mapped + TreeInsert,
        Q: BorrowMut<T>,
    {
        use toql_core::sql_expr::SqlExpr;

        let ty = <T as Mapped>::type_name();
      
        let mut values_expr = SqlExpr::new();
        let mut d = path.descendents();
        let columns_expr = <T as TreeInsert>::columns(&mut d)?;
        for e in entities {
             let mut d = path.descendents();
            <T as TreeInsert>::values(e.borrow(), &mut d, &mut values_expr)?;
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

        Ok(Sql(insert_stmt, values_sql.1))
    }
    pub fn build_insert_tree<T>(
        mappers: &HashMap<String, SqlMapper>,
        paths: &[&str],
        joins: &mut Vec<HashSet<String>>,
        merges: &mut Vec<String>,
    ) -> Result<()>
    where
        T: Mapped,
    {
        let ty = <T as Mapped>::type_name();
        for path in paths {
            let field_path = FieldPath::from(path);
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
                    merges.push(d.as_str().to_string());
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