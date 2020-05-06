
pub(crate) struct SqlTargetData {
    pub(crate) selected: bool, // Target is selected
    pub(crate) used: bool,     // Target is either selected or filtered
}

impl Default for SqlTargetData {
    fn default() -> SqlTargetData {
        SqlTargetData {
            used: false,
            selected: false,
        }
    }
}
