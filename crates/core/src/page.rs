/// [Page](enum.Page.html) is used as an argument in load functions. It tells Toql to build and run an additional query
/// to count the total number of records.
///
/// In Toql there are 2 types of counts: A filtered count and a total count. Lets take a datagrid where the user searches his contacts with a name starting with 'Alice'.
/// The datagrid would show the following:
///  - Total number of contacts (Total count)
///  - Number of found contacts with the name 'Alice' (Filtered count)
///
/// While the filtered count is almost for free and returned for every query,
/// the total count needs a seperate query with a different SQL filter predicate.
/// Toql can do that out of the box, but the fields must be mapped accordingly in the [SqlMapper](../sql_mapper/struct.SqlMapper.html)
pub enum Page {
    /// Retrieve filtered count only.
    /// Argments are *start index* and *number of records*.
    Uncounted(u64, u16),
    // Retrieve filtered count and total count.
    /// Argments are *start index* and *number of records*.
    Counted(u64, u16),
}
