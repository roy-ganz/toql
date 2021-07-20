pub struct Paths {
    pub list: Vec<String>,
}

impl Paths {
    pub fn top() -> Self {
        Self::from(vec![])
    }

    pub fn from(fields: Vec<String>) -> Self {
        Paths { list: fields }
    }
    pub fn into_inner(self) -> Vec<String> {
        self.list
    }
}
