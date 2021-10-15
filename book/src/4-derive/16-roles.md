
# Roles
It's possible to restrict access to fields and structs with boolean role expressions.

```rust

#[derive(Toql)] {
#[toql(roles(insert="poweruser", delete="poweruser"))
struct Book

	#[toql(key)]
	id : u64

	#[toql(roles(load="superuser;poweruser", update="poweruser"))]
	rating: u64
}
```
The role expressions are similar to the Toql query syntax:
 - OR is expressed with ;
 - AND is expressed with ,
 - NOT is expressed with !
 - brackets are allowed

An valid role expression would be `(teacher;student), !lazy` meaning `A teacher OR student AND NOT lazy`.

Roles are provided with the context:
```
let mut r = HashSet::new();
r.insert("teacher");
let context = ContextBuilder::new()
		.with_roles(r)
		.build();
```
See [here](3-api/1-introduction.md) for how to get a backend.

Notice that roles can restrict access to columns but not to rows. 
For row access control, check out the [chapter](../6-appendix/4-row-access-control.md) in the appendix.
