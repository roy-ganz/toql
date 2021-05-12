use super::error::TryFromSqlArgError;
use super::SqlArg;
use std::convert::TryInto;

macro_rules! try_into_primitives {
       ($(($type:ty, $fnc:ident)),+) => {
        $(
             impl TryInto<$type> for SqlArg {
                 type Error = TryFromSqlArgError;
                 fn try_into(self) -> Result<$type, Self::Error> {
                    let v =  self. $fnc ().ok_or(super::error::TryFromSqlArgError(self.to_owned()))?;
                    <$type as std::convert::TryFrom<_>>::try_from(v)
                        .map_err(|_|TryFromSqlArgError(self.to_owned()))
                }
             }
             impl TryInto<$type> for &SqlArg {
                 type Error = TryFromSqlArgError;
                 fn try_into(self) -> Result<$type, Self::Error> {
                    let v =  self. $fnc().ok_or(TryFromSqlArgError(self.to_owned()))?;
                    <$type as std::convert::TryFrom<_>>::try_from(v)
                        .map_err(|_|TryFromSqlArgError(self.to_owned()))
                }
             }
             impl TryInto<Option<$type>> for SqlArg {
                type Error = TryFromSqlArgError;
                fn try_into(self) -> Result<Option<$type>, Self::Error> {

                if self.is_null() {
                    Ok(None)
                } else {
                    let v =  self. $fnc().ok_or(TryFromSqlArgError(self.to_owned()))?;
                    <$type as std::convert::TryFrom<_>>::try_from(v)
                    .map(Some)
                    .map_err(|_|TryFromSqlArgError(self.to_owned()))
                }
                }
            }
             impl TryInto<Option<$type>> for &SqlArg {
                type Error = TryFromSqlArgError;
                fn try_into(self) -> Result<Option<$type>, Self::Error> {

                if self.is_null() {
                    Ok(None)
                } else {
                    let v =  self. $fnc().ok_or(TryFromSqlArgError(self.to_owned()))?;
                    <$type as std::convert::TryFrom<_>>::try_from(v)
                    .map(Some)
                    .map_err(|_|TryFromSqlArgError(self.to_owned()))
                }
                }
            }
        )+
        };
    }

try_into_primitives!(
    (u64, get_u64),
    (u32, get_u64),
    (u16, get_u64),
    (u8, get_u64),
    (i64, get_i64),
    (i32, get_i64),
    (i16, get_i64),
    (i8, get_i64),
    (f64, get_f64),
    (String, get_str),
    (bool, get_bool)
);
