use toql::mock_db::MockDb;
use toql::prelude::{fields, paths, query, Cache, Join, Toql, ToqlApi};
use tracing_test::traced_test;
//
#[derive(Debug, Default, Toql)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join, preselect)] // Default mapping
    level2: Option<Level2>, // Preselected left join
}
#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join(columns(self = "level_3", other = "id")))] // Specified columns
    level3: Option<Option<Level3>>, // Selectable left join
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join(on_sql = "..text = 'ABC'"), preselect)] // Custom ON statement
    level4: Option<Join<Level4>>, // Preselected left join
}

#[derive(Debug, Default, Toql)]
pub struct Level4 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join)]
    level5: Option<Option<Join<Level5>>>, // Selectable left join
}

#[derive(Debug, Default, Toql)]
pub struct Level5 {
    #[toql(key)]
    id: u64,
    text: String,
}

fn populated_level() -> Level1 {
    let l5 = Level5 {
        id: 5,
        text: "level5".to_string(),
    };
    let l4 = Level4 {
        id: 4,
        text: "level4".to_string(),
        level5: Some(Some(Join::with_entity(l5))),
    };
    let l3 = Level3 {
        id: 3,
        text: "level3".to_string(),
        level4: Some(Join::with_entity(l4)),
    };
    let l2 = Level2 {
        id: 2,
        text: "level2".to_string(),
        level3: Some(Some(l3)),
    };

    Level1 {
        id: 1,
        text: "level1".to_string(),
        level2: Some(l2),
    }
}

#[tokio::test]
#[traced_test("info")]
async fn load() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 1 + preselected level 2
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text, level1_level2.id, level1_level2.text \
             FROM Level1 level1 \
             LEFT JOIN (Level2 level1_level2) \
             ON (level1.level2_id = level1_level2.id)"
    );

    // Load preselects from level 1..4 and fields from level 5
    let q = query!(Level1, "level2_level3_level4_level5_*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(),
           "SELECT level1.id, level1.text, \
           level1_level2.id, level1_level2.text, \
           level1_level2_level3.id, level1_level2_level3.text, \
           level1_level2_level3_level4.id, level1_level2_level3_level4.text, \
           level1_level2_level3_level4_level5.id, level1_level2_level3_level4_level5.text \
           FROM Level1 level1 \
           LEFT JOIN (Level2 level1_level2 \
                LEFT JOIN (Level3 level1_level2_level3 \
                    LEFT JOIN (Level4 level1_level2_level3_level4 \
                        LEFT JOIN (Level5 level1_level2_level3_level4_level5) \
                        ON (level1_level2_level3_level4.level5_id = level1_level2_level3_level4_level5.id)) \
                    ON (level1_level2_level3.level4_id = level1_level2_level3_level4.id AND level1_level2_level3.text = 'ABC')) \
                ON (level1_level2.level_3 = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id)");
}

