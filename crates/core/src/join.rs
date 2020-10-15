


pub enum Join<K: crate::key::Keyed> {
    Key(K::Key),
    Entity(K)
}

/*
impl<K> Serialize for Join<K> {


}*/