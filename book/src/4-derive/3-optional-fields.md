
# Optional fields
A [Toql query](../5-query-language/1-introduction.md) can select individual fields from a struct. However fields must be `Option` for this, otherwise they will always be selected in the SQL statement, regardless of the query. 

### Example:

```rust
  #[derive(Toql)]
 	struct User {

		#[toql(key)]
		id: u32,			// Always selected in SQL (keys must not be optional)

		age: u8,			// Always selected in SQL

		firstname: Option<String>	// Selectable field of non nullable column
		middlename: Option<Option<String>>// Selectable field of nullable column

		#[toql(preselect)]	
		lastname: Option<String>	// Always selected in SQL, nullable column
  }
```

You noticed it: Nullable columns that should always be selected must be annotated with `preselect`. 



## Preselection and joins
Preselected fields on _joined_ structs are selected, if 
- A join itself is preselected
- or at least one field on that join is selected

#### Preselection example 
```rust
  #[derive(Toql)]
	struct User {

		#[toql(key)]
		id: u32,

		#[toql(join())]
		native_language: Language,	// Preselected inner join

		#[toql(join())]
		foreign_language: Option<Option<Language>>,
	}

	#[derive(Toql)]
	struct Language {
			#[toql(key)]
			id: u32,

			code: Option<String>
	}
```

Above `id` in `User` is always selected, because it's _not_ `Option`. 
As `native_language` is a preselected (inner) join, its `id` will also always be selected.
But on the contrary `foreign_language` is a selectable (left) join. `id` will only be selected if the query requests any other field from that join. For example with `foreignLanguage_code`.

## Preselection on parent paths
One more thing: If a field on a related struct is selected, all preselected fields from the path line will be selected too.

Lets assume we have a _user_ that has an _address_, which contains _country_ information. 

The query 

```toql 
address_country_code
``` 

would therefore
- select `code` from the table `Country`
- select all preseleted fields from table `Country`
- select all preseleted fields from table `Address`
- select all preseleted fields from table `User`

