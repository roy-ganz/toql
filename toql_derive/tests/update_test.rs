use toql::derive::Toql;
use toql::mutate::Mutate;

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct UpdateBook {
    #[toql(key)]
    id: u8,
    title: Option<String>, // Selectable, update if some value

    #[toql(preselect)]
    pages: Option<u8>, // Nullable column, update always

    isbn: Option<Option<String>>, // Selectable nullable column, update if some value

    #[toql(join( columns(self = "author_id", other = "id")))]
    author: Option<UpdateUser>, // Selectable inner join, update foreign key if some value
}

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct UpdateUser {
    #[toql(key)]
    id: u8, // Always selected
    username: Option<String>,
}

#[test]
fn update_one() {
    let b = UpdateBook {
        id: 5,
        title: Some(String::from("Foo")),
        pages: Some(6),
        isbn: Some(Some(String::from("12345678-9"))),
        author: Some(UpdateUser {
            id: 16,
            username: None,
        }),
    };

    let (sql, params) = UpdateBook::update_one_sql(&b).unwrap().unwrap();

    assert_eq!(
        "UPDATE UpdateBook t0 SET t0.title = ?, t0.pages = ?, t0.isbn = ?, t0.author_id = ? WHERE t0.id = ?",
        sql
    );
    assert_eq!(["Foo", "6", "12345678-9", "16", "5"], *params);
}

#[test]
fn update_many() {
    let u1 = UpdateUser {
        id: 11,
        username: Some(String::from("Foo")),
    };
    let u2 = UpdateUser {
        id: 22,
        username: Some(String::from("Bar")),
    };
    let users = vec![u1, u2];

    let (sql, params) = UpdateUser::update_many_sql(&users).unwrap().unwrap();

    assert_eq!("UPDATE UpdateUser t0 INNER JOIN UpdateUser t1 SET t0.username = ?, t1.username = ? WHERE t0.id = ? AND t1.id = ?", sql);
    assert_eq!(["Foo", "Bar", "11", "22"], *params);
}

#[test]
fn update_optional() {
    let b = UpdateBook {
        id: 5,
        title: None,
        pages: Some(6),
        isbn: None,
        author: None,
    };

    let (sql, params) = UpdateBook::update_one_sql(&b).unwrap().unwrap();

    assert_eq!("UPDATE UpdateBook t0 SET t0.pages = ? WHERE t0.id = ?", sql);
    assert_eq!(["6", "5"], *params);
}
