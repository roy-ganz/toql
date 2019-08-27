
# Optional fields
Each field in a Toql query can individually be selected. However fields must be `Option<>` for this, otherwise they will always be selected in the SQL statement, regardless of the query. 

Fields that should always be selected, but are `Option<>` must be annotated with `preselect`. 

## Example:

```rust
  #[derive(Toql)]
 	struct User {
		id: u32,				// Always selected in SQL
		firstname: Option<String>			// Optional field of non nullable column
		middlename: Option<Option<String>>	// Optional field of nullable column

		#[toql(preselect)]	
		lastname: Option<String>			// Optional field non nullable column, always selected in SQL
  }
```
