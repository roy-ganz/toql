
use crate::annot::Pair;
use crate::annot::ToqlField;
use crate::annot::Toql;

use crate::heck::MixedCase;
use crate::heck::SnakeCase;

use syn::{Ident, Visibility};
use proc_macro2::{Span, TokenStream};


pub struct Struct {
  pub rust_struct_ident: Ident,
  pub rust_struct_name: String,
  pub sql_table_name: String,
  pub sql_table_alias: String,
  pub rust_struct_visibility: Visibility  
}

impl Struct {
      pub fn create( toql: &Toql) -> Self {
         let renamed_table = crate::util::rename(&toql.ident.to_string(), &toql.tables);
        Struct {
             rust_struct_ident: toql.ident.clone(),
             rust_struct_name: toql.ident.to_string(),
             sql_table_name: toql.table.clone().unwrap_or(renamed_table),
            sql_table_alias: toql
                .alias
                .clone()
                .unwrap_or(toql.ident.to_string().to_snake_case()), 
            rust_struct_visibility: toql.vis.clone()
        }
      }
}

#[derive(Clone)]
pub enum SqlTarget {
  Column(String),
  Expression(String)

}
// Maybe fillup this
#[derive(Clone)]
pub struct RegularField{
    pub sql_target: SqlTarget,
    pub key: bool,
    pub count_select: bool,
    pub count_filter: bool,
    
}
#[derive(Clone)]
pub struct JoinField{
  
  pub sql_join_table_ident: Ident,
  pub sql_join_table_name: String,
  pub join_alias: String,
  pub columns_map_code: TokenStream,
  pub on_sql: Option<String>,
  pub key: bool,
}
#[derive(Clone)]
pub struct MergeField {
 
 //pub field: Vec<Pair>,
 pub on_sql: Option<String>,
}

#[derive(Clone)]
pub struct Field {
  pub rust_field_ident: Ident,
  pub rust_field_name: String,
  pub rust_type_ident: Ident,
  pub rust_type_name: String,
  pub toql_field_name: String,
  pub number_of_options: u8,
  pub ignore_wildcard: bool,
  pub roles: Vec<String>,
  pub preselect: bool,
  pub kind: FieldKind,
  pub skip_mut: bool,
 
}

#[derive(Clone)]
pub enum FieldKind {
    Regular(  RegularField),
    Join(  JoinField),
    Merge(  MergeField)
}

impl Field {
 pub fn create( field: &ToqlField, toql: &Toql) -> Self {

    
          
      let rust_field_ident= field.ident.as_ref().unwrap().to_owned();
      let  rust_field_name = rust_field_ident.to_string();      
      let rust_type_ident = field.first_non_generic_type().unwrap().to_owned();
      let rust_type_name =  field.first_non_generic_type().unwrap().to_string();
      let toql_field_name = rust_field_name.to_mixed_case();
      let number_of_options = field.number_of_options();

    

     let kind = if field.join.is_some()  {

       let renamed_table = crate::util::rename(field.first_non_generic_type().unwrap().to_string().as_str(), &toql.tables);
       let sql_join_table_name = field.table.as_ref().unwrap_or(&renamed_table).to_owned();
       let columns_translation = field.join.as_ref().unwrap().columns.iter()
                              .map(|column| { 
                              let tc = &column.this; let oc = &column.other;
                              quote!(#oc => #tc, )
                            }).collect::<Vec<_>>();
        let  default_self_column_format = format!("{}_{{}}", field.ident.as_ref().unwrap());
        let columns_map_code = quote!( {
                                        let default_self_column= format!(#default_self_column_format, other_column);
                                        let self_column = match other_column.as_str(){
                                                #(#columns_translation)*
                                                _ => &default_self_column
                                        };
                                        self_column
                                    });                 

       FieldKind::Join(JoinField{
       
        sql_join_table_ident: Ident::new(&sql_join_table_name, Span::call_site()),
        join_alias: field.alias.as_ref().unwrap_or(&sql_join_table_name.to_snake_case()).to_owned(),
        sql_join_table_name,
        columns_map_code,
        on_sql: field.join.as_ref().unwrap().on_sql.clone(),
        key: field.key
       })
    } else  if field.join.is_some() {
      FieldKind::Merge(MergeField {
         on_sql: field.join.as_ref().unwrap().on_sql.to_owned()
      })
 } else  {
    
      FieldKind::Regular(RegularField{
        sql_target: if field.sql.is_some() { SqlTarget::Expression( field.sql.as_ref().unwrap().to_owned())} else { 
             SqlTarget::Column( match &field.column {
                Some(string) => string.to_owned(),
                None => crate::util::rename(&rust_field_name, &toql.columns),
            })
            },
        key: field.key,
        count_select: field.count_select,
        count_filter: field.count_filter,
       
      })
    };

     Field {
          rust_field_ident,
          rust_field_name,
          rust_type_ident,
          rust_type_name,
          toql_field_name,
          number_of_options,
          skip_mut : field.skip_mut,
          ignore_wildcard: field.ignore_wildcard,
          roles: field.role.clone(),
          preselect: field.preselect,
          kind,
    } 

 }

 


}