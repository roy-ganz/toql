use toql::mock_db::MockDb;
use toql::prelude::{query, Cache, Join, SqlBuilderError, Toql, ToqlApi, ToqlError};
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(selection(name = "std", fields = "*, level2"))]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: Option<String>,

    #[toql(skip_wildcard, skip_mut)]
    text2: Option<String>,

    #[toql(join)] // Default mapping
    level2: Option<Level2>, // Preselected inner join
}
#[derive(Debug, Default, Toql)]
#[toql(selection(name = "custom", fields = "text, level3_text"))]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    text: Option<String>,
    #[toql(join)]
    level3: Option<Join<Level3>>,
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    text: String,
}

#[tokio::test]
#[traced_test("info")]
async fn std_selection() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Select std selection
    let q = query!(Level1, "$");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text, level1_level2.id, level1_level2.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2) ON (level1.level2_id = level1_level2.id)"
    );

    // Select std selection on level 2
    // Fails because $std is not defined.
    let q = query!(Level1, "$level2_std");
    let err = toql.load_many(q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::SelectionMissing("std".to_string()))
            .to_string()
    );
}

#[tokio::test]
#[traced_test("info")]
async fn custom_selection() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Select custom selection on level 2
    let q = query!(Level1, "$level2_custom");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2.text, level1_level2_level3.id, level1_level2_level3.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2 \
                JOIN (Level3 level1_level2_level3) \
                ON (level1_level2.level3_id = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id)"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn mut_selection() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Select mut selection
    // Select key and mutable fields on level1
    let q = query!(Level1, "$mut");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1"
    );

    // Select mut selection on level 2
    // Select keys from level 1 + 2 and mutable fields on level 2
    let q = query!(Level1, "$level2_mut");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2) \
            ON (level1.level2_id = level1_level2.id)"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn all_selection() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Select mut selection
    // Select key and all fields on level1
    let q = query!(Level1, "$all");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text, level1.text2 FROM Level1 level1"
    );

    // Select mut selection on level 2
    // Select keys from level 1 + 2 and all fields on level 2
    let q = query!(Level1, "$level2_all");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2) \
            ON (level1.level2_id = level1_level2.id)"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn cnt_selection() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Select cnt selection
    // Select fields used to build count query
    let q = query!(Level1, "$cnt");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id FROM Level1 level1"
    );

    // Select cnt selection on level 2
    // Select keys from level 1 + cnt fields from level 2
    // Since no cnt selection is defined on level 2, only preselects are selected
    let q = query!(Level1, "$level2_cnt");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2) \
            ON (level1.level2_id = level1_level2.id)"
    );
}
