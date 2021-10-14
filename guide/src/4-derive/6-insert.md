# Insert
When you insert a struct, all fields and joins the are provided with the path list will be inserted. Merges are inserted seperately.
Check [here](../3-api/4-insert.md) for details.

### Default values
For *selectable* fields in a struct that are `None` Toql will insert the default value for the corresponding table column.
If you have not defined a default value in your database you must ensure that the field in the struct cannot be `None`. 
This can be done through prior validation.


#### Insert Behaviour Example

```rust
#[derive(Toql)]
struct User {
	#[toql(key)]
	id: u64                     // Keys are never inserted
	
	username: String,		// Value
	realname: Option<String>,	// Default or value
	address: Option<Option<<String>>,// Nullable column: Default, value or NULL

	#[toql(preselect)]
	info: Option<String> 	// Nullable column: Value or NULL

	#[toql(join)]
	address1: Option<Address> 	// Selectable inner Join: Foreign key is inserted or default

	#[toql(join)]
	address2: Option<Option<Address>>// Selectable left join: Default, value or NULL

	#[toql(join())]
	address3: Address 		// Inner Join: Foreign key or default

	#[toql(join(), preselect)]
	address4: Option<Address>>	// Selectable inner join: Foreign key or default

	#[toql(merge())]
	phones1: Vec<Phone>>		// No change on table 'User'

	#[toql(merge())]
	phones2: Option<Vec<Phone>>> // No change on table 'User'
}
```

When the path list requires to insert a dependency too, 
left joins and optional merges will only be inserted, if they contains a value.

 



