# Field handlers

It's possible to write an own field handler. Do it, because

 - You want to build an SQL expression with a function.
 - You want to support a database function through [`FN`](../5-query-language/4-filter.md)
 - You want to abuild a filter condition with a function

 
 ## Filter on fields

 Let's support a length function `LLE` , so that we can filter on maximum  word length

 ```rust

use toql::prelude::{BasicFieldFilter, FieldHandler, SqlExpr, SqlBuilderError, FieldFilter};

 struct LengthFieldHandler{
     // The default field handler gives us default filters, such as `eq`, `ne`, ...
	 default_handler: BasicFieldHandler, 
};

impl FieldHandler for PermissionFieldHandler
{
	 fn build_filter(
        &self,
        select: SqlExpr,        // Our column or SQL expression
        filter: &FieldFilter,   // The filter called with this field
        aux_params: &ParameterMap, // All aux params available
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        match filter {
			// Support our custom LL filter that maps to the MySQL FIND_IN_FIELD function
            FieldFilter::Fn(name, args) => match name.as_str() {
                "LLE" => {
                     if args.len() != 1 {
                        return Err(SqlBuilderError::FilterInvalid( "filter `FN LLE` expects exactly 1 argument".to_string()));
                    }
                    Ok(Some(sql_expr!("LENGTH ({}) <= ?", select, args[0])))
                }
                name @ _ => Err(SqlBuilderError::FilterInvalid(name.to_string())),
            },
            _ => self.default_handler.build_filter(select, filter, aux_params),
        }
    }

}

// Getter method for mapper
pub fn length_field_handler() -> impl FieldHandler {
    LengthFieldHandler:{
		 default_handler: BasicFieldHandler::new(), 
	}
}
```

Now we can map our filter with

```rust
#[toql(handler="length_field_handler")]
name: String
```
and use it in a query with

```toql
*, name FN LLE 5
```

For a bigger example, check out our [permission handler](6-appendix/4-row-access-control.md).




