#[macro_export]
macro_rules! log_sql { 
    ($s:expr, $p:expr) => {  $crate::log::info!("SQL `{}` with params {:?}", $s, $p) };
    ($s:expr) => {  $crate::log::info!("SQL `{}`", $s) };
}

