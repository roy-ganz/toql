
# Merge
A struct can also contain a collection of other structs. Because this cannot be done directly in SQL, Toql will execute multiple queries and merge the results afterwards. 

```rust
struct User {
	 id: u32,
	 name: Option<String>
	 #[toql(merge(self="id", other="user_id"))]  // Struct fields for Rust comparison
	 mobile_phones : Vec<Phone>
}

struct Phone {
	number: Option<String>
	user_id : Option<u32>
}
```

Selecting all fields from above with `**` will run 2 SELECT statements and merge the resulting `Vec<Phone>` into `Vec<User>` by the common value of `user.id` and `phone.user_id`.

## Merge attribute
Because merging is done by Rust, the merge fields must be named after the struct fields.

`#[toql(merge(self="rust_field_name_in_this_struct", other="rust_field_name_in_other_struct"))] `


## Composite fields

To merge on composite fields use the attribute multiple times `#[toql(merge(..), merge(..))`.
