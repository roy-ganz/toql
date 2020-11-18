use super::{QueryToken, concatenation::Concatenation};

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
            name : path.into()
        }
    }
}
    

impl Into<QueryToken> for Selection {
    fn into(self) -> QueryToken {
        QueryToken::Selection(self)
    }
}



impl ToString for Selection {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push('#');
        s.push_str(&self.name);
        s
    }
}