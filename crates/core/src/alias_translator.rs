//! Translate a canonical table alias into another [AliasFormat]() format.
//!

use crate::alias_format::AliasFormat;
use std::collections::HashMap;

/// The translator keeps track of alias numbering and earlier translations to ensure that no collisions happen.
/// That means all [SQL expressions](crate::sql_expr::SqlExpr) must be translated with the same instance,
/// typically using [Resolver](crate::sql_expr::resolver::Resolver).
pub struct AliasTranslator {
    format: AliasFormat,
    table_index: u16,
    translations: HashMap<String, String>, // canonical alias to translated alias
}

impl AliasTranslator {
    pub fn new(format: AliasFormat) -> Self {
        AliasTranslator {
            format,
            table_index: 0,
            translations: HashMap::new(),
        }
    }

    /// Translates a canonical sql alias into a shorter alias
    pub fn translate(&mut self, canonical_alias: &str) -> String {
        use std::collections::hash_map::Entry;

        let a = match self.translations.entry(canonical_alias.to_owned()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let alias = match self.format {
                    AliasFormat::TinyIndex => {
                        self.table_index += 1;
                        AliasFormat::tiny_index(self.table_index)
                    }
                    AliasFormat::ShortIndex => {
                        self.table_index += 1;
                        AliasFormat::short_index(&canonical_alias, self.table_index)
                    }
                    AliasFormat::MediumIndex => {
                        self.table_index += 1;
                        AliasFormat::medium_index(&canonical_alias, self.table_index)
                    }
                    _ => canonical_alias.to_owned(),
                };
                v.insert(alias)
            }
        }
        .to_owned();

        a
    }
}

#[cfg(test)]
mod test {
    use super::AliasTranslator;
    use crate::alias_format::AliasFormat;

    #[test]
    fn translate_canonical() {
        let mut t = AliasTranslator::new(AliasFormat::Canonical);

        assert_eq!(t.translate("level1_level2_level3"), "level1_level2_level3");
        assert_eq!(t.translate("level1_level2"), "level1_level2");

        // Repeated translation must yield same alias
        assert_eq!(t.translate("level1_level2_level3"), "level1_level2_level3");
    }

    #[test]
    fn translate_medium() {
        let mut t = AliasTranslator::new(AliasFormat::MediumIndex);

        assert_eq!(t.translate("level1_level2_level3"), "level3_1");
        assert_eq!(t.translate("level1_level2"), "level2_2");

        // Repeated translation must yield same alias
        assert_eq!(t.translate("level1_level2_level3"), "level3_1");
    }
    #[test]
    fn translate_short() {
        let mut t = AliasTranslator::new(AliasFormat::ShortIndex);

        assert_eq!(t.translate("level1_level2_level3"), "le1");
        assert_eq!(t.translate("level1_level2"), "le2");

        // Repeated translation must yield same alias
        assert_eq!(t.translate("level1_level2_level3"), "le1");
    }
    #[test]
    fn translate_tiny() {
        let mut t = AliasTranslator::new(AliasFormat::TinyIndex);

        assert_eq!(t.translate("level1_level2_level3"), "t1");
        assert_eq!(t.translate("level1_level2"), "t2");

        // Repeated translation must yield same alias
        assert_eq!(t.translate("level1_level2_level3"), "t1");
    }
}
