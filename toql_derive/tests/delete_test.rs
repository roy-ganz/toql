use toql::derive::Toql;
use toql::mutate::Delete;

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct DeleteBook {
    #[toql(key)]
    id: u8,
    title: Option<String>,

    #[toql(join(columns(self = "author_id", other = "id")), alias = "a")]
    author: Option<DeleteUser>,
}

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct DeleteUser {
    #[toql(key)]
    id: u8, // Always selected
    #[toql(key)]
    id2: u8,
    username: Option<String>,
}

#[test]
fn delete_one() {
    let (sql, params) = DeleteBook::delete_one_sql(DeleteBookKey(5)).unwrap();

    assert_eq!("DELETE t FROM DeleteBook t WHERE (t.id = ?)", sql);
    assert_eq!(["5"], *params);
}

#[test]
fn delete_many() {
    let books = vec![DeleteBookKey(5), DeleteBookKey(24)];

    let (sql, params) = DeleteBook::delete_many_sql(books).unwrap().unwrap();

    assert_eq!(
        "DELETE t FROM DeleteBook t WHERE (t.id = ?) OR (t.id = ?)",
        sql
    );
    assert_eq!(["5", "24"], *params);
}
