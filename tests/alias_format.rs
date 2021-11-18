use pretty_assertions::assert_eq;
use toql::mock_db::MockDb;
use toql::prelude::{
    fields, paths, query, AliasFormat, Cache, ContextBuilder, Join, Toql, ToqlApi,
};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: Option<String>,

    #[toql(merge)] // Default mapping Level1.id = Level2.level1_id
    level2: Option<Vec<Level2>>, // Preselected merge
}
#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(foreign_key)]
    level1_id: u64,
    text: Option<String>,

    #[toql(join)]
    level3: Option<Option<Join<Level3>>>,
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,

    text: Option<String>,
}

#[tokio::test]
#[traced_test("info")]
async fn tiny() {
    let cache = Cache::new();
    let context = ContextBuilder::new()
        .with_alias_format(AliasFormat::TinyIndex)
        .build();
    let mut toql = MockDb::with_context(&cache, context);

    // Load preselects from level 1..3 + all fields from level 4
    let q = query!(Level1, "level2_level3_id");
    let select1 = "SELECT t1.id FROM Level1 t1";
    let select2 = "SELECT t1.id, t2.id, t2.level1_id, t3.id \
                    FROM Level2 t2 \
                    LEFT JOIN (Level3 t3) \
                    ON (t2.level3_id = t3.id) \
                    JOIN Level1 t1 \
                    ON (t1.id = t2.level1_id AND t1.id = 1)";
    toql.mock_rows(select1, vec![row!(1u64, "level1")]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, 3u64, "level2")]);

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
}

#[tokio::test]
#[traced_test("info")]
async fn short() {
    let cache = Cache::new();
    let context = ContextBuilder::new()
        .with_alias_format(AliasFormat::ShortIndex)
        .build();
    let mut toql = MockDb::with_context(&cache, context);

    // Load preselects from level 1..3 + all fields from level 4
    let q = query!(Level1, "level2_level3_id");
    let select1 = "SELECT le1.id FROM Level1 le1";
    let select2 = "SELECT le1.id, le2.id, le2.level1_id, le3.id \
                    FROM Level2 le2 \
                    LEFT JOIN (Level3 le3) \
                    ON (le2.level3_id = le3.id) \
                    JOIN Level1 le1 \
                    ON (le1.id = le2.level1_id AND le1.id = 1)";
    toql.mock_rows(select1, vec![row!(1u64, "level1")]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, 3u64, "level2")]);

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
}

#[tokio::test]
#[traced_test("info")]
async fn medium() {
    let cache = Cache::new();
    let context = ContextBuilder::new()
        .with_alias_format(AliasFormat::MediumIndex)
        .build();
    let mut toql = MockDb::with_context(&cache, context);

    // Load preselects from level 1..3 + all fields from level 4
    let q = query!(Level1, "level2_level3_id");
    let select1 = "SELECT level1_1.id FROM Level1 level1_1";
    let select2 = "SELECT level1_1.id, level2_2.id, level2_2.level1_id, level3_3.id \
                    FROM Level2 level2_2 \
                    LEFT JOIN (Level3 level3_3) \
                    ON (level2_2.level3_id = level3_3.id) \
                    JOIN Level1 level1_1 \
                    ON (level1_1.id = level2_2.level1_id AND level1_1.id = 1)";
    toql.mock_rows(select1, vec![row!(1u64, "level1")]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, 3u64, "level2")]);

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
}

#[tokio::test]
#[traced_test("info")]
async fn canonical() {
    let cache = Cache::new();
    let context = ContextBuilder::new()
        .with_alias_format(AliasFormat::Canonical)
        .build();
    let mut toql = MockDb::with_context(&cache, context);

    // Load preselects from level 1..3 + all fields from level 4
    let q = query!(Level1, "level2_level3_id");
    let select1 = "SELECT level1.id FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2_level3.id \
                    FROM Level2 level2 \
                    LEFT JOIN (Level3 level2_level3) \
                    ON (level2.level3_id = level2_level3.id) \
                    JOIN Level1 level1 \
                    ON (level1.id = level2.level1_id AND level1.id = 1)";
    toql.mock_rows(select1, vec![row!(1u64, "level1")]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, 3u64, "level2")]);

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
}
