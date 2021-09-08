
use super::SqlArg;
use crate::sql_builder::select_stream::Select;
use crate::from_row::FromRow;
use crate::error::ToqlError;
use  crate::deserialize::error::DeserializeError;

 impl<R, E> FromRow<R, E> for SqlArg 
 where E: From<ToqlError>, u64: FromRow<R,E>,i64: FromRow<R,E>, f64: FromRow<R,E>, bool: FromRow<R,E>, String: FromRow<R,E>
 {
               fn forward<'a, I>( iter: &mut I) -> Result<usize,E>
                where
                        I: Iterator<Item = &'a Select>{
                    if  iter.next().ok_or(
                            ToqlError::DeserializeError(
                                DeserializeError::StreamEnd)
                    )?.is_selected() {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }
                // Return None, if unselected or column is null
                fn from_row<'a, I>(
                        row: &R,
                        i: &mut usize,
                        iter: &mut I,
                    ) -> std::result::Result<Option<SqlArg>, E>
                    where
                        I: Iterator<Item = &'a Select> + Clone,
                    {

                        if iter
                       // .inspect(|v| println!("Select is {:?}", v))
                         . next().ok_or(
                            ToqlError::DeserializeError(DeserializeError::StreamEnd))?.is_selected() {
                            // First Option is None, if Index is out of bounds, second Option is Nullable column
                            let val  = u64::from_row(row, i, iter);
                            if !val.is_err() {
                                
                                match val.or(Err(ToqlError::DeserializeError(DeserializeError::StreamEnd)))? {
                                    Some(v) => return Ok(Some(SqlArg::U64(v))),
                                    None => return Ok(None)
                                }
                            }
                            let val = String::from_row(row, i, iter);
                            if !val.is_err() {
                                match val.or(Err(ToqlError::DeserializeError(DeserializeError::StreamEnd)))? {
                                    Some(v) => return Ok(Some(SqlArg::Str(v))),
                                    None => return Ok(None)
                                }
                            }
                            let val = i64::from_row(row, i, iter);
                            if !val.is_err() {
                             let val  = val.or(Err(ToqlError::DeserializeError(DeserializeError::StreamEnd)))?;
                                match val {
                                    Some(v) => return Ok(Some(SqlArg::I64(v))),
                                    None => return Ok(None)
                                }
                            }
                            let val = f64::from_row(row, i, iter);
                            if !val.is_err() {
                             let val  = val.or(Err(ToqlError::DeserializeError(DeserializeError::StreamEnd)))?;
                                match val {
                                    Some(v) => return Ok(Some(SqlArg::F64(v))),
                                    None => return Ok(None)
                                }
                            }
                            let val = bool::from_row(row, i, iter);
                            match val {
                                Ok(va) => {
                                    match va {
                                        Some(v) => { *i += 1; Ok(Some(SqlArg::Bool(v)))},
                                        None => Ok(None)
                                    }
                                }
                                Err(e) => {
                                     Err(e)
                                }
                            }
                        } else {
                            Ok(None)
                        }
                    }
                }