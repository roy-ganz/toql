
# Joins
A struct can refer to another struct. This is done with a SQL join. 

Joins are automatically added to the SQL statement in these situations
-  Fields in the Toql query refer to another struct. Example `user_phoneId`
-  Fields on a joined struct are always selected. `#[toql(select_always)` 
-  Fields on a joined struct are not `Option<>`. Example `id: u64`

### Example

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

with the Toql query `id` translates into

```sql 
SELECT user.id, null, null, country.id FROM User user 
INNER JOIN Country country ON (user.country_id = country.id)
```

While the same structs with the Toql query `id, mobilePhone_id` translates into

```sql 
SELECT user.id, null, mobile_phone.id, country.id FROM User user 
LEFT JOIN Phone mobile_phone ON (user.mobile_id = mobile_phone.id)
INNER JOIN Country country ON (user.country_id = country.id)
```

# Naming and aliasing
The default table names can be changed with `table`, the alias with `alias`. 

The same Toql query `id` run for this struct

```rust
#[toql table="Users", alias="u"]
struct User {
	 id: u32,	
	 name: Option<String>
	 #[toql(sql_join(self="mobil_id" other="id"), table="Phones", alias="p")]  
	 mobile_phone : Option<Phone>
}
```

now translates into
```sql 
SELECT u.id, null, p.id FROM Users u LEFT JOIN Phones p ON (u.mobile_id = p.id)
```

# Join Attributes
Sql Joins can be defined with
- *self* the column on the referencing table, if omitted the field name is taken
- *other* the column of the joined tabled
- *on* an additional join predicate



# Join Types
Joining on an `Option` field will issue a LEFT JOIN rather than an INNER JOIN. 

If the selected columns cannot be converted into a struct then 
- this will result in a field value of `None` for an `Option<>` type
- will raise an error for non `Option<>` types





