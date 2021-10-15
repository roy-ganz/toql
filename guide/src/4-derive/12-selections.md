
# Selections
Selections are a list of fields and can be defined on a struct. 
A Toql query can then select the selection instead of all the individual fields. See [here](../5-query-language/6-selections).


```rust
#[derive(Toql)]
#[toql(selection(name="std", fields="*, address_street"))]
#[toql(selection(name="tiny", fields="id, name"))]
struct User {

 #[toql(key)]
 id: u64

 name: String

 #[toql(join())]
 address: Address

}

[derive(Toql)]
struct Address {

 #[toql(key)]
 id: u64

 street: String
```

Notice that selection names with 3 letters or less are internally reserved and my have special meanings. 
They can't be defined except `std` and `cnt`.

The selections above can now be used in a query. Instead of writing `name, address_street` it is possible to write `$std` or event just `$`.
(Because the standart selection is so common, `$` aliases to `$std`).

The `cnt` selection is defined in a similar way.TODO









