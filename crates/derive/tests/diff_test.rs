use toql::derive::Toql;
use toql::mutate::Update;

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct DiffBook {
    #[toql(key)]
    id: u8,
    title: Option<String>, // Selectable, update if some value

    #[toql(preselect)]
    pages: Option<u8>, // Nullable column, update always

    isbn: Option<Option<String>>, // Selectable nullable column, update if some value

    #[toql(join(columns(self = "author_id", other = "id")))]
    author: Option<DiffAuthor>,
}

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct DiffAuthor {
    #[toql(key, skip_mut)]
    id: u8, // Always selected (auto value, no insert)
    username: Option<String>,
}

#[test]
fn diff_one() {
    let author1 = DiffAuthor {
        id: 5,
        username: Some(String::from("Outdated Author")),
    };
    let author2 = DiffAuthor {
        id: 6,
        username: Some(String::from("Updated Author")),
    };
    let outdated = DiffBook {
        id: 5,
        title: Some(String::from("Outdated")),
        pages: Some(6),
        isbn: Some(None),
        author: Some(author1),
    };
    let updated = DiffBook {
        id: 5,
        title: Some(String::from("Updated")),
        pages: Some(10),
        isbn: Some(Some(String::from("1234-5678"))),
        author: Some(author2),
    };

    // Updated details and foreign key, but not join details

    let mut statements = DiffBook::diff_one_sql(&outdated, &updated).unwrap();

    assert_eq!(1, statements.len());

    let (sql, params) = statements.pop().unwrap();

    assert_eq!("UPDATE DiffBook t0 SET t0.title = ?, t0.pages = ?, t0.isbn = ?, t0.author_id = ? WHERE t0.id = ?", sql);
    assert_eq!(["Updated", "10", "1234-5678", "6", "5"], *params);
}
