
#[derive(Debug, PartialEq)]
pub enum Select {
    Explicit,
    Implicit,
    None
}

/*
// Select stream memorizes which columns and joins are selected. 
The selections can either be explicit from the query or implicit from 
*/
#[derive(Debug)]
pub struct SelectStream {
    stream: Vec<Select>
}

impl SelectStream {

      pub fn new() -> Self{
        Self { stream : Vec::new()}

    }

  /*   pub fn count_selected(&self) -> usize {
        self.stream.iter().filter(|s| s!= &&Select::None).count()
    } */

    pub fn change(&mut self, pos: usize, select: Select) {

        if let Some(p) = self.stream.get_mut(pos) {
            *p = select;
        }
    }
    pub fn push(&mut self, selection: Select) {
        self.stream.push(selection);

    }
    pub fn len(&mut self) -> usize {
        self.stream.len()

    }

    pub fn iter(&self) -> std::slice::Iter<'_,Select> {
        self.stream.iter()
    }

}