#[macro_export]
macro_rules! log_sql {
    ($s:expr, $p:expr) => {
       // $crate::tracing::info!("Unsafe SQL `{}`", $crate::sql::Sql($s.to_owned(), $p.to_owned()).unsafe_sql())
        //$crate::tracing::info!("SQL `{}` with params {:?}", $s, $p)
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$s, params= ?$p, "Querying Sql.");
    };
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql.to_unsafe_string(), "Querying Sql.");
        //$crate::tracing::info!("SQL `{}`", $s.to_unsafe_string())
    };
}

#[macro_export]
macro_rules! log_mut_sql {
    ($s:expr, $p:expr) => {
       // $crate::tracing::info!("Unsafe SQL `{}`", $crate::sql::Sql($s.to_owned(), $p.to_owned()).unsafe_sql())
        //$crate::tracing::info!("Mut SQL `{}` with params {:?}", $s, $p)
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$s, params = ?$p, "Executing Sql.");
    };
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql.to_unsafe_string(), "Executing Sql.");
        //$crate::tracing::info!("Mut SQL `{}`", $s.to_unsafe_string())
    };
}

#[macro_export]
macro_rules! log_literal_sql {
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql, "Querying Sql.");
        //$crate::tracing::info!("SQL `{}`", $s)
    };
}
#[macro_export]
macro_rules! log_mut_literal_sql {
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql, "Executing Sql.");
        //$crate::tracing::info!("Mut SQL `{}`", $s)
    };
}

#[macro_export]
macro_rules! log_toql {
    ($s:expr) => {
        $crate::tracing::info!("Toql `{}`", $s)
    };
}
