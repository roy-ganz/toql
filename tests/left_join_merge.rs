use toql::mock_db::MockDb;
use toql::prelude::{fields, paths, query, Cache, Join, Toql, ToqlApi};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: Option<String>,

    #[toql(join)]
    level2: Option<Option<Join<Level2>>>, // Selectable left join
}
#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    text: Option<String>,

    #[toql(merge)]
    level3: Option<Vec<Level3>>, // Selectable merge
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    #[toql(key)]
    level2_id: u64,
    text: Option<String>,
}

fn populated_level() -> Level1 {
    let l31 = Level3 {
        id: 31,
        level2_id: 2,
        text: Some("level31".to_string()),
    };
    let l32 = Level3 {
        id: 32,
        level2_id: 2,
        text: Some("level32".to_string()),
    };
    let l33 = Level3 {
        id: 33,
        level2_id: 0,
        text: Some("new level33".to_string()),
    };
    let l34 = Level3 {
        id: 34,
        level2_id: 0,
        text: Some("new level34".to_string()),
    };
    let l2 = Level2 {
        id: 2,
        text: Some("level2".to_string()),
        level3: Some(vec![l31, l32, l33, l34]),
    };

    Level1 {
        id: 1,
        text: Some("level1".to_string()),
        level2: Some(Some(Join::with_entity(l2))),
    }
}

fn populated_level2() -> Level1 {
    Level1 {
        id: 1,
        text: Some("level1".to_string()),
        level2: Some(Some(Join::with_key(Level2Key::from(2)))),
    }
}

#[tokio::test]
#[traced_test("info")]
async fn load() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 3
    let select1 = "SELECT level1.id, level1_level2.id \
                    FROM Level1 level1 \
                    LEFT JOIN (Level2 level1_level2) \
                    ON (level1.level2_id = level1_level2.id)";

    let select2 = "SELECT level1_level2.id, level3.id, level3.level2_id, level3.text \
                    FROM Level3 level3 \
                    JOIN Level2 level1_level2 \
                    ON (level1_level2.id = level3.level2_id AND level1_level2.id = 2)";

    let q = query!(Level1, "level2_level3_*");
    toql.mock_rows(select1, vec![row!(1u64, 2u64)]);
    toql.mock_rows(select2, vec![row!(2u64, 3u64, 2u64, "level3")]);
    assert!(toql.load_many(q).await.is_ok());

    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
}

#[tokio::test]
#[traced_test("info")]
async fn insert() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = Level1::default();

    // insert level 1..3
    // Will only insert level1 because left join is None
    // Does not insert id, because auto_key
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level1 (text, level2_id) VALUES (DEFAULT, DEFAULT)"
    );

    // insert path levels 1..3
    // this will insert level 1..3
    let mut l = populated_level();
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level2 (text) VALUES ('level2')",
            "INSERT INTO Level1 (text, level2_id) VALUES ('level1', 100)",
            "INSERT INTO Level3 (id, level2_id, text) VALUES \
        (31, 100, 'level31'), \
        (32, 100, 'level32'), \
        (33, 100, 'new level33'), \
        (34, 100, 'new level34')"
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

    // Update level 1 (text + foreign key)
    let mut l1 = populated_level();
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1', level2_id = 2 WHERE id = 1"
    );

    // Update level 3 (text)
    // This ignores new item in Vec
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "UPDATE Level3 SET text = 'level31' WHERE id = 31 AND level2_id = 2",
            "UPDATE Level3 SET text = 'level32' WHERE id = 32 AND level2_id = 2"
        ],
    );

    // Update level 1 - 3
    let mut l1 = populated_level();
    assert!(toql
        .update_one(
            &mut l1,
            fields!(
                Level1,
                "*, level2_*, \
            level2_level3_*"
            ),
        )
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "UPDATE Level3 SET text = \'level31\' WHERE id = 31 AND level2_id = 2",
            "UPDATE Level3 SET text = \'level32\' WHERE id = 32 AND level2_id = 2",
            "UPDATE Level2 SET text = \'level2\' WHERE id = 2",
            "UPDATE Level1 SET text = \'level1\', level2_id = 2 WHERE id = 1"
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn update2() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // resize vec of level2
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3"))
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
       ["DELETE level1_level2_level3 \
        FROM Level3 level1_level2_level3 \
        JOIN Level2 level1_level2 ON level1_level2.id = level1_level2_level3.level2_id \
        WHERE level1_level2.id = 2 AND NOT (level1_level2_level3.id = 31 AND level1_level2_level3.level2_id = 2 \
        OR level1_level2_level3.id = 32 AND level1_level2_level3.level2_id = 2)", 
        "INSERT INTO Level3 (id, level2_id, text) VALUES (33, 2, \'new level33\'), (34, 2, \'new level34\')"]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn update3() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Update level 1 (text + foreign key)
    let mut l1 = populated_level2();
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1', level2_id = 2 WHERE id = 1"
    );

    // Update level 3 (text)
    // This only updates level1, because join contains key
    let mut l1 = populated_level2();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_*"))
        .await
        .is_ok());
    assert!(toql.sqls_empty());
}

#[tokio::test]
#[traced_test("info")]
async fn insert2() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // insert path levels 1..3
    // this will insert level 1
    let mut l = populated_level2();
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level1 (text, level2_id) VALUES (\'level1\', 2)"
    );
}
