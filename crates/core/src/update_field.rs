



// Implement for fields ands paths
pub trait UpdateField {

    fn into_field( self) -> String;

}



impl UpdateField for crate::query::field::Field  {
    fn into_field(mut self) -> String {

      // If a join or merge is provided, remove final path separator
      if self.name.ends_with("_") {
        self.name.pop();
      } 
      self.name
      
    }
}



impl UpdateField for crate::query::wildcard::Wildcard  {
    fn into_field(self) ->String {
      let p = self.path.trim_end_matches("_");
      if p.is_empty() {
        String::from("*")
      } else {
        format!("{}_*",p)
      }
    }
}
