use std::collections::HashSet;
use toql::{
    mock_db::MockDb,
    prelude::{query, Cache, ContextBuilder, Join, Toql, ToqlApi},
};
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(roles(load = "level1_load", update = "level1_update"))]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    #[toql(roles(update = "text1_update"))]
    text1: Option<String>,

    #[toql(skip_wildcard, skip_mut)]
    text2: Option<String>,

    #[toql(roles(load = "text3_load"))]
    text3: Option<String>,
}

fn toql_for_roles<'a>(role_names: &[&str], cache: &'a Cache) -> MockDb<'a> {
    let mut roles = HashSet::new();
    role_names.into_iter().for_each(|r| {
        roles.insert(r.to_string());
    });

    let context = ContextBuilder::new().with_roles(roles).build();
    MockDb::with_context(cache, context)
}

#[tokio::test]
#[traced_test("info")]
async fn mut_selection() {
    let cache = Cache::new();

    // Select mut selection
    // Fails, because struct role is missing
    let mut toql = MockDb::from(&cache);
    let q = query!(Level1, "$mut");
    assert!(toql.load_many(q).await.is_err());

    // Select mut selection on level 2
    // Select keys from level 1 + 2 and mutable fields on level 2
    let mut toql = toql_for_roles(&["level1_update", "level1_load"], &cache);
    let q = query!(Level1, "$mut");

    assert!(toql.load_many(q).await.is_ok());
    // Only load id, because
    // - text1 is missing update role
    // - text2 is skipped for mut
    // - text3 is missing for load role
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id FROM Level1 level1"
    );

    let mut toql = toql_for_roles(
        &["level1_update", "level1_load", "text1_update", "text3_load"],
        &cache,
    );
    let q = query!(Level1, "$mut");

    assert!(toql.load_many(q).await.is_ok());
    // Load all fields, except text2, which is skip_mut
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1, level1.text3 FROM Level1 level1"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn all_selection() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Fail, because load role is missing
    let q = query!(Level1, "$all");
    assert!(toql.load_many(q).await.is_err());

    // Select all selection
    // Skip text3 because role is missing
    let mut toql = toql_for_roles(&["level1_load"], &cache);
    let q = query!(Level1, "$all");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1, level1.text2 FROM Level1 level1"
    );

    // Select all selection
    let mut toql = toql_for_roles(&["level1_load", "text3_load"], &cache);
    let q = query!(Level1, "$all");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1, level1.text2, level1.text3 FROM Level1 level1"
    );
}
