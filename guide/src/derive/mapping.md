
# Mapping names
Struct fields are mapped to Toql query fields and databases by default in a predictable way:
1. Table names are UpperCamelCase.
2. Column names are snake_case.
3. Toql fields are lowerCamelCase.
4. Toql paths are lowerCamelCase, separated with an underscore.


## Database
To adjust the default naming to an existing database scheme use the attributes `tables` and `columns` for a renaming scheme or `table` and `column` for explicit name.

Supported renaming schemes are 
- CamelCase
- snake_case
- SHOUTY\_SNAKE\_CASE
- mixedCase

#### Renaming scheme example:
```rust
#[derive(Toql)]
#[toql(tables="SHOUTY_SNAKE_CASE", columns="UpperCase")]
struct UserRef {
  	user_id: u32
	full_name: String,
}
```
is translated into 

`SELECT t0.UserId, t0.FullName FROM USER_REF t0`

#### Explicit naming example:
Use `table` an the struct and `column` on the fields to set a name.

```rust
#[derive(Toql)]
#[toql(table="User")]
struct UserRef {
	#[toql(column="id")]
	user_id: u32
	full_name: String,
}
```
is translated into 

`SELECT t0.id, t0.full_name FROM User t0`

## Toql fields

Toql fields on a struct are always mixed case, while dependencies are separated with an unserscore.

```rust
#[derive(Toql)]
#[toql(table="User")]
struct UserRef {
	#[toql(column="id")]
	id: u32
	full_name: String,
	#[toql(join())]
	county: Country
}
```
is referred to as

`id, fullName, country_id`



## Exclusion
To exclude fields from mapping attribute them with `skip`.


