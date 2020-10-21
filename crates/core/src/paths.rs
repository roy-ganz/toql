pub struct Paths<'a>(pub &'a [&'a str]);

impl<'a> Paths<'a> {
    pub const ROOT :Self = Paths(&[]);
}