use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct FieldPath<'a>(Cow<'a, str>);

impl<'a> Default for FieldPath<'a> {
    fn default() -> Self {
        FieldPath(Cow::Owned(String::from("")))
    }
}

impl<'a> FieldPath<'a> {
    pub fn split_basename(path_with_basename: &str) -> (&str, Option<FieldPath>) {
        if let Some(pos) = path_with_basename.rfind('_') {
            (
                &path_with_basename[pos + 1..],
                Some(FieldPath::from(&path_with_basename[..pos])),
            )
        } else {
            (path_with_basename, None)
        }
    }

    pub fn from(path: &'a str) -> Self {
        FieldPath(Cow::Borrowed(path))
    }

    pub fn prepend(&self, head: &'a str) -> Self {
        let path = format!(
            "{}{}{}",
            head,
            if self.0.is_empty() { "" } else { "_" },
            self.0.as_ref()
        );

        FieldPath(Cow::Owned(path))
    }
    pub fn append(&self, tail: &'a str) -> Self {
        let path = format!(
            "{}{}{}",
            self.0.as_ref(),
            if self.0.is_empty() { "" } else { "_" },
            tail,
        );

        FieldPath(Cow::Owned(path))
    }

    pub fn relative_path(&self, root_path: &str) -> Option<FieldPath> {
        if self.0.starts_with(root_path) {
            let t = self.0.trim_start_matches(root_path).trim_start_matches("_");
            Some(FieldPath::from(t))
        } else {
            None
        }
    }

    pub fn ancestors(&self) -> Ancestor {
        Ancestor {
            pos: self.0.len(),
            path: self.0.as_ref(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn ancestor(&self) -> Option<FieldPath> {
        if let Some(pos) = self.0.rfind('_') {
            Some(FieldPath::from(&self.0[..pos]))
        } else {
            None
        }
    }

    /// True, if path is relative to root path
    pub fn relative(&self, root_path: &str) -> bool {
        self.0.starts_with(root_path)
    }

    /// True, if path is immediate child of root path
    pub fn child(&self, root_path: &str) -> bool {
        let relative_path = root_path.trim_start_matches(self.0.as_ref());
        !relative_path.contains('_')
    }

    pub fn descendents(&self) -> Descendents {
        Descendents {
            pos: 0,
            path: self.0.as_ref(),
        }
    }
    pub fn parents(&self) -> Parents {
        Parents {
            pos: self.0.len(),
            path: self.0.as_ref(),
        }
    }

    // Iterator
    pub fn children(&self) -> Children {
        Children {
            pos: 0,
            path: self.0.as_ref(),
        }
    }

    pub fn step(&self) -> Step {
        Step {
            pos: 0,
            path: self.0.as_ref(),
        }
    }
}

pub struct Step<'a> {
    pos: usize,
    path: &'a str,
}

/// Iterator to step down
/// Field without path has no descendents.
/// user_address_country_id -> user, user_address, user_address_country, user_address_country_id
impl<'a> Iterator for Step<'a> {
    type Item = FieldPath<'a>;
    fn next(&mut self) -> Option<FieldPath<'a>> {
        let p = self.path[self.pos..].find('_');

        match p {
            Some(i) => Some((FieldPath::from(&self.path[..i]), self.pos = i + 1).0),
            None if self.pos != self.path.len() => {
                Some((FieldPath::from(&self.path), self.pos = self.path.len()).0)
            }
            _ => None,
        }
    }
}

pub struct Ancestor<'a> {
    pos: usize,
    path: &'a str,
}

/// Iterator to yield ancestors
/// Field without path has no descendents.
/// user_address_country_id -> user_address_country_id, user_address_country, user_address, user
impl<'a> Iterator for Ancestor<'a> {
    type Item = FieldPath<'a>;
    fn next(&mut self) -> Option<FieldPath<'a>> {
        let p = self.path[0..self.pos].rfind('_');
        match p {
            Some(i) => Some((FieldPath::from(&self.path[..self.pos]), self.pos = i).0),
            None if self.pos != 0 => {
                (Some(FieldPath::from(&self.path[..self.pos])), self.pos = 0).0
            }
            _ => None,
        }
    }
}

/// Iterator to yield descendents
/// Field without path has no descendents.
/// user_address_country_id -> user_address_country_id, address_country_id, country_id, id
pub struct Descendents<'a> {
    pos: usize,
    path: &'a str,
}

impl<'a> Descendents<'a> {
    pub fn is_last(&self) -> bool {
        self.pos == self.path.len()
    }
}

impl<'a> Iterator for Descendents<'a> {
    type Item = FieldPath<'a>;
    fn next(&mut self) -> Option<FieldPath<'a>> {
        let p = self.path[self.pos..].find('_');
        match p {
            Some(i) => {
                (
                    Some(FieldPath::from(&self.path[self.pos..i])),
                    self.pos = i + 1,
                )
                    .0
            }
            None if self.pos != self.path.len() => {
                (
                    Some(FieldPath::from(&self.path[self.pos..])),
                    self.pos = self.path.len(),
                )
                    .0
            }
            _ => None,
        }
    }
}

/// Iterator to yield all parents
/// Field without path has no descendents.
// user_address_country_id -> country, address, user
pub struct Parents<'a> {
    pos: usize,
    path: &'a str,
}

impl<'a> Iterator for Parents<'a> {
    type Item = FieldPath<'a>;
    fn next(&mut self) -> Option<FieldPath<'a>> {
        let p = self.path[..self.pos].rfind('_');
        match p {
            Some(i) => {
                (Some(FieldPath::from(&self.path[i..self.pos])), {
                    self.pos = i
                })
                    .0
            }
            None if self.pos != 0 => {
                (Some(FieldPath::from(&self.path[..self.pos])), self.pos = 0).0
            }
            _ => None,
        }
    }
}

/// Iterator to yield all children
/// Field without path has no descendents.
// user_address_country_id -> user, address, country
pub struct Children<'a> {
    pos: usize,
    path: &'a str,
}

impl<'a> Iterator for Children<'a> {
    type Item = FieldPath<'a>;
    fn next(&mut self) -> Option<FieldPath<'a>> {
        let p = self.path[self.pos..].find('_');
        match p {
            Some(i) => {
                (Some(FieldPath::from(&self.path[self.pos..i])), {
                    self.pos = i + 1
                })
                    .0
            }
            None if self.pos != self.path.len() => {
                (
                    Some(FieldPath::from(&self.path[self.pos..])),
                    self.pos = self.path.len(),
                )
                    .0
            }
            _ => None,
        }
    }
}

impl<'a> ToString for FieldPath<'a> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
