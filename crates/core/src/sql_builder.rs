//!
//! The SQL Builder turns a [Query](../query/struct.Query.html) with the help of a [SQL Mapper](../sql_mapper/struct.SqlMapper.html)
//! into a [SQL Builder Result](../sql_builder_result/BuildResult.html)
//! The result hold the different parts of an SQL query and can be turned into an SQL query that can be sent to the database.
//!
//! ## Example
//!
//! ``` ignore
//!
//! let  query = Query::wildcard().and(Field::from("foo").eq(5));
//! let mapper::new("Bar b").map_field("foo", "b.foo");
//! let builder_result = QueryBuilder::new().build_query(&mapper, &query);
//! assert_eq!("SELECT b.foo FROM Bar b WHERE b.foo = ?", builder_result.to_sql());
//! assert_eq!(["5"], builder_result.params());
//! ```
//!
//! The SQL Builder can also add joins if needed. Joins must be registered on the SQL Mapper for this.
//!
//! ### Count queries
//! Besides normal queries the SQL Builder can als build count queries.
//!
//! Let's assume you have a grid view with books and the user enters a search term to filter your grid.
//! The normal query will get 50 books, but you will only display 10 books. Toql calls those 50 _the filtered count_.
//! To get the unfilted count, Toql must issue another query with different filter settings. Typically to get
//! the number of all books only that user has access to. Toql calls this _the total count_.
//!
//! ### Paths
//! The SQL Builder can also ignore paths to skip paths in the query that are not mapped in the mapper.
//! This is needed for structs that contain collections, as these collections must be querried with a different mapper.
//!
//! Let's assume a struct *user* had a collection of *phones*.
//! The Toql query may look like:  `username, phones_number`.
//! The SQL Builder needs 2 passes to resolve that query:
//!  - The first pass will query all users with the user mapper and will ignore the path *phones_*.
//!  - The second pass will only build the query for the path *phones_* with the help of the phone mapper.
//!

pub(crate) mod build_context;
pub(crate) mod build_result;
pub(crate) mod sql_builder_error;
pub(crate) mod path_tree;
pub(crate) mod sql_with_placeholders;

/* use crate::sql_builder::eval_query::eval_query;
use crate::sql_builder::construct::build_join_clause;
use crate::sql_builder::construct::combine_aux_params;
use crate::sql_builder::construct::build_count_select_clause;
use crate::sql_builder::construct::build_select_clause;
use crate::sql_builder::construct::build_ordering;
use crate::sql_builder::sql_target_data::SqlTargetData; */


use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::error::{Result,ToqlError};
use crate::query::assert_roles;
use crate::query::concatenation::Concatenation;
use crate::query::field_order::FieldOrder;
use crate::query::Query;
use crate::query::{field_filter::FieldFilter, QueryToken};
use super::sql_builder::build_result::BuildResult;
use super::sql_builder::build_context::BuildContext;
use crate::sql_mapper::join::Join;
use crate::sql_mapper::SqlMapper;
use crate::sql_mapper::field::Field as MapperField;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use crate::query::field::Field;
use heck::MixedCase;

use self::sql_with_placeholders::SqlOrPlaceholder;
use crate::sql_mapper_registry::SqlMapperRegistry;

use crate::sql::{Sql, SqlArg};
use crate::alias::AliasFormat;
use crate::alias_translator::AliasTranslator;
use crate::{parameter::ParameterMap, query::field_path::FieldPath};
//use wildcard_scope::WildcardScope;
use path_tree::PathTree;




/// The Sql builder to build normal queries and count queries.
pub struct SqlBuilder<'a> {

    root_mapper: String,
    sql_mapper_registry: &'a SqlMapperRegistry,
    roles: HashSet<String>,
    aux_params: HashMap<String, SqlArg>, // Aux params used for all queries with this builder instance, contains typically config or auth data
     
    extra_joins: HashSet<String>,          // Use this joins
  //  alias_translator: &'a mut AliasTranslator
      
}


impl<'a> SqlBuilder<'a> {
    /// Create a new SQL Builder
    pub fn new(root_mapper: &'a str, sql_mapper_registry: &'a SqlMapperRegistry) -> Self {
      
       SqlBuilder {
           root_mapper: root_mapper.to_mixed_case(),
           sql_mapper_registry,
           roles :HashSet::new(),
           aux_params: HashMap::new(),
           extra_joins:HashSet::new(),
        }
    }

