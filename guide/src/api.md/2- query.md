# Queries 
Some functions require a query as argument. 
To build a query use the `query!` macro. It features 

## Examples
```rust
 let q1 = query!(Type, "$, user_id eq 5");
 let q2 = query!(Type, "$, user_id eq ?", 5);
 let q3 = query!(Type, "+fullname$, {}", q2);

 ``