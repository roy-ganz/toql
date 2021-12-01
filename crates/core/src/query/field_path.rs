use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct FieldPath<'a>(Cow<'a, str>);

impl<'a> Default for FieldPath<'a> {
    fn default() -> Self {
        FieldPath(Cow::Owned(String::from("")))
    }
}

impl<'a> std::ops::Deref for FieldPath<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> FieldPath<'a> {
    /// Split's off the last segment of a path
    /// `author_address_street` becomes `author_address` and `street`
    pub fn split_basename(path_with_basename: &str) -> (FieldPath, &str) {
        if let Some(pos) = path_with_basename.rfind('_') {
            (
                FieldPath::from(&path_with_basename[..pos]),
                &path_with_basename[pos + 1..],
            )
        } else {
            (FieldPath::default(), path_with_basename)
        }
    }
    /// Removes the last segment of a path
    /// `author_address_street` becomes `author_address`
    pub fn trim_basename(path_with_basename: &str) -> FieldPath {
        if let Some(pos) = path_with_basename.rfind('_') {
            FieldPath::from(&path_with_basename[..pos])
        } else {
            FieldPath::default()
        }
    }
    /// Creates a borrowed `FieldPath` from a `str`
    pub fn from(path: &'a str) -> Self {
        FieldPath(Cow::Borrowed(path))
    }
    /// Returns the field path if it is not empty, otherwise returns `b`
    pub fn or(&self, b: &'a FieldPath) -> &FieldPath {
        if self.is_empty() {
            b
        } else {
            self
        }
    }
    /// Returns `true` if the path is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Creates an owned `FieldPath` with head prepended to self.
    pub fn prepend(&self, head: &'a str) -> Self {
        let path = format!(
            "{}{}{}",
            head,
            if self.0.is_empty() || head.is_empty() {
                ""
            } else {
                "_"
            },
            self.0.as_ref()
        );

        FieldPath(Cow::Owned(path))
    }
    /// Creates an owned `FieldPath` with tail appended to self.
    pub fn append(&self, tail: &'a str) -> Self {
        let path = format!(
            "{}{}{}",
            self.0.as_ref(),
            if self.0.is_empty() || tail.is_empty() {
                ""
            } else {
                "_"
            },
            tail,
        );

        FieldPath(Cow::Owned(path))
    }
    /// Return local path for home path
    /// local path for field path `contacts_phone_number` and home path `contacts`
    /// is `phone_number`
    pub fn localize_path(&self, home_path: &str) -> Option<FieldPath> {
        if self.0.starts_with(home_path) {
            let t = self.0.trim_start_matches(home_path).trim_start_matches('_');
            Some(FieldPath::from(t))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Iterator to yield all parents
    /// Field without path has no parents.
    /// user_address_country_id -> country, address, user
    pub fn parents(&self) -> Parents {
        Parents {
            pos: self.0.len(),
            path: self.0.as_ref(),
        }
    }

    /// Creates an iterator to yield all children
    /// Field without path has no children.
    /// user_address_country_id -> user, address, country
    pub fn children(&self) -> Children {
        Children {
            pos: 0,
            path: self.0.as_ref(),
        }
    }

    /// Iterator to step down
    /// Field without path has no steps.
    /// user_address_country_id -> user, user_address, user_address_country, user_address_country_id
    pub fn step_down(&self) -> StepDown {
        StepDown {
            pos: 0,
            path: self.0.as_ref(),
        }
    }

    /// Iterator to step up
    /// Field without path has no steps.
    /// user_address_country_id -> user_address_country_id, user_address_country, user_address, user
    pub fn step_up(&self) -> StepUp {
        StepUp {
            pos: self.0.len(),
            path: self.0.as_ref(),
        }
    }
}

pub struct StepDown<'a> {
    pos: usize,
    path: &'a str,
}

impl<'a> Iterator for StepDown<'a> {
    type Item = FieldPath<'a>;
    fn next(&mut self) -> Option<FieldPath<'a>> {
        let p = self.path[self.pos..].find('_');

        match p {
            Some(i) => {
                let end = self.pos + i;
                Some((FieldPath::from(&self.path[..end]), self.pos = end + 1).0)
            }
            None if self.pos != self.path.len() => {
                Some((FieldPath::from(&self.path), self.pos = self.path.len()).0)
            }
            _ => None,
        }
    }
}

pub struct StepUp<'a> {
    pos: usize,
    path: &'a str,
}

