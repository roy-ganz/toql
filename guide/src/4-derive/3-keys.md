# Keys
Toql requires you to add the attribute `key` to the field that correspond to the primary key in your database.

For composite keys mark multiple fields with the `key` attribute.

For internal reasons keys must always be the first fields in a struct and the must always be selected. Optional primary keys are not allowed.

#### Example:
```struct
#[derive(Toql)]
struct User {
  #[toql(key)]
	id: u64
	name: Option<String>
}
```


## Joins
*Inner* joins can also have the `key` attribute. This is useful for association tables.

For a join used as a key the SQL builder takes the primary key(s) of the joined struct to guess the foreign key columns.

#### Example:
```rust

#[derive(Toql)]
struct Language {

  #[toql(key)] 
  code: String,

  name: String
}

#[derive(Toql)]
struct UserLanguage {

  #[toql(key)] 
  user_id: u64

  #[toql(join(), key)]  
  language: Language; 
}
```
For the example above Toql assumes that the database table `UserLanguage`  has a composite key made up of the two columns `user_id` and `language_code`. You can change this assumption, see [here](4-derive/4-joins.md).

## Generated key struct
The Toql derive creates for every struct a corresponding key struct. The key struct contains only the fields marked as key form the derived stuct.

Keys are useful to :
  - Delete an value with `delete_one`
  - Build a [query](3-api/2-load.md) 
  - Update a [join](4-derive/4-joins.md)

Keys can be serialized and deserialized with serde [TODO feature].
This allows web clients to send either a full joined entity or just the key of it 
if they want to update some dependency.




#### Example
```
use crate::user::{User, UserKey};

let key = UserKey::from(10);
toql.delete_one(key).await?; // Convert 
```

## Unkeyable fields
Only columns and inner joins can be used as keys. Merged fields (`Vec<T>`) and fields that map to an Sql expression (`#[toql(sql="..")`) cannot be used as keys.
