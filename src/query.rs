




#[derive(Clone, Debug)]   
pub enum Concatenation {
    And,
    Or
}

#[derive(Clone, Debug)]     
pub struct Field {
    pub concatenation : Concatenation,
    pub name: String, 
    pub hidden: bool, 
    pub order: Option<FieldOrder>, 
    pub filter: Option<FieldFilter>,
    pub aggregation: bool,
    }


#[derive(Clone, Debug)]     
pub enum FieldFilter {
    Eq (String),
    Ne (String),
    Gt (String),
    Ge (String),
    Lt (String),
    Le (String),
    Lk (String),
    In (Vec<String>),
    Out (Vec<String>),
    Other ( String)
}
#[derive(Clone, Debug)]
pub enum FieldOrder {
    Asc (u8),
    Desc (u8)
}
#[derive(Clone, Debug)]        
pub enum GroupSearch {
    Ma(String)
}

#[derive(Clone, Debug)]     
pub enum QueryToken {
    LeftBracket (Concatenation),
    RightBracket,
    Wildcard,
    Field (Field)
}

#[derive(Clone, Debug)]
pub struct Query {
    pub tokens: Vec<QueryToken>
}



impl Query {
    pub fn new () -> Self {
        Query { tokens: vec![]}
    }
    

    pub fn remove(mut self, subpath: &str) -> Self {
        self.tokens.retain(|t|  match t {
       QueryToken::Field (Field{ name, ..})=> !name.starts_with(subpath),
       _ => true
        
        });
        self
    }

     pub fn retain_top(mut self, subpath: Vec<&str>) ->  Self {

      self.tokens.retain(|t|  match t {
       QueryToken::Field (Field{ name, ..})=> {
         let x = name.find('_').is_none() ||  subpath.iter().any(|&x| name.starts_with(x));
         x
       },
       _ => true
        
        });

      /*   // Remove prefix
        for e in self.tokens.iter_mut() {
             if let QueryToken::Field(f) = e {
                 for s in &subpath {
                     if f.name.starts_with(s) {
                         f.name = f.name.trim_start_matches(s).trim_start_matches("_").to_string();
                         break;
                     }
                 }
            }
        } */
        self
       
     }

    

     

     pub fn traverse(mut self, subpath: &str) ->  Self {

      self.tokens.retain(|t|  match t {
       QueryToken::Field (Field{ name, ..})=> name.starts_with(subpath),
       _ => true
        
        });

        // Remove prefix
        for e in self.tokens.iter_mut() {
             if let QueryToken::Field(f) = e {
                f.name = f.name.trim_start_matches(subpath).trim_start_matches("_").to_string();
            }
        }
        self
       
     }

   

}
   