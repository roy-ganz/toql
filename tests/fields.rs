use toql::{
    mock_db::MockDb,
    prelude::{
        query, sql_expr, Cache, FieldFilter, FieldHandler, ParameterMap, SqlArg, SqlBuilderError,
        SqlExpr, Toql, ToqlApi,
    },
    row,
};
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(handler = "get_field_handler")] // Inject into join predicate
pub struct Level1 {
    #[toql(key)]
    id: u64,
    r#text1: Option<String>,

    #[toql(skip_wildcard)]
    text2: Option<String>,
}

struct MyFieldHandler {
    basic: toql::prelude::DefaultFieldHandler,
}

impl FieldHandler for MyFieldHandler {
    fn build_select(
        &self,
        select: SqlExpr,
        aux_params: &ParameterMap<'_>,
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        self.basic.build_select(select, aux_params)
    }
    fn build_filter(
        &self,
        select: SqlExpr,
        filter: &FieldFilter,
        aux_params: &ParameterMap<'_>,
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        match filter {
            FieldFilter::Fn(name, args) if name == "LK" && args.len() == 1 => Ok(Some(sql_expr!(
                "{} LIKE ?",
                select,
                args.get(0).map_or(SqlArg::Null, |a| a.to_owned())
            ))),
            _ => self.basic.build_filter(select, filter, aux_params),
        }
    }
}
fn get_field_handler() -> MyFieldHandler {
    MyFieldHandler {
        basic: toql::prelude::DefaultFieldHandler::default(),
    }
}

#[tokio::test]
#[traced_test("info")]
async fn skip_wildcard() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load text1 
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1"
    );
    let q = query!(Level1, "text2");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text2 FROM Level1 level1"
    );
}
#[tokio::test]
#[traced_test("info")]
async fn filter() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load text1 with custom field handler
    let q = query!(Level1, "text1 FN LK 'ABC'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 LIKE 'ABC'"
    );

    let q = query!(Level1, "text1 EQ 'ABC'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 = 'ABC'"
    );

    let q = query!(Level1, "text1 EQN");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 IS NULL"
    );

    let q = query!(Level1, "text1 NE 'ABC'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 <> 'ABC'"
    );

    let q = query!(Level1, "text1 NEN");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 IS NOT NULL"
    );

    let q = query!(Level1, "text1 GT 'C'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 > 'C'"
    );
    let q = query!(Level1, "text1 GE 'C'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 >= 'C'"
    );

    let q = query!(Level1, "text1 LT 'C'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 < 'C'"
    );

    let q = query!(Level1, "text1 LE 'C'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 <= 'C'"
    );

    let q = query!(Level1, "text1 LK 'C%'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 LIKE 'C%'"
    );
    let q = query!(Level1, "text1 BW 'C' 'K'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 BETWEEN 'C' AND 'K'"
    );

    let q = query!(Level1, "text1 IN 'A' 'B' 'C'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 IN ('A', 'B', 'C')"
    );

    let q = query!(Level1, "text1 OUT 'A' 'B' 'C'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 WHERE level1.text1 NOT IN ('A', 'B', 'C')"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn concatenation() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load text1 with custom field handler
    let q = query!(Level1, "(text1 eq 'ABC';text1 eq 'DEF'),text1 eq 'GHI'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text1 FROM Level1 level1 \
            WHERE (level1.text1 = 'ABC' OR level1.text1 = 'DEF') AND level1.text1 = 'GHI'"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn order() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load text1 with custom field handler
    let q = query!(Level1, "+id");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id FROM Level1 level1 ORDER BY level1.id ASC"
    );

    let q = query!(Level1, "-id, +text1");
    let select =  "SELECT level1.id, level1.text1 FROM Level1 level1 ORDER BY level1.id DESC, level1.text1 ASC";
    toql.mock_rows(select, vec![row!(1u64, "level1")]);

    assert!(toql.load_one(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(), select);

    let q = query!(Level1, "-2id, +1text1");
    let select = "SELECT level1.id, level1.text1 FROM Level1 level1 ORDER BY level1.text1 ASC, level1.id DESC";
    toql.mock_rows(select, vec![row!(1u64, "level1")]);
    assert!(toql.load_one(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(), select);
}

