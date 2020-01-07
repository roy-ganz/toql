# Update
Toql provides functions to 
- update a struct 
- update only the difference of two structs. 


While a diff update seems to be faster, in reality databases will only update differences anyway and only
transmission time is saved, because no redundant values are sent.

## Update behaviour
The update function will update fields only if they contains some value. Look at this struct:

```rust
struct User {
	id: u64
	username: String,			// Always updated
	realname: Option<String>, 		// Updated conditionally
	address: Option<Option<<String>>, 	// Optional nullable column, updated conditionally
	#[toql(preselect)]
	info: Option<String> 		// Nullable column, always updated
}
```


## Diff behaviour
The diff function compares an outdated struct with an updated struct and generate SQL to update only the differences. 
This operation may fail if the updated TODO


