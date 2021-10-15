
## Loading

There are three loading functions: `load_one`, `load_many` and `load_page`.

All loading functions take a [Toql query](../5-query-language/1-introduction.md) and will translate it into SQL SELECT statement(s). The resulting rows will be deserialized and returned.

If your Toql structs contain a `Vec` of other structs, the load functions issue multiple SELECT statements and merge the results.

If you expect exactly one result, use `load_one`.

```
    use toql::prelude::{query, ToqlApi};

    let toql = ...
    let q = query!(...);
    let u = toql.load_one(q).await?;
```
The function will return `ToqlError::NotFound` if no row matched the query filter or `ToqlError::NotUnique` if more than one row matched.
To load zero or one row use `load_page`, see below.

Similarly, if you need to load multiple rows:

```
    use toql::prelude::{query, ToqlApi};

    let toql = ...
    let q = query!(...);
    let u = toql.load_many(q).await?;
```

`load_many` returns a `Vec` with deserialized rows. 
The `Vec` will be empty, if no row matched the filter criteria.

`load_page` allows you to select a page with a starting point and a certain length. 
It returns a `Vec` and count information.

The count information is either `None` for an uncounted page 
or contains count statistics that is needed for typical pagers in web apps, see below.
(After all Toql was initially created to serve web pages.)

To load the first 10 -or less- rows do this:

```
    use toql::prelude::{query, ToqlApi, Page};

    let toql = ...
    let q = query!(...);
    let (u, _) = toql.load_page(q, Page::Uncounted(0, 10)).await?;
```

To serve a webpage, you may also want to include count informations.

```
    use toql::prelude::{query, ToqlApi, Page};

    let toql = ...
    let q = query!(...);
    let (u, c) = toql.load_page(q, Page::Counted(0, 10)).await?;
```

The code is almost the same, but the different page argument will issue two more select statements
to return the *filtered* page length and the *total* page length. Let's see what those are:

Suppose you have a table with books. The books have an id, a title and an author_id.

|id|title| author_id|
|--|-----|----------|
| 1| The world of foo| 1|
| 2| The world of bar| 1|
| 3| The world of baz| 1|
| 4| What 42 tells me| 1|
| 5| Flowers And Trees|2|

Let's assume we have a webpage that contains a pager with page size _**2**_ and a pager filter. 
The author wants to see all books that contain the word _**world**_. What will he get?
 - The first two rows (id 1, id 2).
 - The *filtered page count* of 3, because 3 rows match the filter criteria. 
   The pager can now calculate the number of pages: ceil(3 / 2) = 2
 - The *total page count* of 4. The author knows now that with a different filter query, he could
   get at most 4 rows back.
 
 In practice the *total page count* is not so straight forward to select: 
 Toql needs to decide, which filters from the query to consider or ignore when building the count SQL statement.
 For the *total page count* only filters are used on fields that are listed in the special [count selection](../4-derive/9-selections.md) and [predicates](../4-derive/10-predicates.md) that are marked as count filters.
 