#[tokio::test]
#[traced_test("info")]
async fn count() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Count with joined level 1 + preselected level 2
    let q = query!(Level1, "*"); // Query contains no filter
    assert!(toql.count(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(), "SELECT COUNT(*) FROM Level1 level1");

    let q = query!(Level1, "id eq 4"); // Query contains filter
    assert!(toql.count(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT COUNT(*) FROM Level1 level1 WHERE level1.id = 4"
    );

    // Load preselects from level 1..4 and fields from level 5
    // Left joins are converted into inner joins
    let q = query!(Level1, "level2_level3_level4_level5_id eq 5");
    assert!(toql.count(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(),
           "SELECT COUNT(*) \
           FROM Level1 level1 \
           JOIN (Level2 level1_level2 \
                JOIN (Level3 level1_level2_level3 \
                    JOIN (Level4 level1_level2_level3_level4 \
                        JOIN (Level5 level1_level2_level3_level4_level5) \
                        ON (level1_level2_level3_level4.level5_id = level1_level2_level3_level4_level5.id)) \
                    ON (level1_level2_level3.level4_id = level1_level2_level3_level4.id AND level1_level2_level3.text = 'ABC')) \
                ON (level1_level2.level_3 = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id) \
            WHERE level1_level2_level3_level4_level5.id = 5");
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
    // Left joins are converted into inner joins
    let q = query!(Level1, "level2_level3_level4_level5_id eq 5");
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(),
           "DELETE level1 \
           FROM Level1 level1 \
           JOIN (Level2 level1_level2 \
                JOIN (Level3 level1_level2_level3 \
                    JOIN (Level4 level1_level2_level3_level4 \
                        JOIN (Level5 level1_level2_level3_level4_level5) \
                        ON (level1_level2_level3_level4.level5_id = level1_level2_level3_level4_level5.id)) \
                    ON (level1_level2_level3.level4_id = level1_level2_level3_level4.id AND level1_level2_level3.text = 'ABC')) \
                ON (level1_level2.level_3 = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id) \
            WHERE level1_level2_level3_level4_level5.id = 5");
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
        "INSERT INTO Level1 (id, text, level2_id) VALUES (0, '', NULL)"
    );

    // insert path levels 1..5
    // this will only insert level 1,
    // level 2.. is skipped (unselected)
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3_level4_level5"))
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        ["INSERT INTO Level1 (id, text, level2_id) VALUES (0, '', NULL)"]
    );

    // insert path levels 1..5
    // this will insert level 1..5
    let mut l = populated_level();
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3_level4_level5"))
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level5 (id, text) VALUES (5, 'level5')",
            "INSERT INTO Level4 (id, text, level5_id) VALUES (4, 'level4', 5)",
            "INSERT INTO Level3 (id, text, level4_id) VALUES (3, 'level3', 4)",
            "INSERT INTO Level2 (id, text, level_3) VALUES (2, 'level2', 3)",
            "INSERT INTO Level1 (id, text, level2_id) VALUES (1, 'level1', 2)"
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn insert2() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = Level4 {
        id: 4,
        text: "level4".to_string(),
        level5: Some(Some(Join::with_key(5))),
    };

    // insert level 4 with joined key
    assert!(toql.insert_one(&mut l, paths!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level4 (id, text, level5_id) VALUES (4, 'level4', 5)"
    );

    // Insert level 4 + 5
    // -> only Level 4 is inserted, because level 5 is empty
    assert!(toql
        .insert_one(&mut l, paths!(Level4, "level5"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level4 (id, text, level5_id) VALUES (4, 'level4', 5)"
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
        "UPDATE Level1 SET text = 'level1', level2_id = 2 WHERE id = 1"
    );

    // Update level 5 (text)
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_level4_level5_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level5 SET text = 'level5' WHERE id = 5"
    );

    // Update level 1 - 5
    let mut l1 = populated_level();
    assert!(toql
        .update_one(
            &mut l1,
            fields!(
                Level1,
                "*, level2_*, \
            level2_level3_*, level2_level3_level4_*,\
            level2_level3_level4_level5_*"
            ),
        )
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "UPDATE Level1 SET text = \'level1\', level2_id = 2 WHERE id = 1",
            "UPDATE Level2 SET text = \'level2\', level_3 = 3 WHERE id = 2",
            "UPDATE Level3 SET text = \'level3\', level4_id = 4 WHERE id = 3",
            "UPDATE Level4 SET text = \'level4\', level5_id = 5 WHERE id = 4",
            "UPDATE Level5 SET text = \'level5\' WHERE id = 5"
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn update2() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = Level4 {
        id: 4,
        text: "level4".to_string(),
        level5: Some(Some(Join::with_key(5))),
    };

    // Update level 4 with joined key
    assert!(toql.update_one(&mut l, fields!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level4 SET text = \'level4\', level5_id = 5 WHERE id = 4"
    );

    // Update level 4 + 5
    // -> No update statement because join does not contain value
    assert!(toql
        .update_one(&mut l, fields!(Level4, "level5_*"))
        .await
        .is_ok());
    assert!(toql.sqls_empty());
}
