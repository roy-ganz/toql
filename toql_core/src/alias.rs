//! Alias formatter.
//!
//! AliasFormat determines how the [SqlMapper](../sql_mapper/struct.SqlMapper.html) formats table aliases in SQL code. 
//! Table aliases are needed for every column. E.g. in  `country.id` the table alias is `country` and `ìd` is the column.
//! 
//! There are 4 different formats supported:
//!  - Canonical, this is the internal default alias format. 
//!    It is the most verbose and useful for debugging, it can however blow SQL state up in size.
//!  - TinyIndex, the shortes possible alias, not human friendly, but useful for production as it's fast for the database to parse.
//!  - ShortIndex and MediumIndex, readable and fast to parse. 
//! 


/// Represents the table alias format
#[derive(Clone, Debug)]
pub enum AliasFormat {
    /// Letter t plus number (t1)
    TinyIndex,
    /// First 2 characters plus number (co1)
    ShortIndex,
    /// Table name plus number (country1)
    MediumIndex,
    /// Full path (user_address_country)
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
