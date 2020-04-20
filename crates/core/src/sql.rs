 
    use chrono::NaiveDateTime;
    use chrono::NaiveDate;
    use chrono::NaiveTime;
    

    pub type Sql = (String, Vec<SqlArg>);


    pub fn unsafe_sql(sql: &str, params: &[SqlArg]) -> String {
        // Replace every ? with param
        let mut params = params.iter();

        sql.chars().map(|c| {if c == '?' { 
            match params.next() {
                Some(p) => p.to_sql_string(),
                None => String::from("?")
            }
        } else {
            c.to_string()
        }
        }).collect::<String>()



    }

        #[derive(Clone, Debug)]
        pub enum SqlArg {
            U64(u64),
            I64(i64),
            F64(f64),
            Str(String),
            Bool(bool),
            Null()
        }


    impl SqlArg {

        pub fn to_sql_string(&self) -> String {
             match self {
                SqlArg::U64(t) => t.to_string(),
                SqlArg::I64(t) => t.to_string(),
                SqlArg::F64(t) => t.to_string(),
                SqlArg::Str(t) => format!("'{}'", t.replace("'", "''")),
                SqlArg::Bool(t) => t.to_string(),
                SqlArg::Null() => "NULL".to_string(),
                }

        }

        pub fn get_i64(&self) -> Option<i64> {
            if let Self::I64(v) = self {
                Some(v.to_owned())
            } else {
                None
            }
        }
         pub fn get_f64(&self) -> Option<f64> {
            if let Self::F64(v) = self {
                Some(v.to_owned())
            } else {
                None
            }
        }
        pub fn get_bool(&self) -> Option<bool> {
            if let Self::Bool(v) = self {
                Some(v.to_owned())
            } else {
                None
            }
        }
        pub fn get_u64(&self) -> Option<u64> {
            if let Self::U64(v) = self {
                Some(v.to_owned())
            } else {
                None
            }
        }
         pub fn get_str(&self) -> Option<&str> {
            if let Self::Str(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub fn is_null(&self) -> bool {
            if let Self::Null() = self {
                true
            } else {
                false
            }
        }
           pub fn cmp_str(&self, other: &str) -> bool {
            if let Self::Str(v) = self {
                v == other
            } else {
                false
            }
        }
    }
        


        impl ToString for SqlArg {

            fn to_string(&self) -> String {

                match self {
                SqlArg::U64(t) => t.to_string(),
                SqlArg::I64(t) => t.to_string(),
                SqlArg::F64(t) => t.to_string(),
                SqlArg::Str(t) => t.to_string(),
                SqlArg::Bool(t) => t.to_string(),
                SqlArg::Null() => "NULL".to_string(),
                }
            }
        }

       /*   impl Display for SqlArg {

            fn to_string(&self) -> String {

                match self {
                SqlArg::U64(t) => t.to_string(),
                SqlArg::I64(t) => t.to_string(),
                SqlArg::F64(t) => t.to_string(),
                SqlArg::Str(t) => format!("'{}'", t),
                SqlArg::Bool(t) => t.to_string(),
                SqlArg::Null() => "NULL".to_string(),
                }
            }
        } */


  macro_rules! from_float {
       ($($type:ty),+) => {
        $(
             impl From<$type> for SqlArg {
                fn from(t: $type) -> Self {
                    SqlArg::F64(t.into())
                }
             }
            impl From<&$type> for SqlArg {
                fn from(t: &$type) -> Self {
                    SqlArg::F64(t.to_owned().into())
                }
        }
         impl From<&Option<$type>> for SqlArg {
                fn from(t: &Option<$type>) -> Self {
                    match t {
                        Some(v) =>  SqlArg::F64(v.to_owned().into()),
                        None => SqlArg::Null()
                    }
                    
                }
             }
        )+
        };

      
    }
      

        macro_rules! from_unsigned {
       ($($type:ty),+) => {
        $(
             impl From<$type> for SqlArg {
                fn from(t: $type) -> Self {
                    SqlArg::U64(t.into())
                }
             }
              impl From<&$type> for SqlArg {
                fn from(t: &$type) -> Self {
                    SqlArg::U64(t.to_owned().into())
                }
            }
             impl From<&Option<$type>> for SqlArg {
                fn from(t: &Option<$type>) -> Self {
                    match t {
                        Some(v) =>  SqlArg::U64(v.to_owned().into()),
                        None => SqlArg::Null()
                    }
                    
                }
             }
        
        )+
        };
    }
        macro_rules! from_signed {
        ($($type:ty),+) => {
            $(
                impl From<$type> for SqlArg {
                fn from(t: $type) -> Self {
                    SqlArg::I64(t.into())
                }
            }
             impl From<&$type> for SqlArg {
                fn from(t: &$type) -> Self {
                    SqlArg::I64(t.to_owned().into())
                }
             }
             impl From<&Option<$type>> for SqlArg {
                fn from(t: &Option<$type>) -> Self {
                    match t {
                        Some(v) =>  SqlArg::I64(v.to_owned().into()),
                        None => SqlArg::Null()
                    }
                    
                }
             }
            )+
        };
        }
        macro_rules! from_string {
        ($($type:ty),+) => {
            $(
                impl From<$type> for SqlArg {
                fn from(t: $type) -> Self {
                    SqlArg::Str(t.to_string())
                }
            }
             impl From<&$type> for SqlArg {
                fn from(t: &$type) -> Self {
                    SqlArg::Str(t.to_string())
                }
             }
             impl From<&Option<$type>> for SqlArg {
                fn from(t: &Option<$type>) -> Self {
                    match t {
                        Some(v) =>  SqlArg::Str(v.to_string()),
                        None => SqlArg::Null()
                    }
                    
                }
             }
            )+
        };
        }
       
      
       
       from_unsigned!(u8, u16, u32, u64);
       from_signed!(i8, i16, i32, i64);
       from_float!(f32, f64);
       from_string!(String, NaiveDateTime, NaiveDate, NaiveTime);
       

       
        
        impl From<bool> for SqlArg {
            fn from(t: bool) -> Self {
                SqlArg::Bool(t)
            }
        }
        impl From<&bool> for SqlArg {
            fn from(t: &bool) -> Self {
                SqlArg::Bool(t.to_owned())
            }
        }
         impl From<&Option<bool>> for SqlArg {
            fn from(t: &Option<bool>) -> Self {
                match t {
                    Some(v) =>  SqlArg::Bool(v.to_owned()),
                    None => SqlArg::Null()
                }
               
            }
        }


         impl From<&str> for SqlArg {
            fn from(t: &str) -> Self {
                SqlArg::Str(t.to_owned())
            }
        }
    

        impl<T> From<Option<T>> for SqlArg 
         where T: Into<SqlArg>
         {
            fn from(t: Option<T>) -> Self {
                match t {
                    Some(t) => t.into(),
                    None => SqlArg::Null()
                }
            }
        } 


       

         

         
        
       

         
        
        