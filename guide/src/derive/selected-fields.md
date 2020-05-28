
# Preselection
Fields on a struct that are **not** `Option<>` will always be selected and put into the struct regardless the query string. 
Optional fields however can also be always selected with the `preselect` annotation.

#### Example 1
```rust
  #[derive(Toql)]
	struct User {
		id: u32,
		#[toql(preselect)]
		name: Option<String>,
	}
```

Both `id` and `name` are always selected from the table regardless the query string.


## Preselection and Joins
The behaviour of preselected fields on _joined_ structs depends on the join type:
- On inner joins preselected fields are always selected.
- On left joins (optional relation) preselected fields will only appear in the SQL statement if at least one other field is selected.

#### Example 2
```rust
  #[derive(Toql)]
	struct User {
		id: u32,
		native_language: Language,
		foreign_language: Option<Option<Language>>,
	}
	struct Language {
			id: u32,
			code: Option<String>
	}
```

In the above Example `id` in `User` is always selected, because it's _not_ `Option<>`. 
As `native_language` is an inner join, its `id` will also always be selected.
But on the contrary `foreign_language` is a selectable optional relation (left join), `id` will only be selected automatically if the query requests any additional field from the related struct. For example with `id, foreignLanguage_code`.

## Preselection on parent paths
One more important thing: If a field on a related struct is selected, all preselected fields from the path line will be selected too.

Lets assume we have a user that has an address, which contains country information. The query 

```toql 
address_country_code
``` 

would therefore
- select `code` from the table `Country`
- select all preseleted fields from table `Country`
- select all preseleted fields from table `Address`
- select all preseleted fields from table `User`









