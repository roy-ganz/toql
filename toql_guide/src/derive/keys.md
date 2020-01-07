# Keys
Toql requires you to add the attribute the `key` to field(s) that correspond to the primary key(s) in your database.

For composite keys mark multiple fields with the `key` attribute.

Auto generated database columns (auto increment values) should be attributed with `skip_mut` to avoid insert or update operations on them. 

#### Example:
```struct
struct User {
  #[toql(key, skip_skip)] // Key for delete / update, never insert / update
	id: u64
	name: Option<String>
}
```


## Joins
*Inner* joins can also have the `key` attribute. This is useful for association tables.

For a join used as a key the SQL builder takes the primary key(s) of the joined struct to guess the foreign key columns.

#### Example:
```struct
struct UserLanguage {
  #[toql(key)] 
  user_id: u64

  #[toql(join(), key)]  
  language: Language; // key field inside 'Language' is assumed to be 'code'
}
```
For the example above Toql assumes that the database table `UserLanguage`  has a composite key made up of the two columns `user_id` and `language_code`.

## Generated key struct
The delete and select functions require a key instead of a struct to work.
The Toql derive creates for every `Toql` attributed struct a corresponding key struct. 

A struct can be converted into its key struct. This conversion however fails, if optional key fields are `None`.

#### Example
```
use crate::user::{User, UserKey};

let key = UserKey(10);
let user = toql.select::<User>(key);
toql.delete::<User>(user.try_into()?); // Convert struct into key, may fail
```


## Unkeyable fields
Merged fields (`Vec<T>`) and fields that map to an SQL expression (`#[toql(sql="..")`) cannot be used as keys.
