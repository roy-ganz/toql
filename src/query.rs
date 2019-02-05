


use std::slice::Iter;

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
#[derive(Clone, Debug)]
pub struct QueryIter<'a> {
         query: &'a Query,
         pos: usize,
}

#[derive(Clone, Debug)]
 pub struct TraverseTop<'a> {
         iter: QueryIter<'a>,
         subpaths: Vec<&'a str>
}

impl<'a> Iterator for TraverseTop<'a>{
    type Item= &'a QueryToken;

            fn next(&mut self) -> Option<Self::Item> {
                let token = self.iter.next();
                match token {
                     Some(QueryToken::Field (field)) => {
                         
                         let f = field.clone();
                         f.name="hallo".to_owned();
                         let t= QueryToken::Field(f);
                         Some(&t)
                          
                     }
                     _ => token

                }
               /*  if let  Some(QueryToken::Field (Field{ name, ..})) = token {
                // Obviously, there isn't any more data to read so let's stop here.
                 for s in &self.subpaths {
                     if name.starts_with(s) {
                         Some()name = f.name.trim_start_matches(s).trim_start_matches("_").to_string();
                         break;
                     } else {
                         token
                     }
                 }
                None
                } else {
                 None
                } */
            }
}


impl<'a> QueryIter<'a> {

    pub fn traverse_top(self, subpaths: Vec<&'a str>) -> TraverseTop<'a> {
        
         TraverseTop {subpaths: subpaths, iter:self}
     }

}


impl<'a> Iterator for QueryIter<'a>{
    type Item= &'a QueryToken;

            fn next(&mut self) -> Option<Self::Item> {
            if self.pos >= self.query.tokens.len() {
                // Obviously, there isn't any more data to read so let's stop here.
                None
                } else {
                    // We increment the position of our iterator.
                    self.pos += 1;
                    // We return the current value pointed by our iterator.
                self.query.tokens.get(self.pos - 1)
                }
            }
}




/* impl<'a> Iterator for TraverseTop<'a> {
    type Item= &'a QueryToken;

        fn next(&mut self) -> Option<Self::Item> {
          if self.query_iter.pos >= self.query.tokens.len() {
            // Obviously, there isn't any more data to read so let's stop here.
            None
            } else {
                // We increment the position of our iterator.
                self.pos += 1;
                // We return the current value pointed by our iterator.
               self.query.tokens.get(self.pos - 1)
            }
        }
} */


impl Query {
    pub fn new () -> Self {
        Query { tokens: vec![]}
    }

   pub fn iter(&self) -> QueryIter {
       QueryIter {query:self, pos:0} 
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
   