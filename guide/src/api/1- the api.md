**The Toql Api**

Toql relies on backends to handle database differences. 
This backends implement also a common trait, the ToqlApi, 
which serves as an entry point for all users to call any function.
The functions in the ToqlApi then call functions from the Toql library to do their job.
This chapter explains how tu use the ToqlApi.

It is also possible to write backend agnostic code. See net chapter for details on this.

## Creating the backend



## Loading

There are three loading functions: load_one, load_many and load_page.
All loading functions will select, filter and order columns or expressions 
acccording to the query argumnet and the type mapping, see XXX . If needed, the load functions issue multiple select
statements on your database and merge the results.

If you expect exactly one result, use load_one

```
    use toql::prelude::{query, ToqlApi};

    let toql = ...
    let q = query!(...);
    let u = toql.load_one(q).await?;
```
The function will return NotFound if no row matched the query filter or NotUnique if too many rows matched.
To load zero or one row use load_page, see below.

Similarly, if you need to load multiple rows:


```
    use toql::prelude::{query, ToqlApi};

    let toql = ...
    let q = query!(...);
    let u = toql.load_many(q).await?;
```

load_many returns a Vec<> with deserialized rows. 
The Vec will be empty, if no row matched the filter criteria.

load_page allows you to select a page with a starting point and a certain length. 
It returns a tuple of a Vec and count infromation.

The count information is either None for an uncounted page, 
or contains count statistics that are needed for typical pagers in web apps, see below.
(After all Toql was initially created to serve web pages)

In case you want to load the first 10 -or less- rows do this

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

The code is almost the same, but the altered page argument will issue two more select statements
to return the filtered page length and the unfiltered page length. Let's see what those are:

Suppose you have a table with books. The books have an id, a title and an author_id.

Books:
------
id, title, author_id
1, The world of foo, 1
2, The world of bar, 1
3, The world of baz, 1
4, My life with 42, 1
5, Plants And Trees, 2


Let's assume we have a webpage that contains a pager with page size 2 and a pager filter. 
The author wants to see all books that contain the word 'world'. What will he get?
 - The first two rows (id 1, id 2).
 - The filtered page count of 3, because 3 rows match the filter criteria. 
   The pager can now calculate the number of pages: ceil(3 / 2) = 2
 - The unfiltered page count of 4. The author knows now that with a different filter query, he could
   get at most 4 rows back.
 
 In practice the unfiltered page count is not so straight forward to select. 
 However Toql provides that reliably out of the box. How can Toql figure out, 
 which record belongs to author 1 and which to author 2? 
 Toql uses certain fields that are marked to make a difference ('count fields'). 
 Query filters that use one of those fields will be included in the count query.
 A count Selection, see XXX is used to defined those fields.













