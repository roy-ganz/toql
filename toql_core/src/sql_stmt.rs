 
    use chrono::NaiveDateTime;
    use chrono::NaiveDate;
    use chrono::NaiveTime;


    pub type SqlStmt = (String, Vec<SqlArg>);

        #[derive(Clone, Debug)]
        pub enum SqlArg {
            U64(u64),
            I64(i64),
            F64(f64),
            Str(String),
            Bool(bool),
            Null()
        }

        impl ToString for SqlArg {

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
        }



        macro_rules! from_unsigned {
       ($($utype:ty),+) => {
        $(
             impl From<$utype> for SqlArg {
                fn from(t: $utype) -> Self {
                    SqlArg::U64(t.into())
                }
        })+
        };
    }
        macro_rules! from_signed {
        ($($stype:ty),+) => {
            $(
                impl From<$stype> for SqlArg {
                fn from(t: $stype) -> Self {
                    SqlArg::I64(t.into())
                }
            }
            )+
        };
        }
       
       from_unsigned!(u8, u16, u32, u64);
       from_signed!(i8, i16, i32, i64);
       
        impl From<f32> for SqlArg {
            fn from(t: f32) -> Self {
                SqlArg::F64(t.into())
            }
        }

        impl From<f64> for SqlArg {
            fn from(t: f64) -> Self {
                SqlArg::F64(t)
            }
        }
        
        impl From<bool> for SqlArg {
            fn from(t: bool) -> Self {
                SqlArg::Bool(t)
            }
        }
         impl From<&str> for SqlArg {
            fn from(t: &str) -> Self {
                SqlArg::Str(t.to_owned())
            }
        }
          impl From<&String> for SqlArg {
            fn from(t: &String) -> Self {
                SqlArg::Str(t.to_owned())
            }
        }
        impl From<String> for SqlArg {
            fn from(t: String) -> Self {
                SqlArg::Str(t)
            }
        }

        impl From<NaiveDateTime> for SqlArg {
            fn from(t: NaiveDateTime) -> Self {
                SqlArg::Str(t.to_string())
            }
        }
        impl From<NaiveDate> for SqlArg {
            fn from(t: NaiveDate) -> Self {
                SqlArg::Str(t.to_string())
            }
        }
        impl From<NaiveTime> for SqlArg {
            fn from(t: NaiveTime) -> Self {
                SqlArg::Str(t.to_string())
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
        