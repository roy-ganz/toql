use super::SqlArg;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

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
                        None => SqlArg::Null
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
                        None => SqlArg::Null
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
                        None => SqlArg::Null
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
                        None => SqlArg::Null
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
            Some(v) => SqlArg::Bool(v.to_owned()),
            None => SqlArg::Null,
        }
    }
}

impl From<&str> for SqlArg {
    fn from(t: &str) -> Self {
        SqlArg::Str(t.to_owned())
    }
}

impl From<&&str> for SqlArg {
    fn from(t: &&str) -> Self {
        SqlArg::Str((*t).to_owned())
    }
}

impl From<&SqlArg> for SqlArg {
    fn from(t: &SqlArg) -> Self {
        t.to_owned()
    }
}

impl<T> From<Option<T>> for SqlArg
where
    T: Into<SqlArg>,
{
    fn from(t: Option<T>) -> Self {
        match t {
            Some(t) => t.into(),
            None => SqlArg::Null,
        }
    }
}
