
use mysql::Value;
use toql_core::sql_stmt::SqlArg;

pub fn values_from(args :Vec<SqlArg>) -> Vec<Value> {

  args.into_iter().map(|a| value_from(a)).collect::<Vec<_>>()

}
    
  pub fn value_from(arg: SqlArg) -> Value{
                match arg {
                    SqlArg::U64(d) => Value::from(d),
                    SqlArg::I64(d) => Value::from(d),
                    SqlArg::F64(d) => Value::from(d),
                    SqlArg::Str(d) => Value::from(d),
                    SqlArg::Bool(d) => Value::from(d),
                    SqlArg::Null() => Value::NULL,
                }
}