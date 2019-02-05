pub trait Buildable<T> {
    fn build(row:&str)->T;
}