
# Merges

A struct can contain a `Vec` of other structs. Because this can't be loaded directly in SQL, Toql will execute multiple queries and merge the results. 

```rust
#[derive(Toql)]
#[toql(auto_key = true)]
struct User {

	#[toql(key)]
	 id: u32,

	 name: Option<String>

	 #[toql(merge())]  
	 mobile_phones : Vec<Phone>
}

#[derive(Toql)]
struct Phone {

	#[toql(key)]
	number: Option<String>

	prepaid : Option<bool>
}
```

Selecting all fields from above with `*, mobilePhones_*` will run 2 SELECT statements and merge the resulting `Vec<Phone>` into `Vec<User>` by the common value of `User.id` and `Phone.user_id`.

## Renaming merge columns
By default the merge column names follow the pattern above. However it's possible to explicitly specify the column names:

```rust
#[toql(merge(columns(self="id", other="user_id")))]  
phones : Vec<Phone>
```


## No association table with `join_sql `

Often in a 1-n-1 situation the association table (n) does not contain any other columns apart 
from the composite key. In those situations it's often desirable to skip it.

Let's go with an example:

```rust
#[derive(Toql)]
#[toql(auto_key = true)]
struct User {

	#[toql(key)]
	id: u32,

	name: Option<String>

	#[toql(merge()))] // Default merging on User.id = UserCountry.user_id
	countries1 : Vec<UserCountry>

	 #[toql(merge(
        join_sql = "JOIN UserCountry uc ON (...id = uc.country_id)",
        columns(self = "id", other = "uc.user_id")
    ))]  
	 countries2 : Vec<Country>
}

#[derive(Toql)]
struct UserCountry {

	#[toql(key)]
	 user_id: u32,

	#[toql(key, join())] // Default joining on UserCountry.country_id = Country.id
	 country: Country
	 
}
#[derive(Toql)]
struct Country {

	#[toql(key)]
	 id: String,

	 name: Option<String>
}
```

Wow, a lot going on here:
- `countries1` merges on default column names (User.id = UserCountry.user_id).
  Here the `Vec` contains `UserCountry`, which does not contain any interesting data and
  is unconvenient when accessing `Country `.

- `countries2` skips the association table with a custom SQL join. 
  Let's look at `join_sql` first: The special other alias `...` refers - as always- to the merged struct (Country here), 
  so  `Country` will be joined with `UserCountry` on `Country.id = uc.country_id`.
  After the select Toql merges the countries into the users on common column values of `User.id` and `uc.user_id` column value. 
  Because the later column is already aliased with `uc` no alias will be added. 

- In `UserCountry`, notice the nice example of a composite key made up with a join :)



## No association table with `#[toql(foreign_key)]`
In the example above `Country` knows nothing about the `User`, so we must merge with `join_sql`.

However sometimes the merged struct does have a suitable foreign key and we can apply a different pattern:

In the example below we don't have a classic association table.
Still we merge normally on `User.id` = `Todo.user_id`, but `Todo.user_id` is not part of a composite key, as it would be in a asscociation table. Instead it is just a normal foreign key.

This is not a problem when loading the merge. But when doing inserts, 
Toql wishes to update `Todo.user_id` to ensure the foreign key contains the right value.
If `Todo.user_id` was part of the primary key this would work out of the box. 
But since it's not, we have to mark it with `#[foreign_key]`. This tells to consider this column too when setting keys.

#### Foreign key example


```rust
#[derive(Toql)]
#[toql(auto_keys= true)]
struct User {
	#[toql(key)]
	id: u64,

	#[toql(merge())]
	todos: Vec<UserTodo>
}


#[derive(Toql)]
#[toql(auto_keys= true)]
struct Todo {
	#[toql(key)]
	id: u64,

	#[toql(foreign_key)]
	user_id: ,
	
	what: String
}
```




