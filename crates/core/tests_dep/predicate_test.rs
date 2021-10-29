fn setup_mapper() -> TableMapper {
    let mut mapper = TableMapper::new("Book");
    mapper
        .map_predicate("user_id", "SELECT 1 FROM Book WHERE User = ?")
        .map_field_with_options("id", "id", FieldOptions::new().preselect(true))
        .map_field("title", "title")
        .map_field("publishedAt", "published_at");
    mapper
}

#[test]
fn predicate_with_one_arg() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("@userId 12, id").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, published_at FROM Book WHERE EXISTS( SELECT 1 FROM Book WHERE User = ?)",
        result.to_sql()
    );
}
