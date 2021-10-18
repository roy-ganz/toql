# MySQL Enums

To map enums between a column and a struct field 
- some MySQL boilerplate code is required. 
- the enum must implement the `ToString` and `FromStr` traits.

For the first I made a little crate [mysql_enum](https://github.com/roy-ganz/mysql_enum) and for the later several crates exit. Here an example with [strum](https://crates.io/crates/strum):

With this in Cargo.toml
```toml
[dependencies]
mysql_enum = "0.1"
strum = "0.22"
strum_macros = "0.22"
 ```
you can attribute your enums:
 ```Rust
use mysql_enum::MysqlEnum;
use strum_macros::{Display, EnumString};

#[derive(PartialEq, EnumString, Display, MysqlEnum)]
 enum Mood {
    Happy,
    Sad
} 
```
Now _Mood_ can be used:

```
#[derive (Debug, Toql)]
struct User {
    id : u64,
    name: Option<string>
    mood: Option<Mood>
}
```

