

pub trait FilterArg<T> {
    fn to_sql(self) -> String;

}

impl FilterArg<&str> for &str {
    fn to_sql(self) -> String {
        let mut s = String::from("'");
        // TODO escape for swl
        s.push_str(self);
        s.push('\'');
        s

    }

}

impl FilterArg<u8> for u8 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<u16> for u16 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<u32> for u32 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<u64> for u64 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<u128> for u128 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<i8> for i8 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<i16> for i16 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<i32> for i32 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<i64> for i64 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<i128> for i128 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}
impl FilterArg<bool> for bool {
    fn to_sql(self) -> String {
        String::from(if self == true {"1"} else {"0"})
    }
}
impl FilterArg<f32> for f32 {
    fn to_sql(self) -> String {
       self.to_string()
    }
}
impl FilterArg<f64> for f64 {
    fn to_sql(self) -> String {
        self.to_string()
    }
}



#[derive(Clone, Debug)]
pub enum Concatenation {
    And,
    Or,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub concatenation: Concatenation,
    pub name: String,
    pub hidden: bool,
    pub order: Option<FieldOrder>,
    pub filter: Option<FieldFilter>,
    pub aggregation: bool,
}


impl Field {
    pub fn from<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        let name = name.into();
        #[cfg(debug)]
        {
            // Ensure name does not end with wildcard
            if name.ends_with("*") {
                panic!("Fieldname {:?} must not end with wildcard!", name);
            }
        }

        Field {
            concatenation: Concatenation::And,
            name: name.into(),
            hidden: false,
            order: None,
            filter: None,
            aggregation: false,
        }
    }
  
    pub fn hide(mut self) -> Self {
        self.hidden = true;
        self
    }
    pub fn aggregate(mut self) -> Self {
        self.aggregation = true;
        self
    }
    pub fn asc(mut self, order:u8) -> Self 
    {
        self.order = Some(FieldOrder::Asc(order));
        self
    }
     pub fn desc(mut self, order:u8) -> Self 
    {
        self.order = Some(FieldOrder::Desc(order));
        self
    }
    pub fn eq<T>(mut self, criteria: impl FilterArg<T>) -> Self 
    {
        self.filter = Some(FieldFilter::Eq(criteria.to_sql()));
        self
    }
    pub fn eqn(mut self) -> Self  
    {
        self.filter = Some(FieldFilter::Eqn);
        self
    }
    pub fn ne<T>(mut self, criteria:  impl FilterArg<T>) -> Self  
    {
        self.filter = Some(FieldFilter::Ne(criteria.to_sql()));
        self
    }
    pub fn nen(mut self) -> Self  
    {
        self.filter = Some(FieldFilter::Nen);
        self
    }
    pub fn gt<T>(mut self, criteria:  impl FilterArg<T>) -> Self  
    {
        self.filter = Some(FieldFilter::Gt(criteria.to_sql()));
        self
    }
    pub fn ge<T>(mut self, criteria:  impl FilterArg<T>) -> Self  
    {
        self.filter = Some(FieldFilter::Ge(criteria.to_sql()));
        self
    }
     pub fn lt<T>(mut self, criteria:  impl FilterArg<T>) -> Self   
    {
        self.filter = Some(FieldFilter::Lt(criteria.to_sql()));
        self
    }
    pub fn le<T>(mut self, criteria:  impl FilterArg<T>) -> Self   
    {
        self.filter = Some(FieldFilter::Le(criteria.to_sql()));
        self
    }
    pub fn bw<T>(mut self, lower:  impl FilterArg<T>, upper:  impl FilterArg<T>) -> Self 
    {
        self.filter = Some(FieldFilter::Bw(lower.to_sql(), upper.to_sql()));
        self
    }
    pub fn lk<T>(mut self, criteria:  impl FilterArg<T>) -> Self  
    {
        self.filter = Some(FieldFilter::Lk(criteria.to_sql()));
        self
    }
     pub fn re<T>(mut self, criteria:  impl FilterArg<T>) -> Self  
    {
        self.filter = Some(FieldFilter::Re(criteria.to_sql()));
        self
    }
     pub fn sc<T>(mut self, criteria:  impl FilterArg<T>) -> Self  
    {
        self.filter = Some(FieldFilter::Sc(criteria.to_sql()));
        self
    }
    pub fn ins<T>(mut self, criteria:  Vec<impl FilterArg<T>>) -> Self  
    {
        self.filter = Some(FieldFilter::In(criteria.into_iter().map(|c| c.to_sql()).collect() ));
        self
    }
    pub fn out<T>(mut self, criteria:  Vec<impl FilterArg<T>>) -> Self  
    {
        self.filter = Some(FieldFilter::Out(criteria.into_iter().map(|c| c.to_sql()).collect() ));
        self
    }
    pub fn fnc<U, T>(mut self, name: U, args:   Vec<impl FilterArg<T>>) -> Self   where U: Into<String>,
    {
        self.filter = Some(FieldFilter::Fn(name.into(), args.into_iter().map(|c| c.to_sql()).collect()));
        self
    }
}


