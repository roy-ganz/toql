use super::{
    concatenation::Concatenation, field::Field, predicate::Predicate, selection::Selection,
    wildcard::Wildcard,
};

#[derive(Clone, Debug)]
pub(crate) enum QueryToken {
    LeftBracket(Concatenation),
    RightBracket,
    Wildcard(Wildcard),
    Field(Field),
    Predicate(Predicate),
    Selection(Selection),
}

impl From<&str> for QueryToken {
    fn from(s: &str) -> QueryToken {
        if s.ends_with('*') {
            QueryToken::Wildcard(Wildcard::from(s))
        } else {
            QueryToken::Field(Field::from(s))
        }
    }
}

impl From<Field> for QueryToken {
    fn from(field: Field) -> QueryToken {
        QueryToken::Field(field)
    }
}

impl From<Predicate> for QueryToken {
    fn from(predicate: Predicate) -> QueryToken {
        QueryToken::Predicate(predicate)
    }
}

impl From<Selection> for QueryToken {
    fn from(selection: Selection) -> QueryToken {
        QueryToken::Selection(selection)
    }
}

impl ToString for QueryToken {
    fn to_string(&self) -> String {
        match self {
            QueryToken::RightBracket => String::from(")"),
            QueryToken::LeftBracket(c) => match c {
                Concatenation::And => String::from("("),
                Concatenation::Or => String::from("("),
            },
            QueryToken::Field(field) => field.to_string(),
            QueryToken::Predicate(predicate) => predicate.to_string(),
            QueryToken::Selection(selection) => format!("${}", &selection.name),
            QueryToken::Wildcard(wildcard) => format!("{}*", wildcard.path),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Concatenation, Field, Predicate, QueryToken, Selection, Wildcard};

    #[test]
    fn to_string() {
        assert_eq!(QueryToken::RightBracket.to_string(), ")");
        assert_eq!(QueryToken::LeftBracket(Concatenation::And).to_string(), "(");
        assert_eq!(QueryToken::LeftBracket(Concatenation::Or).to_string(), "(");
        assert_eq!(QueryToken::Field(Field::from("prop")).to_string(), "prop");
        assert_eq!(
            QueryToken::Predicate(Predicate::from("search")).to_string(),
            "@search"
        );
        assert_eq!(
            QueryToken::Selection(Selection::from("std")).to_string(),
            "$std"
        );
        assert_eq!(
            QueryToken::Wildcard(Wildcard::from("level1")).to_string(),
            "level1_*"
        );
    }
}
