/// Macros for backends to log SQL statements and TOQL queries in a common way
#[macro_export]
macro_rules! log_sql {
    ($s:expr, $p:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$s, params= ?$p, "Db query.");
    };
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql.to_unsafe_string(), "Db query.");
    };
}

#[macro_export]
macro_rules! log_mut_sql {
    ($s:expr, $p:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$s, params = ?$p, "Db exec.");
    };
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql.to_unsafe_string(), "Db exec.");
    };
}

#[macro_export]
macro_rules! log_literal_sql {
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql, "Db query.");
    };
}
#[macro_export]
macro_rules! log_mut_literal_sql {
    ($sql:expr) => {
        $crate::tracing::event!($crate::tracing::Level::INFO, sql =  %$sql, "Db exec.");
    };
}

#[macro_export]
macro_rules! log_toql {
    ($s:expr) => {
        $crate::tracing::info!("Toql `{}`", $s)
    };
}
