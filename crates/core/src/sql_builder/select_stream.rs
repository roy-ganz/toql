//! Deserializing selected columns into struct fields.

/// Hold select state about a struct field.
#[derive(Debug, PartialEq)]
pub enum Select {
    /// Field is selected in the query
    Query,
    /// Field is always selected (preselection)
    Preselect,
    /// Field is not selected
    None,
}

impl Select {
    /// Return true, if field is selected
    pub fn is_selected(&self) -> bool {
        self != &Select::None
    }
}

/// SelectStream memorizes which fields and joined structs are selected.
///
/// It is needed for the deserialization trait [FromRow](crate::from_row::FromRow).
/// The selections can either come from the query or the preselections in the mapping.
/// The number of selections does not correspond with the number of selected columns or expressions, because
/// each join gets an additional selection. For the number of columns take (BuildResult::column_counter)[crate::sql_builder::build_result::BuildResult::column_counter].
#[derive(Debug, PartialEq)]
pub struct SelectStream {
    stream: Vec<Select>,
}

impl SelectStream {
    /// Create new stream.
    pub fn new() -> Self {
        Self { stream: Vec::new() }
    }
    /// Add select state at the end of the stream.
    pub fn push(&mut self, selection: Select) {
        self.stream.push(selection);
    }
    /// Return iterator to the stream.
    pub fn iter(&self) -> std::slice::Iter<'_, Select> {
        self.stream.iter()
    }
}

impl Default for SelectStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::SelectStream;

    #[test]
    fn default() {
        assert_eq!(SelectStream::new(), SelectStream::default())
    }
}
