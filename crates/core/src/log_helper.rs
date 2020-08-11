#[macro_export]
macro_rules! log_sql {
    ($s:expr, $p:expr) => {
       // $crate::log::info!("Unsafe SQL `{}`", $crate::sql::Sql($s.to_owned(), $p.to_owned()).unsafe_sql())
        $crate::log::info!("SQL `{}` with params {:?}", $s, $p)
    };
    ($s:expr) => {
        $crate::log::info!("Unsafe SQL `{}`", $s.unsafe_sql())
    };
}

#[macro_export]
macro_rules! log_toql {
    ($s:expr) => {
        $crate::log::info!("Toql `{}`", $s)
    };
}
