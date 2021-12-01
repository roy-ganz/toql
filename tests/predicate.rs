use toql::mock_db::MockDb;
use toql::predicate_handler::PredicateHandler;
use toql::prelude::{
    query, Cache, ResolverError, SqlArg, SqlBuilderError, Toql, ToqlApi, ToqlError,
};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(
    predicate(name = "pred", sql = "..predicate1 = ?"),
    predicate(name = "ap_pred", sql = "..ap_predicate1 = <aux_param>"),
    predicate(name = "h_pred", sql = "..h_predicate1 = 1", handler = "get_handler")
)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join)] // Default mapping
    level2: Option<Level2>, // Preselected inner join
}
#[derive(Debug, Default, Toql)]
#[toql(predicate(name = "pred", sql = "..predicate2 = ?"))]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    text: String,
    #[toql(merge)]
    level3: Option<Vec<Level3>>,
}

#[derive(Debug, Default, Toql)]
#[toql(predicate(name = "pred", sql = "..predicate3 = ?"))]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    text: String,
}

struct MyPredicateHandler;

impl PredicateHandler for MyPredicateHandler {
    fn build_predicate(
        &self,
        expression: toql::sql_expr::SqlExpr,
        args: &[SqlArg],
        _aux_params: &toql::parameter_map::ParameterMap,
    ) -> Result<Option<toql::sql_expr::SqlExpr>, toql::prelude::SqlBuilderError> {
        // Let handler respond according to predicate argument
        let a = args.get(0).map_or(2u64, |a| a.get_u64().unwrap_or(2));
        match a {
            0 => Ok(None),
            1 => Ok(Some(expression)),
            _ => Err(toql::prelude::SqlBuilderError::FilterInvalid(
                "expected 0 or 1".to_string(),
            )),
        }
    }
}

fn get_handler() -> MyPredicateHandler {
    MyPredicateHandler
}

#[tokio::test]
#[traced_test("info")]
async fn load() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Filter level1
    let q = query!(Level1, "@pred 5");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1 WHERE level1.predicate1 = 5"
    );
    // Predicate on  level1
    let q = query!(Level1, "@level2_pred 2");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text, level1_level2.id, level1_level2.text \
        FROM Level1 level1 \
        JOIN (Level2 level1_level2) ON (level1.level2_id = level1_level2.id) \
        WHERE level1_level2.predicate2 = 2"
    );

    // Predicate on level3 (merge)
    // Predicate is ignored, because level3 is a merge
    let q = query!(Level1, "@level2_level3_pred 3");
    toql.mock_rows(
        "SELECT level1.id, level1.text FROM Level1 level1",
        vec![row!(1u64, "level1")],
    );
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sqls(),
        ["SELECT level1.id, level1.text FROM Level1 level1"]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn load2() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Filter level1 with missing aux param
    // -> Fails
    let q = query!(Level1, "@apPred");
    let err = toql.load_many(&q).await.err().unwrap();
    println!("{:?}", err);
    assert_eq!(
        err.to_string(),
        ToqlError::SqlExprResolverError(ResolverError::AuxParamMissing("aux_param".to_string()))
            .to_string()
    );

    let q = q.aux_param("aux_param", 42);
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1 WHERE level1.ap_predicate1 = 42"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn load3() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Handler raises error
    let q = query!(Level1, "@hPred");
    let err = toql.load_many(&q).await.err().unwrap();

    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::FilterInvalid(
            "expected 0 or 1".to_string()
        ))
        .to_string()
    );

    // Handler raises error
    let q = query!(Level1, "@hPred 2");
    let err = toql.load_many(q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::FilterInvalid(
            "expected 0 or 1".to_string()
        ))
        .to_string()
    );

    // Handler filters predicate
    let q = query!(Level1, "@hPred 1");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1 WHERE level1.h_predicate1 = 1"
    );

    // Handler ommits filter
    let q = query!(Level1, "@hPred 0");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1"
    );
}
