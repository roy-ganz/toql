use toql::mutate::Mutate;
use toql_derive::Toql;

#[derive(Debug, PartialEq, Toql)]
#[toql(skip_query, skip_query_builder)]
struct Book {
    #[toql(key)]
    id: u8,

    #[toql(join(columns(self = "author1_id", other = "id")))] // Non nullable column
    author1: User,

    #[toql(preselect, join( columns(self = "author2_id", other = "id")))] // Nullable column
    author2: Option<User>,

    #[toql(join( columns(self = "author3_id", other = "id")))]
    author3: Option<User>, // Selectable non nullable column

    #[toql(join( columns(self = "author4_id", other = "id")))]
    author4: Option<Option<User>>, // Selectable nullable column */
}

#[derive(Debug, PartialEq, Clone, Toql)]
#[toql(skip_query, skip_query_builder)]
struct User {
    #[toql(key, skip_mut)]
    id: u8, // Always selected (auto value, no insert)
    username: Option<String>,
}

#[test]
fn insert_all_fields() {
    let a = User {
        id: 4,
        username: Some(String::from("Foo")),
    };

    let b = Book {
        id: 5,
        author1: a.clone(),
        author2: Some(a.clone()),
        author3: Some(a.clone()),
        author4: Some(Some(a)),
    };

    let (sql, params) = Book::insert_one_sql(&b).unwrap();

    assert_eq!(
        "INSERT INTO Book (id,author1_id,author2_id,author3_id,author4_id) VALUES (?,?,?,?,?)",
        sql
    );
    assert_eq!(["5", "4", "4", "4", "4"], *params);
}

#[test]
fn update_all_fields() {
    let a = User {
        id: 4,
        username: Some(String::from("Foo")),
    };

    let b = Book {
        id: 5,
        author1: a.clone(),
        author2: Some(a.clone()),
        author3: Some(a.clone()),
        author4: Some(Some(a)),
    };

    let (sql, params) = Book::insert_one_sql(&b).unwrap();

    assert_eq!(
        "INSERT INTO Book (id,author1_id,author2_id,author3_id,author4_id) VALUES (?,?,?,?,?)",
        sql
    );
    assert_eq!(["5", "4", "4", "4", "4"], *params);
}

#[test]
fn update_required_fields() {
    let a = User {
        id: 4,
        username: Some(String::from("Foo")),
    };

    let b = Book {
        id: 5,
        author1: a.clone(),
        author2: None, // Nullable column
        author3: None, // Selectable don't update if None
        author4: None, // Selectable, don't update if None
    };

    let (sql, params) = Book::update_one_sql(&b).unwrap().unwrap();

    assert_eq!(
        "UPDATE Book t0 SET t0.author1_id = ?, t0.author2_id = ? WHERE t0.id = ?",
        sql
    );
    assert_eq!(["4", "NULL", "5"], *params);
}
