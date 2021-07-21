
#[macro_export]
macro_rules! join {
    ($x: expr) => {
        toql::try_join::TryJoin::try_join_or($x, toql::none_error!())?
    };
}
#[macro_export]
macro_rules! rjoin {
    ($x: expr) => {
        toql::try_join::TryJoin::try_join_or($x.as_ref(), toql::none_error!())?
    };
}