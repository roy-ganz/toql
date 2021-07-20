pub struct Fields {
    pub list: Vec<String>,
}

impl Fields {
    pub fn top() -> Self {
        Self::from(vec!["*".to_string()])
    }

    pub fn from(fields: Vec<String>) -> Self {
        Fields { list: fields }
    }

    pub fn into_inner(self) -> Vec<String> {
        self.list
    }
}
