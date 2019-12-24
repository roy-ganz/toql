# Keys
Toql requires you to attribute the field(s) that correspond to the primary key(s) in your database with `key`.


```struct
struct User {
  #[toql(key, skip_skip)] // Key for delete / update, never insert / update
	id: u64
	name: Option<String>
}
```

For composite keys mark multiple fields with the `key`.

Auto generated keys (auto increment) are usually attributed with `skip_mut` to avoid insert or update. 


## Joins
It is also possible to mark an *inner* join with `key`. This is useful for association tables, like so

```struct
struct UserLanguage {
  #[toql(key)] 
  user_id: u64

  #[toql(join(), key)]  
  language: Language
}
```

For a join used as a key the SQL builder takes the primary key(s) of the joined struct to guess the foreign key columns.

For the example above the database table is assumed to have two columns : user_id and language_id.


## Generated key struct
The Delete and Select functions require a key to the struct you want to delete, resp. select. 
The Toql derive creates key structs. 

A struct can be converted into their key struct. This operation can however fail, if optional key fields are none.

```
let key = UserKey(10);
let user = toql.select::<User>(key);

toql.delete::<User>(user.try_into()?); // Convert struct into key

```



## Unkeyable fields
Merged fields `Vec<T>` and fields that map to an SQL expression `#[toql(sql="..")` cannot be used as keys.
