use toql_core::query_parser::QueryParser;
use toql_core::sql_builder::SqlBuilder;
use toql_core::table_mapper::FieldOptions;
use toql_core::table_mapper::TableMapper;

fn setup_mapper() -> TableMapper {
    let mut mapper = TableMapper::new("Book");
    mapper
        .map_field_with_options("id", "id", FieldOptions::new().preselect(true))
        .map_field("title", "title")
        .map_field("publishedAt", "published_at");
    mapper
}

#[test]
fn order_simple() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("id, +title, publishedAt").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, published_at FROM Book ORDER BY title ASC",
        result.to_sql()
    );
}
#[test]
fn order_priority() {
    let mapper = setup_mapper();
    // Fields can have different ordering priorities: Lower numbers comes first.
    let query = QueryParser::parse("-2id, +title, -3publishedAt").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, published_at FROM Book ORDER BY title ASC, id DESC, published_at DESC",
        result.to_sql()
    );
}

#[test]
fn order_natural() {
    let mapper = setup_mapper();
    // + and +1 have the same priority, so order fields according to their appearance
    let query = QueryParser::parse("-1id, +title, -1publishedAt").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, published_at FROM Book ORDER BY id DESC, title ASC, published_at DESC",
        result.to_sql()
    );
}
