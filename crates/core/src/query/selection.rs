use super::concatenation::Concatenation;

#[derive(Clone, Debug)]
pub struct Selection {
    pub(crate) concatenation: Concatenation,
    pub(crate) name: String,
}

#[derive(Clone, Debug)]
pub struct SelectionPool<'a> {
    pub(crate) selections: &'a [&'a Selection],
}

impl Selection {
    pub fn from<T>(path: T) -> Self
    where
        T: Into<String>,
    {
        Selection {
            concatenation: Concatenation::And,
            name: path.into(),
        }
    }

    pub fn into_string(self) -> String {
        self.name
    }
}

impl ToString for Selection {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push('$');
        s.push_str(&self.name);
        s
    }
}

#[cfg(test)]
mod test {
    use super::Selection;

    #[test]
    fn build() {
        assert_eq!(Selection::from("sel").to_string(), "$sel");
        assert_eq!(Selection::from("level2_sel").to_string(), "$level2_sel");
    }
}
