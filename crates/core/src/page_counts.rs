//! Return type of [load_page](crate::toql_api::ToqlApi::load_page) method.

/// Keeps page count information.
/// See [load_page](crate::toql_api::ToqlApi::load_page) for details.

#[derive(Debug)]
pub struct PageCounts {
    /// The number of filtered rows.
    pub filtered: u64,

    /// The number of total rows.
    pub total: u64,
}
