# Update

The update functions from the API will update a field,
- if the field name is in the field list 
- and a selectable field in the struct contains a value.


#### Update Behaviour Example

If we want to update all fields of the struct below with a field list of `*`, the behaviour would be

```rust
#[derive(Toql)]
struct User {
	#[toql(key)]
	id: u64			// Keys are never updated	

	username: String,		// Update
	realname: Option<String>, 	// Updated , if Some
	address: Option<Option<<String>>, // Update NULL or String, if Some

	#[toql(preselect)]
	info: Option<String>, 	//Update NULL or String

	#[toql(join)]
	address1: Option<Address>, // Update foreign_key, if Some 

	#[toql(join)]
	address2: Option<Option<Address>>,//  Update foreign_key or NULL, if Some 

	#[toql(join())]
	address3: Address, 		// Update foreign_key

	#[toql(join(), preselect)]
	address4: Option<Address>>,	// Update foreign_key or NULL

	#[toql(merge())]
	phones1: Vec<Phone>>,	// No effect for *

	#[toql(merge())]
	phones2: Option<Vec<Phone>>> // No effect for *, 
}
```

Notice that foreign keys of joins are included (*User.address1_id, User.address2_id, ..*) with the `*` in the field list.
However merges must be explicitly mentioned. 

To update all fields from `User` and to resize the `Vec` of `phones1` (insert new phones + delete old phones ) the field list would be
` *, phones1`.






