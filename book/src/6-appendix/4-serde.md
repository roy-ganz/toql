
# Serde

Toql structs usually have a lot of `Option` types to make fields selectable with a query.
Let's look how to attribute them with serde for smotth interaction.

## Serializing
It's nice to omit unselected fields. This can easily achieved with `#[serde(skip_serializing_if = "Option::is_none")]`

### Serialize example
```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    age: Option<u8>

    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<Option<Join<Address>>> // Selectable left join
```

## Deserializing
Your server needs deserializing either 
- when creating a new item 
- or when updating an existing item



### Deserialize example:

```rust
    #[derive(Toql)]
    #[toql(auto_key = true)]
    struct User {
    
    #[serde(default)]  // 'default' allows missing field 'id' in Json, needed typically for insert
    #[toql(key)]
    id: u64
    
    // No Serde attribute: Field must always be present in Json, but may be null -> None
    name: Option<String>

    #[serde(skip_deserializing)]  // Never deserialize expressions
    #[toql(sql = "(SELECT COUNT(*) From Book b WHERE b.author_id = ..id)")]
    pub number_of_books: Option<u64>,
   

    #[serde(default, deserialize_with="des_double_option")] // See below
    address: Option<Option<Join<Address>>> 
    }
```

Notice the double `Option` on the selectable left join `address`. 
When deserializing from JSON the following mapping works:
 
|JSON | Rust|
|-----|-----|
| undefined| None|
| null | Some(None)|
| value | Some(Some(value))|

To make this happen you need a custom deserialization function:

```rust
use serde::{Deserializer, Deserialze};

pub fn des_double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(de).map(Some)
}
```

Now you get the following:
- If you omit address in your JSON `#[serde(default)]` kicks in and you get `None`.
- If you send `"addess": null`, you get `Some(None)`.
- If you send `"address: {"id": 5}"`, you get `Some(Some(Join::Key(AddressKey{id:5})))`.
- If you send `"address: {"id": 5, ...}"`, you get `Some(Some(Join::Entity(Address{id:5, ...})))`.

Toql update will now work as expected.






