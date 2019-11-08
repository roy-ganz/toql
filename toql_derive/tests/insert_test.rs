use toql::mutate::Mutate;
use toql_derive::Toql;

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct NewBook {
    #[toql(key)]
    id: u8,
    title: Option<String>, // Selectable

    #[toql(preselect)]
    pages: Option<u8>, // Nullable column

    isbn: Option<Option<String>>, // Selectable nullable column

    #[toql(preselect, join(columns(self = "author_id", other = "id")))]
    author: Option<NewUser>, // Nullable column
}

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct NewUser {
    #[toql(key, skip_mut)]
    id: u8, // Always selected (auto value, no insert)
    username: Option<String>,
}

#[test]
fn insert_one() {
    let b = NewBook {
        id: 5,
        title: Some(String::from("Foo")),
        pages: Some(42),
        isbn: Some(Some(String::from("12345678-9"))),
        author: Some(NewUser {
            id: 6,
            username: None,
        }),
    };

    let (sql, params) = NewBook::insert_one_sql(&b).unwrap();

    assert_eq!(
        "INSERT INTO NewBook (id,title,pages,isbn,author_id) VALUES (?,?,?,?,?)",
        sql
    );
    assert_eq!(["5", "Foo", "42", "12345678-9", "6"], *params);
}
#[test]
fn insert_many() {
    let u1 = NewUser {
        id: 5,
        username: Some(String::from("Foo")),
    };
    let u2 = NewUser {
        id: 5,
        username: Some(String::from("Bar")),
    };
    let users = vec![u1, u2];

    let (sql, params) = NewUser::insert_many_sql(&users).unwrap();

    assert_eq!("INSERT INTO NewUser (username) VALUES (?) (?)", sql);
    assert_eq!(["Foo", "Bar"], *params);
}

#[test]
fn null_column() {
    let b = NewBook {
        id: 5,
        title: Some(String::from("Foo")),
        pages: None,
        isbn: Some(None),
        author: None,
    };

    let (sql, params) = NewBook::insert_one_sql(&b).unwrap();

    assert_eq!(
        "INSERT INTO NewBook (id,title,pages,isbn,author_id) VALUES (?,?,?,?,?)",
        sql
    );
    assert_eq!(["5", "Foo", "null", "null", "null"], *params);
}

#[test]
fn missing_value() {
    let b = NewBook {
        id: 5,
        title: None, // Title must contain some value -> Error
        pages: None,
        isbn: None,
        author: None,
    };

    let result = NewBook::insert_one_sql(&b);

    assert_eq!(true, result.is_err());
}

#[test]
fn skip_insert() {
    let u = NewUser {
        id: 5,
        username: Some(String::from("Foo")),
    };

    let (sql, params) = NewUser::insert_one_sql(&u).unwrap();

    assert_eq!("INSERT INTO NewUser (username) VALUES (?)", sql);
    assert_eq!(["Foo"], *params);
}
