use pretty_assertions::assert_eq;
use toql::mock_db::MockDb;
use toql::prelude::{fields, paths, query, Cache, Toql, ToqlApi};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(merge())] // Default mapping Level1.id = Level2.level1_id
    level2: Vec<Level2>, // Preselected merge
}
#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(key)]
    level1_id: u64,
    text: String,

    #[toql(merge(columns(self = "id", other = "level2_id")))] // Specified columns
    level3: Option<Vec<Level3>>, // Selectable merge
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    #[toql(key)]
    level2_id: u64,
    text: String,

    #[toql(merge(
        columns(self = "id", other = "level3_id"),
        join_sql = "...text = 'ABC'"
    ))]
    // Custom ON statement
    level4: Vec<Level4>, // Preselected merge join
}
#[derive(Debug, Default, Toql)]
pub struct Level4 {
    #[toql(key)]
    id: u64,
    #[toql(key)]
    level3_id: u64,
    text: String,
}

fn populated_level() -> Level1 {
    let l4 = Level4 {
        id: 4,
        text: "level4".to_string(),
        level3_id: 3,
    };
    let l3 = Level3 {
        id: 3,
        text: "level3".to_string(),
        level2_id: 2,
        level4: vec![l4],
    };
    let l2 = Level2 {
        id: 2,
        text: "level2".to_string(),
        level1_id: 1,
        level3: Some(vec![l3]),
    };

    Level1 {
        id: 1,
        text: "level1".to_string(),
        level2: vec![l2],
    }
}

#[tokio::test]
#[traced_test("info")]
async fn load1() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 1 + preselected level 2
    let q = query!(Level1, "*");
    let select1 = "SELECT level1.id, level1.text FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                        FROM Level2 level2 \
                        JOIN Level1 level1 \
                        ON (level1.id = level2.level1_id AND level1.id = 1)";

    toql.mock_rows(select1, vec![row!(1u64, "level1")]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, "level2")]);

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
}
#[tokio::test]
#[traced_test("info")]
async fn load2() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load preselects from level 1..3 + all fields from level 4
    let q = query!(Level1, "*, level2_level3_level4_*");
    let select1 = "SELECT level1.id, level1.text FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                        FROM Level2 level2 \
                        JOIN Level1 level1 \
                        ON (level1.id = level2.level1_id \
                            AND level1.id = 1)";

    let select3 = "SELECT level1_level2.id, level1_level2.level1_id, level3.id, level3.level2_id, level3.text \
                        FROM Level3 level3 \
                        JOIN Level2 level1_level2 \
                        ON (level1_level2.id = level3.level2_id \
                            AND level1_level2.id = 2 \
                            AND level1_level2.level1_id = 1)";

    let select4 = "SELECT level1_level2_level3.id, level1_level2_level3.level2_id, level4.id, level4.level3_id, level4.text \
                        FROM Level4 level4 level4.text = \'ABC' \
                        JOIN Level3 level1_level2_level3 \
                        ON (level1_level2_level3.id = level4.level3_id \
                            AND level1_level2_level3.id = 3 \
                            AND level1_level2_level3.level2_id = 2)";

    // level1.id
    toql.mock_rows(select1, vec![row!(1u64, "level1")]);

    // level1.id, level2.id, level2.level1_id
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, "level2")]);

    // level1_level2.id, level1_level2.level1_id, level3.id, level3.level2_id, level3.text
    toql.mock_rows(select3, vec![row!(2u64, 1u64, 3u64, 2u64, "level3")]);

    // level1_level2_level3.id, level1_level2_level3.level2_id, level4.id, level4.level3_id, level4.text
    toql.mock_rows(select4, vec![row!(3u64, 2u64, 4u64, 3u64, "level4")]);

    assert!(toql.load_many(q).await.is_ok());

    assert_eq!(
        toql.take_unsafe_sqls(),
        [select1, select2, select3, select4]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn insert() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = Level1::default();

    // insert level 1
    assert!(toql.insert_one(&mut l, paths!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level1 (id, text) VALUES (0, '')"
    );

    // insert path levels 1..4
    // this will only insert level 1,
    // level 2.. is skipped (empty Vec)
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3_level4"))
        .await
        .is_ok());

    assert_eq!(
        toql.take_unsafe_sqls(),
        ["INSERT INTO Level1 (id, text) VALUES (0, '')",]
    );

    // insert path levels 1..4
    // this will insert level 1..4
    let mut l = populated_level();
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3_level4"))
        .await
        .is_ok());
    let mut sqls = toql.take_unsafe_sqls(); // Sorting needed, because inserts come from unsorted hashset
    sqls.sort();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level1 (id, text) VALUES (1, 'level1')",
            "INSERT INTO Level2 (id, level1_id, text) VALUES (2, 1, 'level2')",
            "INSERT INTO Level3 (id, level2_id, text) VALUES (3, 2, 'level3')",
            "INSERT INTO Level4 (id, level3_id, text) VALUES (4, 3, 'level4')",
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn update() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Update level 1
    // Nothing is updated, fields are empty
    let mut l1 = Level1::default();
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(toql.sqls_empty(), true);

    // Update level 4 (text)
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_level4_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level4 SET text = 'level4' WHERE id = 4 AND level3_id = 3"
    );

    // Update level 1 - 4
    let mut l1 = populated_level();
    assert!(toql
        .update_one(
            &mut l1,
            fields!(
                Level1,
                "*, level2_*, \
            level2_level3_*, level2_level3_level4_*"
            ),
        )
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "UPDATE Level1 SET text = 'level1' WHERE id = 1",
            "UPDATE Level2 SET text = 'level2' WHERE id = 2 AND level1_id = 1",
            "UPDATE Level3 SET text = 'level3' WHERE id = 3 AND level2_id = 2",
            "UPDATE Level4 SET text = 'level4' WHERE id = 4 AND level3_id = 3"
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn delete() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Delete without filter return no queries
    // This is for safety, otherwise everything would be deleted
    let q = query!(Level1, "*");
    assert!(toql.delete_many(q).await.is_ok());
    assert!(toql.sqls_empty());

    // Delete with filter on level1
    let q = query!(Level1, "id eq 4");
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "DELETE level1 FROM Level1 level1 WHERE level1.id = 4"
    );

    // Delete with filter on level 5
    // No SQL is generated, because merge filter is ignored and there
    // is no direct / joined filter
    let q = query!(Level1, "level2_id eq 5");
    assert!(toql.delete_many(q).await.is_ok());
    assert!(toql.sqls_empty());
}

#[tokio::test]
#[traced_test("info")]
async fn count() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Query contains filter
    let q = query!(Level1, "id eq 4");
    assert!(toql.count(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT COUNT(*) FROM Level1 level1 WHERE level1.id = 4"
    );

    // Filters on merges are ignored
    // Left joins are converted into inner joins
    let q = query!(Level1, "level2_level3_level4_id eq 5");
    assert!(toql.count(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(), "SELECT COUNT(*) FROM Level1 level1");
}
