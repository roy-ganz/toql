use toql::{
    mock_db::MockDb,
    prelude::{fields, paths, query, Cache, Join, Toql, ToqlApi},
};
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    #[toql(key, join)]
    level2: Join<Level2>, // join on all key columns: Level1.level2_id -> Level2.id, Level1.level2_level3_id -> Level2.level3_id
    text: String,
}

#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(key, join(columns(self = "level3_id", other = "id")))]
    // join on Level2.level3_id -> Level3.id
    level3: Join<Level3>,
    text: String,
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    text: String,
}

fn populated_level() -> Level1 {
    let l3 = Level3 {
        id: 3,
        text: "level3".to_string(),
    };
    let l2 = Level2 {
        id: 2,
        level3: Join::with_entity(l3),
        text: "level2".to_string(),
    };

    Level1 {
        id: 1,
        level2: Join::with_entity(l2),
        text: "level1".to_string(),
    }
}

#[tokio::test]
#[traced_test("info")]
async fn load() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 1 + level2 + level3 (key join)
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2_level3.id, level1_level2_level3.text, level1_level2.text, level1.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2 \
                JOIN (Level3 level1_level2_level3) \
                ON (level1_level2.level3_id = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id AND level1.level2_level3_id = level1_level2.level3_id)"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn key_predicate() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let key3 = Level3Key::from(3);
    let key2 = Level2Key {
        id: 2,
        level3: key3,
    };

    let key1 = Level1Key {
        id: 1,
        level2: key2,
    };

    // Load level 1 + level2 + level3 (key join)
    let q = query!(Level1, "*,{}", key1);
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2_level3.id, level1_level2_level3.text, level1_level2.text, level1.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2 \
                JOIN (Level3 level1_level2_level3) \
                ON (level1_level2.level3_id = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id AND level1.level2_level3_id = level1_level2.level3_id) \
            WHERE (level1.id = 1 AND level1_level2.id = 2 AND level1_level2_level3.id = 3)"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn delete() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Delete without filter return no queries
    // This is for safety, otherwise everything would be deleted
    let q = query!(Level1, "*"); // Query contains no filter
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(toql.sqls_empty(), true);

    // Delete with filter on level1
    let q = query!(Level1, "id eq 4"); // Query contains filter
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "DELETE level1 FROM Level1 level1 WHERE level1.id = 4"
    );

    // Delete with filter on level 5
    let q = query!(Level1, "level2_level3_id eq 5");
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(),
           "DELETE level1 FROM Level1 level1 \
                JOIN (Level2 level1_level2 \
                    JOIN (Level3 level1_level2_level3) \
                    ON (level1_level2.level3_id = level1_level2_level3.id)) \
                    ON (level1.level2_id = level1_level2.id AND level1.level2_level3_id = level1_level2.level3_id) \
            WHERE level1_level2_level3.id = 5");
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
        "INSERT INTO Level1 (id, level2_id, level2_level3_id, text) VALUES (0, 0, 0, '')"
    );

    // insert path levels 1..5
    // this will only insert level 1 + 2,
    // level 3.. is skipped (unselected)
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());
    let mut sqls = toql.take_unsafe_sqls();
    sqls.sort();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level1 (id, level2_id, level2_level3_id, text) VALUES (0, 0, 0, '')",
            "INSERT INTO Level2 (id, level3_id, text) VALUES (0, 0, '')",
            "INSERT INTO Level3 (id, text) VALUES (0, '')",
        ]
    );

    // insert path levels 1..3
    // this will insert level 1..3
    let mut l = populated_level();
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());
    let mut sqls = toql.take_unsafe_sqls();
    sqls.sort();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level1 (id, level2_id, level2_level3_id, text) VALUES (1, 2, 3, 'level1')",
            "INSERT INTO Level2 (id, level3_id, text) VALUES (2, 3, 'level2')",
            "INSERT INTO Level3 (id, text) VALUES (3, 'level3')"
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

    // Update level 1 with invalid key
    // Nothing is updated
    let mut l1 = populated_level();
    l1.id = 0;
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(toql.sqls_empty(), true);

    // Update level 1 (text + foreign key)
    let mut l1 = populated_level();
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1' WHERE id = 1 AND level2_id = 2 AND level2_level3_id = 3"
    );

    // Update level 5 (text)
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level2 SET text = 'level2' WHERE id = 2 AND level3_id = 3"
    );

    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level3 SET text = 'level3' WHERE id = 3"
    );
}