    /*  pub fn with_alias_translator(mut self, alia: &'a mut AliasTranslator) ->  Self {
        self.alias_translator = alia;
        self
    } */
   /*  pub fn with_alias_format(mut self, alias_format: AliasFormat) ->  Self {
        self.alias_translator = &mut AliasTranslator::new(alias_format);
        self
    } */
   
    pub fn with_roles(mut self, roles: HashSet<String>) ->  Self {
        self.roles = roles;
        self
    }
   
        
    pub fn with_extra_join<T: Into<String>>(mut self, join: T) -> Self {
        self.extra_joins.insert(join.into());
        self
    }

    


    pub fn build_delete_sql<M>(&mut self,  query: &Query<M>, _modified: &str, _extra: &str, format:AliasFormat ) -> Result<Sql> {
                      
        let mut context = BuildContext::new(AliasTranslator::new(format));
        let root_mapper = self.sql_mapper_registry.mappers.get(&self.root_mapper)
                .ok_or(ToqlError::MapperMissing(self.root_mapper.to_owned()))?;
        let alias_table = context.alias_translator.translate(&root_mapper.canonical_table_alias);
        let aliased_table = format!("{} {}", root_mapper.table_name, alias_table);
        let mut result = BuildResult::new(&aliased_table);

        self.build_where_clause(&query, &mut context, &mut result)?;
        self.build_join_clause(&mut context, &mut result)?;
             
        Ok(result.delete_sql())
    }

    pub fn build_select_sql<M>(&mut self,  query: &Query<M>, modified: &str, extra: &str, format:AliasFormat ) -> Result<(Sql,impl Iterator<Item = bool>, HashSet<String>)> {
                      
        let mut context = BuildContext::new(AliasTranslator::new(format));
        let root_mapper = self.sql_mapper_registry.mappers.get(&self.root_mapper)
                .ok_or(ToqlError::MapperMissing(self.root_mapper.to_owned()))?;
        let alias_table = context.alias_translator.translate(&root_mapper.canonical_table_alias);
        let aliased_table = format!("{} {}", root_mapper.table_name, alias_table);
        let mut result = BuildResult::new(&aliased_table);
       
        
        self.build_where_clause(&query, &mut context, &mut result)?;
        self.build_select_clause(&query, &mut context, &mut result)?;
        self.build_join_clause(&mut context, &mut result)?;
       
        Ok((result.select_sql(modified, extra), result.selection_stream.into_iter(), result.unmerged_paths))
    }


    fn mapper_for_path(&self, path: &Option<FieldPath>) -> Result<&SqlMapper> {

        let mut current_mapper = self.sql_mapper_registry
            .get(&self.root_mapper)
            .ok_or(ToqlError::MapperMissing(self.root_mapper.to_string()))?;
             
            if let Some(path) = path {
                for p in path.children() {
                    let join = current_mapper.joins.get(p.as_str()).ok_or(ToqlError::MapperMissing(p.as_str().to_string()))?;
                    current_mapper = self.sql_mapper_registry.get(&join.joined_mapper).ok_or(ToqlError::MapperMissing(self.root_mapper.to_string()))?;
                }
            }
            
        Ok(current_mapper)
    }

    fn build_join_clause(&self, mut build_context: &mut BuildContext, result: &mut BuildResult)-> Result<()> {

            // Build join tree for all selected paths
            // This allows to nest joins properly
            // Eg [user] = [user_address, user_folder]
            // [user_folder] = [ user_folder_owner]
            // [user_folder_owner] =[]
            // [user address] =[]
           
           let mut join_tree = PathTree::new();

            for selectect_path in &build_context.joined_paths {
                let absolute_path= format!("{}_{}", self.root_mapper, selectect_path);
                join_tree.insert(&FieldPath::from(&absolute_path));
            }
            dbg!(&join_tree);
            
            // Build join 
            for r in join_tree.roots() {
              let sql = &self.resolve_join(&FieldPath::from(&r), &join_tree, &mut build_context)?;
              result.join_sql.append(sql);
              result.join_sql.pop_literals(1); // Remove trailing whitespace
            }

            Ok(())
    } 
    fn resolve_join(&self, canonical_path: &FieldPath, join_tree: &PathTree, build_context : &mut BuildContext) -> Result<Sql> {
        
      //  let mapper_name= canonical_path.ancestor().unwrap_or(canonical_path.basename());
      
       let mut join_sql = Sql::new();
      
        for nodes in join_tree.nodes(canonical_path.as_str()){
            for n in nodes {
                let mapper = self.mapper_from_path(canonical_path)?;
                
                let (basename, _) = FieldPath::split_basename(n);
                
                 let join = mapper.join(basename)
                .ok_or(SqlBuilderError::JoinMissing(n.to_string()))?; 

                let p = [&self.aux_params, &join.options.aux_params];
                let aux_params = ParameterMap::new(&p);

                let self_alias = build_context.alias_translator.translate(canonical_path.as_str());
                let other_alias = build_context.alias_translator.translate(n.as_str());
                let sql = join.join_expression.resolve(&self_alias, Some(&other_alias), &aux_params)?;
                join_sql.append(&sql);
               
                let sql = self.resolve_join(&FieldPath::from(n.as_str()), join_tree, build_context)?;
                if !sql.is_empty() {
                    join_sql.push_literal(" (");
                    join_sql.append(&sql);
                    join_sql.push_literal(")");
                }
                               
                join_sql.push_literal(" ON (");

                let on_sql= join.on_expression.resolve(&self_alias, Some(&other_alias), &aux_params)?;
               
               let sql = match &join.options.join_handler {
                    Some(handler) => handler.build_on_predicate(on_sql, &aux_params)?,
                    None => on_sql
                };
                join_sql.append(&sql); 
               
                join_sql.push_literal(") ");
            }
        }
       
        Ok(join_sql)
    }
   
