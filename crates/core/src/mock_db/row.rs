use crate::deserialize::error::DeserializeError;
use crate::sql_arg::SqlArg;
use crate::error::ToqlError;

use std::convert::TryInto;
use std::fmt;
use crate::sql_builder::select_stream::Select;
/// Newtype for mysql database row
/// This allows to implement the conversion traits for basic data
/// without violating the orphan rule.

#[derive(Debug, Clone)]
pub struct Row(pub Vec<SqlArg>);

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Row(")?;
        let mut first = true;
        for v in &self.0 {
            if !first {
                write!(f, ", ")?;
            } else {
                first = false;
            }
            write!(f, "{}", v.to_string())?;
        }
         write!(f, ")")?;
        Ok(())
    }
}

/// row macro for easier row creation
/// ### Example
/// row!( 1u64, "hallo")
#[macro_export]
macro_rules! row {
        ($($x:expr),+ $(,)?) => {
           Row(vec![$(toql::prelude::SqlArg::from($x)),+])
        }
}


macro_rules! from_row {
        ($($type:ty),+) => {
            $(
               impl crate::from_row::FromRow<Row, ToqlError> for $type {
               fn forward<'a, I>( iter: &mut I) -> Result<usize,ToqlError>
                where
                        I: Iterator<Item = &'a Select>{
                    if  iter.next().ok_or(
                            ToqlError::DeserializeError(DeserializeError::StreamEnd)
                    )?.is_selected() {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }
                // Return None, if unselected or column is null
                fn from_row<'a, I>(
                        row: &Row,
                        i: &mut usize,
                        iter: &mut I,
                    ) -> std::result::Result<Option<$type>, ToqlError>
                    where
                        I: Iterator<Item = &'a Select> + Clone,
                    {
                        if iter
                         . next()
                         .ok_or(ToqlError::DeserializeError(DeserializeError::StreamEnd))?
                         .is_selected() {
                            let sql_arg = row.0.get(*i)
                                .ok_or(ToqlError::ValueMissing(format!("Row is missing value at index {}", &i)))?
                                .to_owned();
                            let v :Option<$type>= sql_arg.try_into()
                                 .map_err( |_| ToqlError::DeserializeError(
                                        DeserializeError::ConversionFailed(
                                            format!("{} at row index {} ", stringify!($type), i),
                                            format!("{:?}",row.0.get(*i)))
                                    ))?;
                            *i += 1;
                            Ok(v)
                        } else {
                            Ok(None)
                        }
                    }
                }

            )+
        };
        }

from_row!(String, u8, u16, u32, u64, i8, i16, i32, i64, f64, bool);
