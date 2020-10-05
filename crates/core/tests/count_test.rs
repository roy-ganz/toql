use toql_core::query_parser::QueryParser;
use toql_core::sql_builder::SqlBuilder;
use toql_core::sql_expr_parser::SqlExprParser;
use toql_core::sql_mapper::FieldOptions;
use toql_core::sql_mapper::JoinType;
use toql_core::sql_mapper::SqlMapper;

struct User {}

fn setup_mapper() -> SqlMapper {
    let mut mapper = SqlMapper::new::<User>("User");
    mapper
        .map_join(
            "book",
            "Book",
            SqlExprParser::parse("JOIN Book b").unwrap(),
            SqlExprParser::parse("..id = ...id").unwrap(),
        )
        .map_column("id", "id")
        .map_column("username", "username");
    mapper
}

#[test]
fn count_simple() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("*, book_id eq 1").unwrap();
    let result = SqlBuilder::new().build_count(&mapper, &query).unwrap();

    assert_eq!("SELECT 1 FROM User", result.to_sql());
}
#[test]
fn count_filter() {
    let mut mapper = setup_mapper();
    mapper.alter_field("book_id", "b.id", FieldOptions::new().count_filter(true));
    let query = QueryParser::parse("*,book_id eq 1").unwrap();
    let result = SqlBuilder::new().build_count(&mapper, &query).unwrap();
    assert_eq!(
        "SELECT 1 FROM User JOIN Book b ON (id = b.id) WHERE b.id = ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["1"]);
}
#[test]
fn count_select() {
    let mut mapper = setup_mapper();
    mapper.alter_field("id", "id", FieldOptions::new().count_select(true));
    mapper.alter_field("book_id", "b.id", FieldOptions::new().count_filter(true));
    let query = QueryParser::parse("*,book_id eq 1").unwrap();
    let result = SqlBuilder::new().build_count(&mapper, &query).unwrap();
    assert_eq!(
        "SELECT id FROM User JOIN Book b ON (id = b.id) WHERE b.id = ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["1"]);
}
