
# Predicates
All normal filtering is based on fields, [see here](../5-query-language/4-filter.md). 
However sometimes you may have a completely different filter criteria, that cannot be mapped on fields. 

An example is the MySql full text search. Let's do it:

```rust
#[derive(Toql)]
#[toql(predicate(name="search", 
		sql="MATCH (..firstname, ..lastname) AGAINST (?  IN BOOLEAN MODE)"))]

#[toql(predicate(name="street", 
		sql="EXISTS( SELECT 1 FROM User u JOIN Address a ON (u.address_id = a.id) \
		 	WHERE a.street = ? AND u.id = ..id)"))]
struct User {

 #[toql(key)]
 id: u64,

 firstname: String,
 lastname: String,
}
```

With the two predicates above you can seach for users that have a certain name with `@search 'peter'` 
and retrieve all users from a certain street with `@street 'elmstreet'`.

The question marks in the predicate are replaced by the arguments provided. 
If there is only one argument, it can also be used to build an `ON` predicate in a join. See [on param](4-joins.md).

## Custom predicate handler

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

pub fn my_handler() -> impl PredicateHandler {
    MyPredicateHandler {}
}
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

```


Use it in a Toql query with `@names 'Peter' 'Sandy' 'Bob'`


## Reference

The full predicate syntax is
`predicate(name="..", sql="..", handler="..", on_aux_param="..", count_filter=true|false)` 
where 
- _name_ is the name of the predicate. It can be called with this name `@name ..`. 
  If a predicate is defined on a joined struct, that predicate can be called with a path
  `@path_name ..`. See [predicate](5-query-language/5-predictes.md) for more details.
- _sql_ is a raw QL expression. Use `?` to insert a predicate param in the SQL, 
  `..` for the table alias, `<aux_param>` for aux params
- _handler_ allows a custom predicate handler (build SQL with a function). 
  Provide a function name without parenthesis that return a struct that implement `toql::prelude::PredicateHandler`
- *on_aux_param* set the name of an aux_param that can be used when building custom joins. See [example](4-join.md).
  Can only be used when the predicate takes exactly one argument.
- *count_filter* determines if a predicate used in Toql query should also be included in [count queries](3-api/2-load.md). 
  Default is `false`