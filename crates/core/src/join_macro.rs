
#[macro_export]
macro_rules! join {
    ($x: expr) => {
        $x.try_join_or(toql::none_error!())?
    };
}
#[macro_export]
macro_rules! rjoin {
    ($x: expr) => {
        $x.as_ref().try_join_or(toql::none_error!())?
    };
}