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
fn logic_and_or() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("id, title EQ 'Foo'; title EQ 'Bar', id NE 3").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, null FROM Book WHERE title = ? OR title = ? AND id <> ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["Foo", "Bar", "3"]);
}

#[test]
fn logic_where_parens() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("id, (title EQ 'Foo'; (title EQ 'Bar')), id NE 3").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, null FROM Book WHERE (title = ? OR (title = ?)) AND id <> ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["Foo", "Bar", "3"]);
}
#[test]
fn logic_where_having_parens() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("id, (title EQ 'Foo'; (title !EQ 'Bar')), id NE 3").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, null FROM Book WHERE (title = ?) AND id <> ? HAVING ((title = ?))",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["Foo", "3", "Bar"]);
}