impl<'a> Iterator for StepUp<'a> {
    type Item = FieldPath<'a>;
    fn next(&mut self) -> Option<FieldPath<'a>> {
        let p = self.path[..self.pos].rfind('_');

        match p {
            Some(i) => {
                let end = self.pos;
                Some((FieldPath::from(&self.path[..end]), self.pos = i).0)
            }
            None if self.pos != 0 => {
                Some((FieldPath::from(&self.path[..self.pos]), self.pos = 0).0)
            }
            _ => None,
        }
    }
}

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
                (Some(FieldPath::from(&self.path[i + 1..self.pos])), {
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

#[derive(Clone)]
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
                let end = self.pos + i;
                (Some(FieldPath::from(&self.path[self.pos..end])), {
                    self.pos = end + 1
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

#[cfg(test)]
mod test {
    use super::FieldPath;

    #[test]
    fn parents() {
        let p = FieldPath::from("level1")
            .parents()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        assert_eq!(p, vec!["level1"]);

        let p = FieldPath::from("level1_level2_level3")
            .parents()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        assert_eq!(p, vec!["level3", "level2", "level1"]);

        let p = FieldPath::default()
            .parents()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        assert_eq!(p.is_empty(), true);
    }
    #[test]
    fn children() {
        let p = FieldPath::from("level1")
            .children()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        assert_eq!(p, vec!["level1"]);

        let p = FieldPath::from("level1_level2_level3")
            .children()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        assert_eq!(p, vec!["level1", "level2", "level3"]);

        let p = FieldPath::default()
            .children()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        assert_eq!(p.is_empty(), true);
    }
    #[test]
    fn step_down() {
        let p = FieldPath::from("level1")
            .step_down()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        assert_eq!(p, vec!["level1"]);

        let p = FieldPath::from("level1_level2_level3")
            .step_down()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        assert_eq!(p, vec!["level1", "level1_level2", "level1_level2_level3"]);

        let p = FieldPath::default()
            .step_down()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        assert_eq!(p.is_empty(), true);
    }
    #[test]
    fn step_up() {
        let p = FieldPath::from("level1")
            .step_up()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        assert_eq!(p, vec!["level1"]);

        let p = FieldPath::from("level1_level2_level3")
            .step_up()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        assert_eq!(p, vec!["level1_level2_level3", "level1_level2", "level1",]);

        let p = FieldPath::default()
            .step_up()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        assert_eq!(p.is_empty(), true);
    }

    #[test]
    fn build() {
        let p = FieldPath::default();
        let p = p.append("level1");
        assert_eq!(p.to_string(), "level1");

        let p = FieldPath::from("level1");
        let p = p.append("level2");
        assert_eq!(p.to_string(), "level1_level2");

        let p = FieldPath::default();
        let p = p.prepend("level1");
        assert_eq!(p.to_string(), "level1");

        let p = FieldPath::from("level2");
        let p = p.prepend("level1");
        assert_eq!(p.to_string(), "level1_level2");

        let p = FieldPath::trim_basename("level1_level2");
        assert_eq!(p.to_string(), "level1");

        let p = FieldPath::trim_basename("level1_level2_");
        assert_eq!(p.to_string(), "level1_level2");

        let p = FieldPath::trim_basename("");
        assert_eq!(p.to_string(), "");

        let (p, n) = FieldPath::split_basename("level1_level2");
        assert_eq!(p.to_string(), "level1");
        assert_eq!(n, "level2");

        let (p, n) = FieldPath::split_basename("level1_level2_");
        assert_eq!(p.to_string(), "level1_level2");
        assert_eq!(n, "");

        let (p, n) = FieldPath::split_basename("");
        assert_eq!(p.to_string(), "");
        assert_eq!(p.is_empty(), true);
        assert_eq!(n, "");

        let pn = FieldPath::default();
        let p1 = FieldPath::from("level1");
        let p2 = FieldPath::from("level2");
        assert_eq!(pn.or(&p1).to_string(), "level1");
        assert_eq!(p1.or(&pn).to_string(), "level1");
        assert_eq!(p1.or(&p2).to_string(), "level1");
        assert_eq!(p2.or(&p1).to_string(), "level2");

        let p = FieldPath::from("level1_level2_level3");
        assert_eq!(
            p.localize_path("level1"),
            Some(FieldPath::from("level2_level3"))
        );
        assert_eq!(p.localize_path("level4").is_some(), false);
        assert_eq!(
            p.localize_path("level1_"),
            Some(FieldPath::from("level2_level3"))
        );
    }
}
