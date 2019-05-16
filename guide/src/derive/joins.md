
# Joins
A struct can refer to another struct. This is done with a SQL join. 

Joins are automatically added to the SQL statement in these situations:
-  Fields in the Toql query refer to another struct through a path: `user_phoneId`.
-  Fields on a joined struct are always selected: `#[toql(select_always)`. 
-  Fields on a joined struct are not `Option<>`: `id: u64`.

#### Example:

The Toql query `id` translates this

```rust
struct User {
	 id: u32,	
	 name: Option<String>
	 #[toql(sql_join(self="mobile_id" other="id"))]  
	 mobile_phone : Option<Phone>

	 #[toql(sql_join(self="country_id" other="id"))]  
	 country : Country
}

struct Country {
	id: String // Always selected
}

struct Phone {
	id : Option<u64>, 
}
```
into

```sql 
SELECT user.id, null, null, country.id FROM User user 
INNER JOIN Country country ON (user.country_id = country.id)
```

While the Toql query `id, mobilePhone_id` for the same structs translates into

```sql 
SELECT user.id, null, mobile_phone.id, country.id FROM User user 
LEFT JOIN Phone mobile_phone ON (user.mobile_id = mobile_phone.id)
INNER JOIN Country country ON (user.country_id = country.id)
```

## Naming and aliasing
The default table names can be changed with `table`, the alias with `alias`. 

The Toql query `id` for this struct

```rust
#[toql table="Users", alias="u"]
struct User {
	 id: u32,	
	 name: Option<String>
	 #[toql(sql_join(self="mobil_id", other="id"), table="Phones", alias="p")]  
	 mobile_phone : Option<Phone>
}
```

now translates into
```sql 
SELECT u.id, null, p.id FROM Users u LEFT JOIN Phones p ON (u.mobile_id = p.id)
```

## Join Attributes
SQL joins can be defined with
- *self*, the column on the referencing table. If omitted the struct field's name is taken.
- *other*, the column of the joined tabled.
- *on*, an additional SQL predicate. Must include the table alias.

For composite keys use multiple `sql_join` attributes.

#### Example
``` rust
 	#[toql(sql_join(self="country_id", other="id"), sql_join(self="language_id", other="language_id", on="country.language_id = 'en'") ]  
	country : Option<Country>
```


## Join Types
Joining on an `Option` field will issue a LEFT JOIN rather than an INNER JOIN. 

If the selected columns cannot be converted into a struct
- then this will result in a field value of `None` for an `Option<>` type
- or will raise an error for non `Option<>` types





