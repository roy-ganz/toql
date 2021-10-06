# Keys
Toql requires you to add the attribute the `key` to field(s) that correspond to the primary key(s) in your database.

For composite keys mark multiple fields with the `key` attribute.

For internal reasons keys must always be the first fields in a struct and the must always be selected: Optional primary keys are not allowed.

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
```struct
#[derive(Toql)]
struct UserLanguage {
  #[toql(key)] 
  user_id: u64

  #[toql(join(), key)]  
  language: Language; // key field inside 'Language' is assumed to be 'code'
}
```
For the example above Toql assumes that the database table `UserLanguage`  has a composite key made up of the two columns `user_id` and `language_code`.

## Generated key struct
The delete_one function from the ToqlApi requires a key instead of a struct to work.

The Toql derive creates for every `Toql` attributed struct a corresponding key struct. 
See query XX or join XX how you can benefit from this



#### Example
```
use crate::user::{User, UserKey};

let key = UserKey::from(10);
toql.delete_one(key.into()).await?; // Convert 
```

## Unkeyable fields
Only columns and inner joins can be used as keys. Merged fields (`Vec<T>`) and fields that map to an Sql expression (`#[toql(sql="..")`) cannot be used as keys.