impl ToString for Field {
    fn to_string(&self) -> String {
        let mut s = String::new();
               match self.order {
                   Some(FieldOrder::Asc(o)) => {s.push('+'); s.push_str(&o.to_string());}
                   Some(FieldOrder::Desc(o)) => {s.push('-'); s.push_str(&o.to_string());}
                   None => {}
               };
                if self.hidden {
                    s.push('.');
                }
                 s.push_str(&self.name);

                 if self.filter.is_some() {
                    if self.aggregation {
                        s.push_str(" !");
                    } else {
                        s.push(' ');
                    }
                 }
                match self.filter  {
                    None => s.push_str(""),
                    Some(FieldFilter::Eq(ref arg)) => { s.push_str("EQ ");s.push_str(arg); },
                    Some(FieldFilter::Eqn) => { s.push_str("EQN");},
                    Some(FieldFilter::Ne(ref arg)) => { s.push_str("NE ");s.push_str(arg); },
                    Some(FieldFilter::Nen) => { s.push_str("NEN");},
                    Some(FieldFilter::Gt(ref arg)) => { s.push_str("GT ");s.push_str(arg); },
                    Some(FieldFilter::Ge(ref arg)) => { s.push_str("GE ");s.push_str(arg); },
                    Some(FieldFilter::Lt(ref arg)) => { s.push_str("LT ");s.push_str(arg); },
                    Some(FieldFilter::Le(ref arg)) => { s.push_str("LE ");s.push_str(arg); },
                    Some(FieldFilter::Lk(ref arg)) => { s.push_str("LK ");s.push_str(arg); },
                    Some(FieldFilter::Re(ref arg)) => { s.push_str("RE ");s.push_str(arg); },
                    Some(FieldFilter::Sc(ref arg)) => { s.push_str("SC ");s.push_str(arg); },
                    Some(FieldFilter::Bw(ref lower, ref upper)) => { s.push_str("BW ");s.push_str(lower);s.push(' ');s.push_str(upper); },
                    Some(FieldFilter::In(ref args)) => { s.push_str("IN "); s.push_str(&args.join(" "))},
                    Some(FieldFilter::Out(ref args)) => { s.push_str("OUT ");s.push_str(&args.join(" ")) },
                    Some(FieldFilter::Fn(ref name, ref args)) => { s.push_str("FN ");s.push_str(name); s.push(' ');s.push_str(&args.join(" "))},
                }
              s
    }
}


impl From<&str> for Field {
    fn from(s: &str) -> Field {
        Field::from(s)
    }
}



impl Into<QueryToken> for Field {
    fn into(self) -> QueryToken {
        QueryToken::Field(self)
    }
}


#[derive(Clone, Debug)]
pub enum FieldFilter {
    Eq(String),
    Eqn,
    Ne(String),
    Nen,
    Gt(String),
    Ge(String),
    Lt(String),
    Le(String),
    Lk(String),
    Bw(String, String), // Lower upper
    In(Vec<String>),
    Out(Vec<String>),
    Re(String),
    Sc(String),
    Fn(String, Vec<String>), // Name Args
}
#[derive(Clone, Debug)]
pub enum FieldOrder {
    Asc(u8),
    Desc(u8),
}


#[derive(Clone, Debug)]
pub enum QueryToken {
    LeftBracket(Concatenation),
    RightBracket,
    Wildcard(Concatenation),
    Field(Field),
}

impl From<&str> for QueryToken {
    fn from(s: &str) -> QueryToken {
        if s.ends_with("*") {
            QueryToken::Wildcard(Concatenation::And)
        } else {
            QueryToken::Field(Field::from(s))
        }
    }
}

