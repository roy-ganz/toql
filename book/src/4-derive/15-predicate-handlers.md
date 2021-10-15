
# Predicate handlers

It's also possible to write an own predicate handler. 
Let's write a handler that concatenates all argument passed to the predicate and puts those arguments into the SQL predicate.


```rust
#[derive(Toql)]
#[toql(predicate(name="names", 
				 sql="EXISTS (Select 1 FROM User u JOIN Todo t ON (u.id = t.user_id) \
				AND u.name IN <args>)", handler="my_handler"))]
struct Todo {

 #[toql(key)]
 id: u64,

 what: String,
}


use toql::prelude::{PredicateHandler, SqlExpr, SqlArg, ParameterMap, SqlBuilderError};

pub(crate) struct MyPredicateHandler;
impl PredicateHandler for MyPredicateHandler {
    fn build_predicate(
        &self,
        predicate: SqlExpr, 		// SQL from predicate
        predicate_args: &[SqlArg],	// Arguments from the query
        aux_params: &ParameterMap,	// Aux params
    ) -> Result<Option<SqlExpr>, SqlBuilderError>  // Return None if no filtering should take place
	{
		if predicate_args.is_empty() {
            return Err(SqlBuilderError::FilterInvalid(
                "at least 1 argument expected".to_string(),
            ));
        }
        let mut args_expr = SqlExpr::new();
        predicate_args.iter().for_each(|a| { 
            args_expr.push_arg(a.to_owned());
            args_expr.push_literal(", ");
        });
        args_expr.pop(); // remove trailing ', '

        let mut replace = HashMap::new();
        replace.insert("args".to_string(), args_expr);
        let predicate = Resolver::replace_aux_params(predicate, &replace); // Replace  aux params with SQL expressions
        
        Ok(Some(predicate))
    }
}

// Getter function
pub fn my_handler() -> impl PredicateHandler {
    MyPredicateHandler {}
}

```


Use it in a Toql query with `@names 'Peter' 'Sandy' 'Bob'`