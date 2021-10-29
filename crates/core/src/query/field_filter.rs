/// The filter operation on a field. You use this when creating a [FieldHandler](../table_mapper/trait.FieldHandler.html)
/// to provide custom functions through the _Fn_ filter or implement a alternative mapping to SQL.
use crate::sql_arg::SqlArg;

#[derive(Clone, Debug)]
pub enum FieldFilter {
    Eq(SqlArg),
    Eqn,
    Ne(SqlArg),
    Nen,
    Gt(SqlArg),
    Ge(SqlArg),
    Lt(SqlArg),
    Le(SqlArg),
    Lk(SqlArg),
    Bw(SqlArg, SqlArg), // Lower, upper limit
    In(Vec<SqlArg>),
    Out(Vec<SqlArg>),
    Fn(String, Vec<SqlArg>), // Function name, args
}

impl ToString for FieldFilter {
    fn to_string(&self) -> String {
        let mut s = String::new();
        match self {
            FieldFilter::Eq(ref arg) => {
                s.push_str("EQ ");
                s.push_str(&arg.to_string());
            }
            FieldFilter::Eqn => {
                s.push_str("EQN");
            }
            FieldFilter::Ne(ref arg) => {
                s.push_str("NE ");
                s.push_str(&arg.to_string());
            }
            FieldFilter::Nen => {
                s.push_str("NEN");
            }
            FieldFilter::Gt(ref arg) => {
                s.push_str("GT ");
                s.push_str(&arg.to_string());
            }
            FieldFilter::Ge(ref arg) => {
                s.push_str("GE ");
                s.push_str(&arg.to_string());
            }
            FieldFilter::Lt(ref arg) => {
                s.push_str("LT ");
                s.push_str(&arg.to_string());
            }
            FieldFilter::Le(ref arg) => {
                s.push_str("LE ");
                s.push_str(&arg.to_string());
            }
            FieldFilter::Lk(ref arg) => {
                s.push_str("LK ");
                s.push_str(&arg.to_string());
            }
            FieldFilter::Bw(ref lower, ref upper) => {
                s.push_str("BW ");
                s.push_str(&lower.to_string());
                s.push(' ');
                s.push_str(&upper.to_string());
            }
            FieldFilter::In(ref args) => {
                s.push_str("IN ");
                s.push_str(
                    &args
                        .iter()
                        .map(|a| a.to_sql_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            }
            FieldFilter::Out(ref args) => {
                s.push_str("OUT ");
                s.push_str(
                    &args
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            }
            FieldFilter::Fn(ref name, ref args) => {
                s.push_str("FN ");
                s.push_str(name);
                s.push(' ');
                s.push_str(
                    &args
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            }
        }
        s
    }
}


#[cfg(test)]
mod test {
    use super::FieldFilter;
    use crate::sql_arg::SqlArg;
    
    #[test]
    fn to_string() {
        assert_eq!(FieldFilter::Eqn.to_string(), "EQN");
        assert_eq!(FieldFilter::Ne(SqlArg::U64(1)).to_string(), "NE 1");
        assert_eq!(FieldFilter::Nen.to_string(), "NEN");
        assert_eq!(FieldFilter::Gt(SqlArg::U64(1)).to_string(), "GT 1");
        assert_eq!(FieldFilter::Ge(SqlArg::U64(1)).to_string(), "GE 1");
        assert_eq!(FieldFilter::Lt(SqlArg::U64(1)).to_string(), "LT 1");
        assert_eq!(FieldFilter::Le(SqlArg::U64(1)).to_string(), "LE 1");
        assert_eq!(FieldFilter::Lk(SqlArg::Str("%ABC%".to_string())).to_string(), "LK '%ABC%'");
        assert_eq!(FieldFilter::Bw(SqlArg::U64(1), SqlArg::U64(10)).to_string(), "BW 1 10");
        assert_eq!(FieldFilter::In(vec![SqlArg::U64(1), SqlArg::U64(10)]).to_string(), "IN 1 10");
        assert_eq!(FieldFilter::Out(vec![SqlArg::U64(1), SqlArg::U64(10)]).to_string(), "OUT 1 10");
        assert_eq!(FieldFilter::Fn("SC".to_string(), vec![SqlArg::U64(1), SqlArg::U64(10)]).to_string(), "FN SC 1 10");
    }
}