    fn mapper_from_path(&self, canonical_path: &FieldPath) -> Result<&SqlMapper> {
        
            //let path = canonical_path.trim_start_matches(self.root_path);
            let path= canonical_path;
            let mut mapper :Option<&SqlMapper>= None;
            for c in path.children() {
                mapper= Some(self.sql_mapper_registry.get(c.as_str())
                    .ok_or(ToqlError::MapperMissing(c.as_str().to_string()))?);
            }

           mapper.ok_or(ToqlError::MapperMissing("".to_ascii_lowercase()))
        }
      
    fn build_where_clause<M>(&mut self, query: &Query<M>, build_context: &mut BuildContext, result: &mut BuildResult) -> Result<()>{
        let p= [&self.aux_params, &query.aux_params];
        let aux_params = ParameterMap::new(&p);

        for token in &query.tokens{

            match token {
                QueryToken::Field(field) => {

                    // Returns error for inexistant field
                
                    let (basename, path)= FieldPath::split_basename(&field.name);
                    let mapper =  self.mapper_for_path(&path)?;
                    let mapped_field =  mapper.fields.get(basename)
                        .ok_or(SqlBuilderError::FieldMissing(basename.to_string()))?;

                    // Continue if field is not filtered
                    if field.filter.is_none() {
                        continue;
                    }

                    if let Some(role) = self.missing_role( &mapped_field.options.roles/*mapper.load_roles(field)*/) {
                            return Err (SqlBuilderError::RoleRequired(role.to_string()).into());
                    }
                    
                    let alias = build_context.alias_translator.translate(&mapper.canonical_table_alias);
                    let sql = mapped_field.expression.resolve(&alias, None, &aux_params)?;

                    let select_sql = mapped_field
                                        .handler
                                        .build_select(
                                            sql,
                                            &aux_params,
                                        )?
                                        .unwrap_or(Sql("NULL".to_string(), vec![]));

                    // Does filter apply
                    if let Some(sql) = mapped_field.handler
                                            .build_filter(select_sql, field.filter.as_ref().unwrap(), &aux_params)?
                    {
                        result.where_sql.append(&sql);

                        if let (_, Some(path)) = FieldPath::split_basename(&field.name){
                            build_context.joined_paths.insert(path.as_str().to_string());
                        }
                    }
                    
                },
                QueryToken::Predicate(predicate) => {

                },
                _ => {}
            }

        }
        Ok(())

    }

    fn build_select_clause<M>(&mut self, query: &Query<M>, build_context: &mut BuildContext, result: &mut BuildResult) 
    -> Result<()>{

       
        let (selected_fields, selected_paths) = self.selection_from_query(query)?;
        build_context.selected_fields = selected_fields;
        build_context.selected_paths = selected_paths;

       self.resolve_select(&None, query, build_context, result)?;
         result.select_sql = build_context.select_sql.into_sql(&build_context.selected_placeholders);
        if result.select_sql.is_empty() {
           result.select_sql.push_literal("1");
        } else {
           result.select_sql.pop_literals(2); // Remove trailing ,
        }
        
        Ok(())

    }

