pub struct Fields<'a>(pub &'a [&'a str]);

impl<'a> Fields<'a> {
    pub const ALL :Self = Fields(&[]);
}