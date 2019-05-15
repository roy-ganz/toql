
# Insert, update and delete
Structs for toql queries include typically a lot of `Option<>` fields. The Toql derive can build proper insert, update and delete functions.

## Keys and skipping
To make this work you need to provide additional information about keys.

```struct
struct User {
  #[toql(delup_key, skip_inup)] // Key for delete / update, never insert / update
	id: u64

	name: Option<String>
}
```

For composite keys mark multiple columns with the `delup_key`.

Join, merge and SQL fields are excluded. To skip other fields from insert or update functions use the `skip_inup` annotation. Useful for auto incremented primary keys or trigger generated values. 

### Example 

```rust
#[derive(Toql)]
struct User {
	#[toql(delup_key, skip_inup)]
	 id: u32,
	 name: Option<String>
}

--snip--
use toql::mysql::insert_one;
use toql::mysql::udate_one;
use toql::mysql::delete_one;

let mut conn = --snip--

let u = User{id:0, name: Some("Susane")};
let x = insert_one(&u, &mut conn); // returns key
u.id = x;
u.name= Some("Peter");
update_one(&u, &mut conn);

delete_one(&u, &mut conn);
```


## Update behaviour
The update function will update fields only if they contains some value. Look at this struct:

```rust
struct User {
	id: u64
	username: String,			// Always updated
	realname: Option<String>, 		// Updated conditionally
	address: Option<Option<<String>>, 	// Optional nullable column, updated conditionally
	#[toql(select_always)]
	info: Option<String> 		// Nullable column, always updated


}
```


# Collections
Collections or dependend structs are **not** affected by insert, delete or update. You must do this manually (for safety reasons).

However functions for collections are provided.


```rust
#[derive(Toql)]

struct Phone {
	#[toql(delup_key, skip_inup)]
	id: u64
}

struct User {
	#[toql(delup_key, skip_inup)]
	 id: u32,
	 phones: vec<Phone>
}

--snip--
use toql::mysql::insert_one;
use toql::mysql::insert_many;

use toql::mysql::delete_one;
use toql::mysql::delete_many;

// TODO



```