    fn resolve_select<M>(&self, join_path: &Option<FieldPath>, query: &Query<M>, build_context: &mut BuildContext, result: &mut BuildResult) -> Result<()> {

        use crate::sql_mapper::DeserializeType;
        let mapper = self.mapper_for_path(&join_path)?;
        let p= [&self.aux_params, &query.aux_params];
        let aux_params = ParameterMap::new(&p);
        let canonical_alias = join_path.as_ref().map(|j|j.as_str()).unwrap_or(&self.root_mapper);

      
        let ph_index = build_context.current_placeholder  + 1;
        
       
        let mut any_selected = false;
        

        for deserialization_type in &mapper.deserialize_order {
            match deserialization_type {
                DeserializeType::Field(field) => {
                    if  if let Some(a) = FieldPath::from(field).ancestor() {
                        build_context.selected_paths.contains(a.as_str())
                    } else {false} 
                    || build_context.selected_fields.contains(field) {

                        let field_info = mapper.field(field).ok_or(SqlBuilderError::FieldMissing(field.to_string()))?;
                        let alias = build_context.alias_translator.translate(&canonical_alias);
                        let select_sql = field_info.expression.resolve(&alias, None, &aux_params)?;
                        let select_sql =  field_info.handler.build_select(select_sql, &aux_params)?;

                        if let Some(sql) = select_sql {
                            if field_info.options.preselect {
                                // TODO Insert placeholder
                                build_context.select_sql.push_placeholder(ph_index, sql);
                                
                                result.selection_stream.push(false);
                            } else {
                                build_context.select_sql.push_sql(sql);
                                build_context.select_sql.push_literal(", ");
                                result.selection_stream.push(true);
                                any_selected = true;
                            }
                        } else {
                            result.selection_stream.push(false);
                        }
                     } else {
                        result.selection_stream.push(false);
                    }
                
                }, 
                DeserializeType::Join(join) => {
                    let new_join_path= format!("{}_{}", &canonical_alias, &join);

                    if build_context.selected_paths.contains(join){
                        self.resolve_select(&Some(FieldPath::from(&new_join_path)), query, build_context, result)?;
                    } else {
                        self.resolve_select_none(&Some(FieldPath::from(&new_join_path)), result)?;
                    }
                }, 
                DeserializeType::Merge(merge) => {
                    if build_context.selected_paths.contains(merge) {
                        result.unmerged_paths.insert(merge.to_owned());
                    }
                },  
            }
        }
       
        if any_selected {
            build_context.selected_placeholders.insert(ph_index);
        }
        Ok(())

    }
    fn resolve_select_none(&self, join_path: &Option<FieldPath>,result: &mut BuildResult) -> Result<()> {

        use crate::sql_mapper::DeserializeType;
        let mapper = self.mapper_for_path(&join_path)?;
        let canonical_alias = join_path.as_ref().map(|j|j.as_str()).unwrap_or(&self.root_mapper);

        for deserialization_type in &mapper.deserialize_order {
            match deserialization_type {
                DeserializeType::Field(_) => {
                        result.selection_stream.push(false);
                }, 
                DeserializeType::Join(join) => {
                    let new_join_path= format!("{}_{}", &canonical_alias, &join);
                    self.resolve_select_none(&Some(FieldPath::from(&new_join_path)), result)?;
                }, 
                DeserializeType::Merge(_) => { },  
            }
        }
        Ok(())

    }

    fn selection_from_query<M>(&mut self, query: &Query<M>) -> Result<(HashSet<String>, HashSet<String>)> {

        let mut relative_fields= HashSet::new();
        let mut relative_paths= HashSet::new();
        for token in &query.tokens{

            match token {
                QueryToken::Field(field) => {
                    
                    // TODO validate roles and raise error
                    relative_fields.insert(field.name.to_string());
                    

                      
                    
                },
                QueryToken::Wildcard(wildcard) => {

                    // Todo validate roles and skip
                    relative_paths.insert(wildcard.path.to_string());
                },
               /*  QueryToken::Selection(selection) => {

                }, */
                _ => {}
            }

        }
        Ok((relative_fields, relative_paths))

    } 

    fn root_mapper(&self) -> Result<&SqlMapper> {
        self.sql_mapper_registry
        .get(&self.root_mapper)
        .ok_or(ToqlError::MapperMissing(self.root_mapper.to_string()))
    }
    fn missing_role<'b>(&'b self, roles: &'b HashSet<String>) ->Option<&'b str>{
           let s =  roles.difference(&self.roles).next();
           s.map(|r| r.as_str())
    }

  
}