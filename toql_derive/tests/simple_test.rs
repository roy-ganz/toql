extern crate toql_derive;
use toql_derive::Toql;

#[derive(Debug, Clone, Toql)]
struct Book {
    id: u8,
    title: Option<String>,
    author_id: u8,
    #[toql(join(self = "author_id", other = "id"), alias = "a")]
    author: Option<User>,
}

#[derive(Debug, Clone, Toql)]
struct User {
    id: u8, // Always selected

  
    username: Option<String>,

    #[toql(skip)]
    other: String,

    #[toql(merge(self = "id", other = "author_id"))]
    books: Vec<Book>,
}

#[test]
fn attributes() {
    let mut mu = toql::sql_mapper::SqlMapper::map::<Book>("book");
    // mu.join("author", "LEFT JOIN User a ON (book.author_id = a.id)"); // Done with toql annotation

    let q = toql::query_parser::QueryParser::parse("id, title, author_id");
    let r = toql::sql_builder::SqlBuilder::new().build(&mu, &q.unwrap());
    assert_eq!("SELECT book.id, book.title, book.author_id, a.id, null FROM Book book LEFT JOIN User a ON (book.author_id = a.id)", r.unwrap().to_sql());
}
