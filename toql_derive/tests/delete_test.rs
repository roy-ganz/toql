use toql::derive::Toql;
use toql::indelup::Indelup;

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct DeleteBook {
    #[toql(delup_key)]
    id: u8,
    title: Option<String>,

    #[toql(sql_join(self = "author_id", other = "id"), alias = "a")]
    author: Option<DeleteUser>,
}

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct DeleteUser {
    #[toql(delup_key)]
    id: u8, // Always selected
    #[toql(delup_key)]
    id2: u8,
    username: Option<String>,
}

#[test]
fn delete_one() {
    let b = DeleteBook {
        id: 5,
        title: Some(String::from("Foo")),
        author: None,
    };

    let (sql, params) = DeleteBook::delete_one_sql(&b).unwrap();

    assert_eq!("DELETE t FROM DeleteBook t WHERE (t.id = ?)", sql);
    assert_eq!(["5"], *params);
}

#[test]
fn delete_many() {
    let b1 = DeleteBook {
        id: 5,
        title: Some(String::from("Foo")),
        author: None,
    };
    let b2 = DeleteBook {
        id: 24,
        title: Some(String::from("Foo")),
        author: None,
    };
    let books = vec![b1, b2];

    let (sql, params) = DeleteBook::delete_many_sql(&books).unwrap();

    assert_eq!("DELETE t FROM DeleteBook t WHERE (t.id = ?) OR (t.id = ?)", sql);
    assert_eq!(["5", "24"], *params);
}
