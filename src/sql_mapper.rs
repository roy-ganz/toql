
use crate::query::FieldFilter;
use std::collections::BTreeSet;
use std::collections::HashMap;

pub (crate) enum FilterType {
    Where,
    Having,
    None,
}

pub struct SqlTarget {
   //  pub (crate) selected: bool,             // Target is selected
     //pub (crate) alias: String,              // Calculated alias for field
     //pub (crate) used: bool,                 // Target is either selected or filtered
     //pub (crate) joined_target: bool,        // Target must be joined
     pub (crate) options: MapperOptions,     // Options
     pub (crate) filter_type: FilterType,    // Filter on where or having clause
     pub (crate) handler: Box<FieldHandler>, // Handler to create clauses
     pub (crate) subfields: bool,                // Target name has subfields separated by underscore
}



pub struct SqlField {
    name: String,
}
pub struct MapperOptions {
      pub (crate) always_selected: bool, // Always select this field, regardless of query fields
      pub (crate) alias: bool,           // This field must not be aliased
      pub (crate) count_query: bool,     // Use this field also in count query
      pub (crate) ignore_wildcard: bool, // Ignore field for wildcard selection
      pub (crate) roles: BTreeSet<String>,    // Only for use by these roles
}

// OPT use references
impl MapperOptions {
    pub fn new() -> Self {
        MapperOptions {
            always_selected: false,
            alias: true,
            count_query: false,
            ignore_wildcard: false,
            roles: BTreeSet::new(),
        }
    }
    pub fn select_always(mut self, always_selected: bool) -> Self {
        self.always_selected = always_selected;
        self
    }
    pub fn alias(mut self, alias: bool) -> Self {
        self.alias = alias;
        self
    }
    pub fn count_query(mut self, count_query: bool) -> Self {
        self.count_query = count_query;
        self
    }
    pub fn ignore_wildcard(mut self, ignore_wildcard: bool) -> Self {
        self.ignore_wildcard = ignore_wildcard;
        self
    }
    pub fn restrict_roles(mut self, roles: BTreeSet<String>) -> Self {
        self.roles = roles;
        self
    }
}



trait MapperFilter {
    fn build(field: crate::query::QueryToken) -> String;
}
pub trait FieldHandler {
    fn validate_query(&self) -> bool {
        true
    }
    fn build_select(&self) -> Option<String> {
        None
    }
    fn build_filter(&self, filter: &FieldFilter) -> Option<String> {
        None
    }
    fn build_param(&self, filter: &FieldFilter) -> Option<String> {
        None
    }
    fn build_join(&self) -> Option<String> {
        None
    }
}

// Test trait to develop derive
pub trait ToqlQuery {
    fn hello_macro();
}

impl FieldHandler for SqlField {
    fn build_select(&self) -> Option<String> {
        Some(format!("{}", self.name))
    }
    fn build_param(&self, filter: &FieldFilter) -> Option<String> {
        match filter {
            FieldFilter::Eq(criteria) => Some(criteria.clone()),
            FieldFilter::Ne(criteria) => Some(criteria.clone()),
            FieldFilter::Ge(criteria) => Some(criteria.clone()),
            FieldFilter::Gt(criteria) => Some(criteria.clone()),
            FieldFilter::Le(criteria) => Some(criteria.clone()),
            FieldFilter::Lt(criteria) => Some(criteria.clone()),
            FieldFilter::In(_) => None,
            FieldFilter::Out(_) => None,
            FieldFilter::Lk(criteria) => Some(criteria.clone()),
            FieldFilter::Other(_) => None,
            _ => None,
        }
    }
    fn build_filter(&self, filter: &FieldFilter) -> Option<String> {
        match filter {
            FieldFilter::Eq(_) => Some(format!("{} = ?",  self.name)),
            FieldFilter::Ne(_) => Some(format!("{} <> ?", self.name)),
            FieldFilter::Ge(_) => Some(format!("{} >= ?", self.name)),
            FieldFilter::Gt(_) => Some(format!("{} > ?",  self.name)),
            FieldFilter::Le(_) => Some(format!("{} <= ?", self.name)),
            FieldFilter::Lt(_) => Some(format!("{} < ?",  self.name)),
            FieldFilter::In(values) => {
                Some(format!("{} IN ({})", self.name, values.join(",")))
            }
            FieldFilter::Out(values) => Some(format!(
                "{} NOT IN ({})",
                self.name,
                values.join(",")
            )),
            FieldFilter::Lk(_) => Some(format!("{} LIKE ?", self.name)),
            FieldFilter::Other(_) => None,
            _ => None,
        }
    }
    /*  fn build_join(&self) ->Option<String> {
        self.join_clause.clone()
    } */
}

pub struct SqlMapper {
  //  alias: String,
    pub (crate) field_order: Vec<String>,
    pub (crate) fields: HashMap<String, SqlTarget>,
    pub (crate) joins: HashMap<String, Join>,
}

pub struct Join {
     pub (crate) join_clause: String,
 //   alias: String,
   // joined: bool
}




pub trait Mappable {
    fn map(mapper: &mut SqlMapper, prefix: &str);
}

impl SqlMapper {
    pub fn new() -> SqlMapper {
        SqlMapper {
          //  dirty: false,
          //  alias: "".to_string(),
            joins: HashMap::new(),
            fields: HashMap::new(),
            field_order: Vec::new(),
        }
    }
    pub fn map<T: Mappable>() -> SqlMapper {
        let mut m = Self::new();
        T::map(&mut m, "");
        m
    }
    pub fn map_with_prefix<T: Mappable>(prefix: &str) -> SqlMapper {
        let mut m = Self::new();
        T::map(&mut m, prefix);
        m
    }

    pub fn map_handler<'a> (
        &'a mut self,
        toql_field: &str,
        handler: Box<FieldHandler>,
        options: MapperOptions,
    ) ->  &'a mut Self{
        // Check if toql field has underscore
       // let x = toql_field.split('_').rev().next().is_some();

        let t = SqlTarget {
            options: options,
            filter_type: FilterType::Where, // filter on where clause
        //    selected: false,
        //    used: false,
         //   joined: false,
            //joined_target: x,
            subfields: toql_field.find('_').is_some(),
            handler: handler,
        };
        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }

    pub fn alter_field(&mut self, toql_field: &str, sql_field: &str, options: MapperOptions) ->  & mut Self {
        let sql_target = self
            .fields
            .get_mut(toql_field)
            .expect("Field  is not mapped.");

        let f = SqlField {
            name: sql_field.to_string(),
        };
        sql_target.options = options;
        sql_target.handler = Box::new(f);
        self
    }

    pub fn map_field<'a>(
        &'a mut self,
        toql_field: &str,
        sql_field: &str,
        options: MapperOptions,
    ) -> &'a mut Self {
        let f = SqlField {
            name: sql_field.to_string(),
        };

      
        let t = SqlTarget {
            options: options,
            filter_type: FilterType::Where, // filter on where clause
         //   selected: false,
         //   alias: String::from(""),
         //   used: false,
            subfields: toql_field.find('_').is_some(),
           // joined: false,
            handler: Box::new(f),
        };

        self.field_order.push(toql_field.to_string());
        self.fields.insert(toql_field.to_string(), t);
        self
    }

   

    pub fn join<'a>(
        &'a mut self,
        toql_field: &str,
        join_clause: &str,
    ) -> &'a mut Self {
        self.joins.insert(
            toql_field.to_string(),
            Join {
               join_clause: join_clause.to_string(),
          
            },
        );

        // Find targets that use join and set join field
        


        self
    }

   
}
