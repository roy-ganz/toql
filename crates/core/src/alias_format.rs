//! Alias format for table aliases.


/// This determines how the [AliasTranslator](crate::alias_translator::AliasTranslator) formats table aliases in SQL code.
/// Toql uses table aliases for every column. 
/// E.g. in  `user_address_country.id` the table alias is `user_address_country` and `id` is the column.
///
/// There are 4 different formats to cater for development and production environments:
///  - [Canonical](AliasFormat::Canonical), this is the internal and default alias. 
///    It is the most verbose and useful for debugging, it can however heavily blow up SQL statements. Example: `user_address_country`.
///  - [MediumIndex](AliasFormat::MediumIndex), last part of the canonical alias plus number: `country1`
///  - [ShortIndex](AliasFormat::ShortIndex), first 2 characters of the last part of the canonical alias plus number: `co1`
///  - [TinyIndex](AliasFormat::TinyIndex) the shortest possible alias, not human friendly, but useful for production as it's fast for databases to parse. 
///    Its made up of the letter t plus a number:`t1`
#[derive(Clone, Debug)]
pub enum AliasFormat {
    /// Letter _t_ plus number
    TinyIndex,
    /// First 2 characters of last canonical path node plus number 
    ShortIndex,
    /// Last canonical path node plus number
    MediumIndex,
    /// Full canonical path
    Canonical,
}

impl AliasFormat {
    /// Creates a tiny alias from an index number
    pub fn tiny_index(index: u16) -> String {
        let mut tiny_name = String::from("t");
        tiny_name.push_str(index.to_string().as_str());
        tiny_name
    }

    /// Creates a short alias from a name and an index number.
    /// If the name ends with a number an underscore is added to separate it from the index.
    pub fn short_index(name: &str, index: u16) -> String {
        let medium_name = String::from(if name.is_empty() {
            "t"
        } else {
            let x = name.rfind('_');
            if let Some(xi) = x {
                &name[(xi + 1)..]
            } else {
                name
            }
        });

        let mut it = medium_name.chars();
        let f = it.next().unwrap_or('t');
        let mut short_name = String::new();
        short_name.push(f);
        let s = it.next();
        if let Some(sv) = s {
            short_name.push(sv);
            if sv.is_ascii_digit() {
                short_name.push('_');
            }
        }
        short_name.push_str(index.to_string().as_str());
        short_name
    }
    /// Creates a medium alias from a name and an index number.
    /// If the name ends with a number an underscore is added to separate it from the index.
    pub fn medium_index(name: &str, index: u16) -> String {
        let mut wrap = false;
        let mut medium_name = String::from(if name.is_empty() {
            "t"
        } else {
            let x = name.rfind('_');
            if let Some(xi) = x {
                &name[(xi + 1)..]
            } else {
                name
            }
        });
        let c = medium_name.chars().last().unwrap();
        if c.is_ascii_digit() {
            medium_name.push('$');
            wrap = true;
        }
        medium_name.push_str(index.to_string().as_str());

        if wrap {
            format!("`{}`", medium_name)
        } else {
            medium_name
        }
    }
}
