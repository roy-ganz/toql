#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RenameCase {
    CamelCase,
    SnakeCase,
    ShoutySnakeCase,
    MixedCase,
}

impl RenameCase {
    pub const VARIANTS: [&'static str; 4] =
        ["CamelCase", "snake_case", "SHOUTY_SNAKE_CASE", "mixedCase"];

    pub fn rename_str(&self, s: &str) -> String {
        use heck::{CamelCase, MixedCase, ShoutySnakeCase, SnakeCase};

        match self {
            RenameCase::CamelCase => s.to_camel_case(),
            RenameCase::SnakeCase => s.to_snake_case(),
            RenameCase::ShoutySnakeCase => s.to_shouty_snake_case(),
            RenameCase::MixedCase => s.to_mixed_case(),
        }
    }
}

impl std::str::FromStr for RenameCase {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "CamelCase" => RenameCase::CamelCase,
            "snake_case" => RenameCase::SnakeCase,
            "SHOUTY_SNAKE_CASE" => RenameCase::ShoutySnakeCase,
            "mixedCase" => RenameCase::MixedCase,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod test {
    use super::RenameCase;

    #[test]
    fn camelcase() {
        assert_eq!(RenameCase::CamelCase.rename_str("ABC"), "Abc");
        assert_eq!(RenameCase::CamelCase.rename_str("abc_def"), "AbcDef");
    }
    #[test]
    fn snakecase() {
        assert_eq!(RenameCase::SnakeCase.rename_str("ABC"), "abc");
        assert_eq!(RenameCase::SnakeCase.rename_str("abc_def"), "abc_def");
    }
    #[test]
    fn shouty_snakecase() {
        assert_eq!(RenameCase::ShoutySnakeCase.rename_str("ABC"), "ABC");
        assert_eq!(RenameCase::ShoutySnakeCase.rename_str("abc_def"), "ABC_DEF");
    }
    #[test]
    fn mixedcase() {
        assert_eq!(RenameCase::MixedCase.rename_str("ABC"), "abc");
        assert_eq!(RenameCase::MixedCase.rename_str("abc_def"), "abcDef");
    }

    #[test]
    fn from_str() {
        use std::str::FromStr;
        assert_eq!(
            RenameCase::from_str("CamelCase").unwrap(),
            RenameCase::CamelCase
        );
        assert_eq!(
            RenameCase::from_str("snake_case").unwrap(),
            RenameCase::SnakeCase
        );
        assert_eq!(
            RenameCase::from_str("SHOUTY_SNAKE_CASE").unwrap(),
            RenameCase::ShoutySnakeCase
        );
        assert_eq!(
            RenameCase::from_str("mixedCase").unwrap(),
            RenameCase::MixedCase
        );
        assert!(RenameCase::from_str("unknown_case").is_err());
    }
}
