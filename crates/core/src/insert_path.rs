// Implement for fields ands paths
pub trait InsertPath {

    fn as_path<'a>(&'a self) -> &'a str;

}

