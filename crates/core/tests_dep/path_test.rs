use toql_core::query_parser::QueryParser;
use toql_core::sql_builder::SqlBuilder;
use toql_core::table_mapper::FieldOptions;
use toql_core::table_mapper::JoinType;
use toql_core::table_mapper::TableMapper;

fn setup_mapper() -> TableMapper {
    let mut mapper = TableMapper::new("User");
    mapper
        .join("book", JoinType::Inner, "Book b", "id = b.id")
        .map_field_with_options("id", "id", FieldOptions::new().preselect(true))
        .map_field("username", "username")
        .map_field_with_options("book_", "IS_NULL(id)", FieldOptions::new().preselect(true)) // on this path
        .map_field("book_id", "b.id");

    mapper
}

#[test]
fn build_path() {
    let mapper = setup_mapper();
    // Build query for path, ignore other fields
    // Mapper must provide fields of path
    let query =
        QueryParser::parse("fooId, bar_id, author_id, author_username, author_book_id EQ 5")
            .unwrap();

    let result = SqlBuilder::new()
        .build_path("author", &mapper, &query)
        .unwrap();

    assert_eq!(
        "SELECT id, username, IS_NULL(id), b.id FROM User JOIN Book b ON (id = b.id) WHERE b.id = ?",
        result.to_sql()
    );
}
#[test]
fn ignore_path() {
    let mapper = setup_mapper();
    // build query, ignore path
    let query = QueryParser::parse("id, username, book_id, book_foo").unwrap();

    let result = SqlBuilder::new()
        .ignore_path("book") // field "book_foo" is not missing, because path "book" in query is ignored, no error is raised
        .build(&mapper, &query)
        .unwrap();

    assert_eq!("SELECT id, username, null, null FROM User", result.to_sql());
}
