use toql::mock_db::MockDb;
use toql::prelude::{query, Cache, SqlBuilderError, Toql, ToqlApi, ToqlError};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(selection(name = "std", fields = "*, level2"))]
pub struct Level1 {
    #[toql(key)]
    id: u64,

    text: Option<String>,

    #[toql(skip_wildcard, skip_mut)]
    text2: Option<String>,

    #[toql(merge)] // Default mapping
    level2: Option<Vec<Level2>>, // Preselected inner join
}
#[derive(Debug, Default, Toql)]
#[toql(selection(name = "custom", fields = "text, level3_text"))]
pub struct Level2 {
    #[toql(key)]
    id: u64,

    #[toql(foreign_key)]
    level1_id: u64,

    text: Option<String>,

    #[toql(merge)]
    level3: Option<Vec<Level3>>,
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,

    #[toql(foreign_key)]
    level2_id: u64,

    text: String,
}

#[tokio::test]
#[traced_test("info")]
async fn std_selection() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);
    let select1 = "SELECT level1.id, level1.text FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                    FROM Level2 level2 \
                    JOIN Level1 level1 \
                    ON (level1.id = level2.level1_id AND level1.id = 1)";

    toql.mock_rows(select1, vec![row!(1u64, "level1")]);
    toql.mock_rows(select2, vec![]);

    // Select std selection
    let q = query!(Level1, "$");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);

    // Select std selection on level 2
    // Fails because $std is not defined.
    toql.mock_rows("SELECT level1.id FROM Level1 level1", vec![row!(1u64)]);
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
    let select1 = "SELECT level1.id FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                    FROM Level2 level2 \
                    JOIN Level1 level1 \
                    ON (level1.id = level2.level1_id AND level1.id = 1)";
    let select3 = "SELECT level1_level2.id, level3.id, level3.level2_id, level3.text \
                    FROM Level3 level3 \
                    JOIN Level2 level1_level2 \
                    ON (level1_level2.id = level3.level2_id AND level1_level2.id = 2)";

    toql.mock_rows(select1, vec![row!(1u64,)]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, "level2")]);
    toql.mock_rows(select3, vec![]);

    let q = query!(Level1, "$level2_custom");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2, select3]);
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
    let select1 = "SELECT level1.id FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                    FROM Level2 level2 \
                    JOIN Level1 level1 \
                    ON (level1.id = level2.level1_id AND level1.id = 1)";
    toql.mock_rows(select1, vec![row!(1u64)]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, "level2")]);

    let q = query!(Level1, "$level2_mut");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
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
    let select1 = "SELECT level1.id FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                    FROM Level2 level2 \
                    JOIN Level1 level1 \
                    ON (level1.id = level2.level1_id AND level1.id = 1)";

    toql.mock_rows(select1, vec![row!(1u64)]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, "level2")]);

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
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
    let select1 = "SELECT level1.id FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id \
                    FROM Level2 level2 \
                    JOIN Level1 level1 \
                    ON (level1.id = level2.level1_id AND level1.id = 1)";

    toql.mock_rows(select1, vec![row!(1u64)]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64)]);
    let n = toql.load_many(&q).await;
    println!("{:?}", n);

    //assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
}
