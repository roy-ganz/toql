use toql_core::query_parser::QueryParser;
use toql_core::sql_builder::SqlBuilder;
use toql_core::sql_mapper::FieldOptions;
use toql_core::sql_mapper::SqlMapper;
use toql_core::sql_mapper::JoinType;

fn setup_mapper() -> SqlMapper {
    let mut mapper = SqlMapper::new("Book b");
    mapper
        .join("author", JoinType::Inner, "User a", "b.id = a.book_id")
        .map_field_with_options("id", "b.id", FieldOptions::new().preselect(true))
        .map_field("title", "b.title")
        .map_field("publishedAt", "b.published_at")
        .map_field("author_id", "a.id")
        .map_field("author_username", "a.username");
    mapper
}

#[test]
fn select_wildcard() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("*").unwrap(); // select all top fields
    println!("{:?}", query);
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT b.id, b.title, b.published_at, null, null FROM Book b",
        result.to_sql()
    );
}
#[test]
fn select_double_wildcard() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("**").unwrap(); // select all top fields
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT b.id, b.title, b.published_at, a.id, a.username FROM Book b JOIN User a ON (b.id = a.book_id)",
        result.to_sql()
    );
}
#[test]
fn select_path_wildcard() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("author_*").unwrap(); // select all top fields
    println!("{:?}", query);
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT b.id, null, null, a.id, a.username FROM Book b JOIN User a ON (b.id = a.book_id)",
        result.to_sql()
    );
}

#[test]
fn select_duplicates() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("id, id, title").unwrap(); // select id twice
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT b.id, b.title, null, null, null FROM Book b",
        result.to_sql()
    );
}

#[test]
fn select_optional_join() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("id, title, author_id").unwrap(); // select author from joined table
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT b.id, b.title, null, a.id, null FROM Book b JOIN User a ON (b.id = a.book_id)",
        result.to_sql()
    );
}

#[test]
fn select_hidden() {
    let mapper = setup_mapper();

    // id must always be selected (see mapper), title is hidden
    let query = QueryParser::parse(".id, .title, publishedAt").unwrap();

    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT b.id, null, b.published_at, null, null FROM Book b",
        result.to_sql()
    );
}

#[test]
fn select_missing_field() {
    let mapper = setup_mapper();

    // Field fail does not exist in mapper
    let query = QueryParser::parse("id, fail").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query);

    assert!(result.is_err(), "Field should be missing.");
}
