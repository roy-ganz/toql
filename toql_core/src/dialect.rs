
/// Generic SQL Dialect. Helper struct for internal use only.
/// The Toql derive implements for this struct SQL statements that are common among different SQL dialects.
/// Currently Sql for update and insert.
/// Explicit dialects (MySql, Postgres, etc.) can call the implemented Traits on this struct
pub struct Generic;

