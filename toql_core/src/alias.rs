


/// Desired alias format that the Sql builder uses.
/// Long -> concatenated paths -> user_address_country
/// Medium -> last path with index -> country1
/// Short -> first and last character withindex -> cy1
/// Tiny -> Letter t with number -> t1
#[derive(Clone, Debug)]
pub enum AliasFormat {
    TinyIndex,
    ShortIndex,
    MediumIndex,
    Canonical
}

impl AliasFormat {
    pub fn tiny_index( index: u16) -> String {
    let mut tiny_name= String::from("t");
    tiny_name.push_str(index.to_string().as_str());
    tiny_name
    }
    
    pub fn short_index(name: &str, index: u16) -> String {
         
        let mut it = name.chars();
        let f = it.next().unwrap_or('t');
        let mut short_name = String::new();
        short_name.push(f);
        let l = it.last();
        if let Some(lv) = l{
            short_name.push(lv);
            if lv.is_ascii_digit() {
                short_name.push('_');
            }
        }
        short_name.push_str(index.to_string().as_str());
       short_name
    }
    pub fn medium_index(name: &str, index: u16) -> String {
        let mut wrap = false;
        let mut medium_name = String::from(
                    if name.is_empty() {"t"} else {
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

