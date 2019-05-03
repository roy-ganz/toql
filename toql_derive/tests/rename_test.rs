extern crate toql_derive;
use toql_derive::Toql;

#[derive(Debug, Clone, Toql)]
#[toql(tables="SHOUTY_SNAKE_CASE", columns="mixedCase", skip_indelup)]
struct MyBook {
    #[toql(column="book_id")]
    id: u8,
   
    long_title: Option<String>,
    
    #[toql(sql_join(self = "author_id", other = "id"), alias = "a", table="UserTable")]
    author: Option<MyUser>,

    #[toql(sql_join(other = "ID", on="r.reader = true"), alias = "r", table="UserTable")] // self is taken from field `co_author` 
    co_author: Option<MyUser>,
}

#[derive(Debug, Clone, Toql)]
#[toql(table="UserTable", skip_indelup)]
struct MyUser {
    #[toql(column="ID")]
    id: u8, 
    username: Option<String>,
}

#[test]
fn rename() {
    let mapper = toql::sql_mapper::SqlMapper::map::<MyBook>("b");
    
    let query = toql::query_parser::QueryParser::parse("*, author_id, coAuthor_id"); // Select all top fields and id from author and co-author
    let result = toql::sql_builder::SqlBuilder::new().build(&mapper, &query.unwrap());
    // Sometimes failes becuse join order different
    assert_eq!("SELECT b.book_id, b.longTitle, a.ID, null, r.ID, null FROM MY_BOOK b LEFT JOIN UserTable r ON (b.coAuthor = r.ID AND (r.reader = true)) LEFT JOIN UserTable a ON (b.author_id = a.id)", result.unwrap().to_sql());
   
                

}
