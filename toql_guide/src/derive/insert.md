# Insert
Toql provides two sets of insert functions to handle duplicates properly .

Use  `insert_one` / `insert_many` if inserts should not fail. This is typically the case with simple database tables that contain an auto value as primary key.

If primary keys are not auto generated, then you must provide a strategy to handle duplicates with `insert_dup_one` / `insert_dup_many`. 

Consider the example of an associated tables that is used to store a collection of other tables. How should the insert behave if the collection already contains the table? Possible stategies are:

- Ignore the insert operation with `DuplicateStrategy::Ignore`
- Updated the already existing database record with `DuplicateStrategy::Update`
- Throw an error `DuplicateStrategy::Fail`

####Example:

```
#[derive(Toql)]
struct User {
    #[toql(skip_mut)] // auto value
    id: u64,
    languages : Vec<UserLanguage>
}
#[derive(Toql, Clone)] // Clone needed for merge
struct UserLanguage {
    user_id: u64,
    language: Language
}

fn main () {
    use toql::
    let toql =  --snip--
    let user =  --snip--
    let user_language = --snip--

    toql.insert(user)?; // Should not fail
    toql.insert_dup(user_language, DuplicateStrategy::Fail)?; // New user should speak this language, must not fail
    toql.insert_dup(user_language, DuplicateStrategy::Ignore)?; // New user already speaks this language (Record exists), ignore insert
}

```


### Default values
For *selectable* fields in a struct that are `None` Toql will insert the default value for the corresponding table column.
If you have not defined a default value in your database you must ensure that the field in the struct cannot be `None`. 
This can be done through prior validation.


##### Example

```rust
struct User {
    #[toql(skip_mut)]
	id: u64                     // No insert
	username: String,			// Value
	realname: Option<String>, 		// Default or value
	address: Option<Option<<String>>, 	// Nullable column: Default, value or NULL
	#[toql(preselect)]
	info: Option<String> 		// Nullable column: Value or NULL
}
```


 



