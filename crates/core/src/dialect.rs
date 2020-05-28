/// Generic SQL Dialect. Helper struct for internal use only.
/// The Toql derive implements for this struct SQL statements that are common among different SQL dialects.
/// 
/// Currently SQL for update and insert is implemented.
/// Explicit dialects (MySql, Postgres, etc.) can call the implemented traits on this struct
pub struct Generic;
