
# Optional fields
Each field in a Toql query can individually be selected. However fields must be `Option<>` for this, otherwise they will always be selected in the SQL statement, regardless of the query.



```rust
 	struct User {
		id: u32,				// Always selected in SQL
		name: Option<String>			// Optional field
		middlename: Option<Option<String>>	// Optional field of nullable column
		#[toql(select_always)]
		middlename: Option<String>  		// Nullable column, always selected in SQL
  }
```
