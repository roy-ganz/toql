#[macro_export]
macro_rules! log_sql {
    ($s:expr, $p:expr) => {
       // $crate::tracing::info!("Unsafe SQL `{}`", $crate::sql::Sql($s.to_owned(), $p.to_owned()).unsafe_sql())
        //$crate::tracing::info!("SQL `{}` with params {:?}", $s, $p)
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$s, params= ?$p, "Db query.");
    };
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql.to_unsafe_string(), "Db query.");
        //$crate::tracing::info!("SQL `{}`", $s.to_unsafe_string())
    };
}

#[macro_export]
macro_rules! log_mut_sql {
    ($s:expr, $p:expr) => {
       // $crate::tracing::info!("Unsafe SQL `{}`", $crate::sql::Sql($s.to_owned(), $p.to_owned()).unsafe_sql())
        //$crate::tracing::info!("Mut SQL `{}` with params {:?}", $s, $p)
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$s, params = ?$p, "Db exec.");
    };
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql.to_unsafe_string(), "Db exec.");
        //$crate::tracing::info!("Mut SQL `{}`", $s.to_unsafe_string())
    };
}

#[macro_export]
macro_rules! log_literal_sql {
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql, "Db query.");
        //$crate::tracing::info!("SQL `{}`", $s)
    };
}
#[macro_export]
macro_rules! log_mut_literal_sql {
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql, "Db exec.");
        //$crate::tracing::info!("Mut SQL `{}`", $s)
    };
}

#[macro_export]
macro_rules! log_toql {
    ($s:expr) => {
        $crate::tracing::info!("Toql `{}`", $s)
    };
}
