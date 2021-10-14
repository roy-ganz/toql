#[derive(Debug, PartialEq)]
pub enum Select {
    Query,
    Preselect,
    None,
}

impl Select {
    pub fn is_selected(&self) -> bool {
        self != &Select::None
    }
}

/// SelectStream memorizes which columns and joins are selected and is needed for the deserialization trait FromRow.
/// The selections can either come from the query or the preselections from the mapping.
/// The number of selections does not correspond with the number of selected columns / expressions, because
/// each join gets an additional selection. For the number of columns take `column_counter` from  `BuildResult`.
#[derive(Debug)]
pub struct SelectStream {
    stream: Vec<Select>,
}

impl SelectStream {
    pub fn new() -> Self {
        Self { stream: Vec::new() }
    }
    pub fn change(&mut self, pos: usize, select: Select) {
        if let Some(p) = self.stream.get_mut(pos) {
            *p = select;
        }
    }
    pub fn push(&mut self, selection: Select) {
        self.stream.push(selection);
    }
    pub fn len(&mut self) -> usize {
        self.stream.len()
    }
    pub fn is_empty(&mut self) -> bool {
        self.stream.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Select> {
        self.stream.iter()
    }
}

impl Default for SelectStream {
    fn default() -> Self {
        Self::new()
    }
}
