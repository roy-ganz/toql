## Updates

There are two update functions: `update_one`, and `update_many`. 

The are used like so:

```
use toql::prelude::{ToqlApi, paths};

let u = User {id:27, title: "hello".to_string(), address: None};

toql.update_one(&mut u, fields!(top))?;
toql.update_one(&mut u, fields!(User, "*"))?;

toql.update_many(&[&mut u], fields!(top))?;
```

In the example above all three statements do the same.



### The fields! macro
The `fields!` macro compiles a fiels list. Any invalid paths and field names show up at compile time.

The update function will consider all fields form the field list to update. Optional fields will only 
be updated if they contain some value. See Mapping XXX for details

#### Joins
You can update only the foreign key of a join or field from the join. Consider this field list:

```
let f = fields!(User, "*, addess, addess_*, address_id")
```

With `*` we consider all simple fields from User for updating, 
`address` will update the foreign key to `Address` in the `User` table,
`address_*` will update all simple fields in table `Address`
and finally `address_id` has no effect, since keys cannot be updated.

#### Merges
Updates can delete and insert struct to update merges.

Consider this field list:

```
let f = fields!(User, "*, books, books_*")
```

With `*` we consider all simple fields from User for updating, 
`books` will delete all books that are linked to the user but are not found in the books vector. 
It will also insert new book (and possible partial joins)
`books_*` will update all simple fields in the existing books.

Here a full working example.

```
    #[derive(Debug, PartialEq, Toql)]
    struct Book {
        #[toql(key)]
        id: u64,
        #[toql(key)]
        user_id: u64,
        title: Option<String>
    }

    #[derive(Debug, PartialEq, Toql)]
     #[toql(auto_key= true)]
    struct User {
        #[toql(key)]
        id: u64,
        name: Option<String>,
        #[toql(merge())]
        books : Option<Vec<Book>>
    }

    let u = User {
        id: 27,
        title: Some("Joe Pencil"),
        books: Some(vec![
            Book{
                id: 100,
                user_id: 0,
                title: Some("King Kong".to_string())
            },
            Book{
                id: 200,
                user_id: 27,
                title: Some("Batman".to_string())
            }
        ])
    }

    toql.update_one(&mut u, fields!("*, books, books_*")).await?;
    
```

To mark new books, add them with an invalid key. A value of 0 or '' is considered invalid.
Normally databases start counting indexes from 1 and some databases consider an empty string like null, which is 
also forbidden. So this idea of invalid key should normally work, however check with you database.

 As book has a composite key (id, user_id). For an invalid key here we set the user_id to 0. 
 Toql will notice that and insert a new book (with the corrected user_id of 27) and consider all simple fields form the second book with id 200.





 







