# Join handlers

When doing a join, it's possible do write a custom handler. 
The handler can only build a custom ON predicate. 

Why would you do it? I don't know :) 

In all cases I can think of you get your `ON` predicate cheaper with [on_sql](7-joins.md).

However it's still possible:

```rust

use toql::prelude::{JoinHandler, SqlExpr, SqlArg, ParameterMap, SqlBuilderError};

pub(crate) struct MyJoinHandler;

impl JoinHandler for MyJoinHandler {
    fn build_on_predicate(&self, on_predicate: SqlExpr, aux_params: &ParameterMap,)
     ->Result<Option<SqlExpr>, SqlBuilderError> {
     
      Some(on_expr) 
     }
}

// Getter function
pub fn my_join_handler() -> impl JoinHandler {
    MyJoinHandler {}
}
```

Now map the getter function with

```rust
#[toql(join(), handler="my_join_handler")]
address: Address
```

And any join on `Address` will now call the join handler. 

