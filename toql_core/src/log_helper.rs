#[macro_export]
macro_rules! log_sql {
    ($s:expr, $p:expr) => {
        $crate::log::info!("Unsafe SQL `{}`", $crate::sql::unsafe_sql(&$s, &$p))
        //$crate::log::info!("SQL `{}` with params {:?}", $s, $p)
    };
    ($s:expr) => {
        $crate::log::info!("SQL `{}`", $s)
    };
}

#[macro_export]
macro_rules! log_toql {
    ($s:expr) => {
        $crate::log::info!("Toql `{}`", $s)
    };
}