impl ToString for QueryToken {

    fn to_string(&self)-> String {
              
        let s = match self {
            QueryToken::RightBracket =>  String::from(")"),
            QueryToken::LeftBracket ( c) => {
               match c {
                   Concatenation::And => String::from("("),
                   Concatenation::Or => String::from("("),
               }
            },
          QueryToken::Field (field /*Field {concatenation, name, hidden, order, filter, aggregation}*/)
          => {
              field.to_string()
          },
            QueryToken::Wildcard(_) => String::from("*"),
        };
        s

    }

}


#[derive(Clone, Debug)]
pub struct Query {
    pub tokens: Vec<QueryToken>,
}

impl Query {
    pub fn new() -> Self {
        Query { tokens: vec![] }
    }

 pub fn and<T>(&mut self, query: T) ->  &mut Self
    where
        T: Into<Query>,
    {
        self.tokens.append(&mut query.into().tokens);
        self
    }
    pub fn or<T>(&mut self, query: T) -> &mut Self
    where
        T: Into<Query>,
    {
        let mut query = query.into();
        
        // Put parens around both expression because logical or has higher precendence
        // Example:  a AND b OR c != (a AND b) OR c
        // Reason "b OR c" is evaluated first in first expression
        

        let bracket_needed = query.tokens.len() > 1;
        if self.tokens.len() > 1 {
            self.tokens.insert(0,QueryToken::LeftBracket(Concatenation::And));
            self.tokens.push(QueryToken::RightBracket);
        }
        if bracket_needed {
            self.tokens.push(QueryToken::LeftBracket(Concatenation::Or));
        } 
        // TODO
        // Don't know how to make this work in rust
        // Prevent brackets on single token
         else {
            // make single token concat with or
             if query.tokens.len() == 1 {
                    // TODO make this work with match
                    
                    if let QueryToken::LeftBracket(c) = query.tokens.get_mut(0).unwrap() {
                        *c= Concatenation::Or;
                    }
                     else if let QueryToken::Field(field) = query.tokens.get_mut(0).unwrap() {
                        field.concatenation = Concatenation::Or;
                    }
                    else if let QueryToken::Wildcard(w) = query.tokens.get_mut(0).unwrap() {
                        *w= Concatenation::Or;
                    }

            } 
       } 
        self.tokens.append(&mut query.tokens);
        if bracket_needed {
            self.tokens.push(QueryToken::RightBracket {});
        }

        self
    }

    pub fn prepend<T>(&mut self, query: T) -> &mut Self
    where
        T: Into<Query>,
    {
        // Swap queries for better append performance
        let mut q = query.into();
        q.tokens.append(&mut self.tokens);
        std::mem::swap(&mut q.tokens, &mut self.tokens);
       
        self
    }

    
}



impl ToString for Query {
    fn to_string(&self) -> String {

        fn get_concatenation( c: &Concatenation) -> char {
            match c  {
                Concatenation::And => ',',
                Concatenation::Or => ';'
            }
        }

        let mut s = String::new();
        let mut concatenation_needed = false;
        let mut parens_open = false;
        for token in &self.tokens {
            if concatenation_needed {
                match &token {
                     QueryToken::LeftBracket(concatenation) | QueryToken::Wildcard(concatenation)=> s.push( get_concatenation(concatenation)),
                     QueryToken::Field(field) => s.push( get_concatenation(&field.concatenation)),
                     _ => {}
                }
            }
            s.push_str(&token.to_string());
            match token {
                QueryToken::LeftBracket(..) => {concatenation_needed = false},
                QueryToken::Field(..) => {parens_open= false; concatenation_needed = true;},
                QueryToken::Wildcard(..) => {parens_open= false; concatenation_needed = true},
                _ => {}
            }
            println!("{:?}", s);
        }
        s.trim_start_matches(",");
        s.trim_start_matches(";");
        s
    }
}

impl From<Field> for Query {
    fn from(field: Field) -> Query {
        let mut q = Query::new();
        q.tokens.push(QueryToken::Field(field));
        q
    }
}
impl From<&str> for Query {
    fn from(field: &str) -> Query {
        let mut q = Query::new();

        q.tokens.push(if field.ends_with("*") {
            QueryToken::Wildcard(Concatenation::And)
        } else {
            QueryToken::Field(Field::from(field))
        });
        q
    }
}


