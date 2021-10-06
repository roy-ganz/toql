## Writing backend independend functions

This chapter shows how to write functions that work for multiple databases. 
To write really portable code however, you must also have portable SQL expressions, 
which is out of Toql`s scope.

Let dive in with an example:

```
use std::result::Result;
pub async load_std_selection<T, B,R,E>(toql: B) -> Result::(Vec<T>, E)
where B: Backend<R,E>, T: Load<R,E> 
{
    toql.load_many("$").await?
}
```

Lot of generics! The signature says, that the function takes a backend B that can deserialize from rows of type `R` and may produces errors `E`. 
It returns a Vector of a type `T ` that can be loaded (deserialized) from a Row `R` and may produce an error `E`.

Every database backend is implemented for its own row types and errors. Calling the function with a concrete backend, 
such as `MySqlAsync` will allow rust to infere all generic parameters.

Fortunately other traits are easier. Let's see how to write a generic insert function.

```
pub async insert_simple<T, B,R,E>(toql: B, entity: &mut T) -> std::result::Result::((), E)
where B: Backend<R,E>, T: Insert 
{
    toql.insert_one(entity).await?
}
```

Likewise for other operations use the traits `Update`, `Delete` and `Count`.





