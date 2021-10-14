# Partial tables

A database table may split into several tables sharing the same primary key.
This is done because 
- the original table got too many columns
- A group of columns in the table is optional
- You want to control access  with Toql roles.

Partial tables are supported with joins.
```rust

#[derive(Toql)]
#[toql(auto_keys= true)]
struct Question {
    #[toql(key)]
    id: u64

    text: String,

    #[toql(join(columns(self="id", other="question_id")), partial_table)]
    details: Option<QuestionDetails>
}

#[derive(Toql)]
struct QuestionDetails {
    #[toql(key)]
    question_id: u64
    
    font: String
}
```

In the example above `Question` and `QuestionDetails` share the same values for primary keys.This is what  `patial_table` says.
So for a _question_ with _id = 42_ there is a _solution_ with *question_id = 42*. 

Inserts will always insert all partial tables too, whenever a path list asks to insert the base table (Question). 

Also it will avoid to insert a non existing foreign key: If `QuestionDetails` was regular join (without `partial_table`) 
insert would try to set a (non existing) column `details_id` with the value to the primary key of `Questiondetails`. 
This would be correct for regular joins, but fails on partial tables.

Updates have the same behaviour when inserting new merges and for loading `partial_table` has no effect. 




