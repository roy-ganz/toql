use crate::join::Join;
use crate::keyed::Keyed;
use crate::result::Result;
use crate::error::ToqlError;

// Trait to improve convinience when working with Join<T> and Option<Join<T>>
pub trait TryJoin
where
    Self: Sized,
{
    type Output;
    fn try_join(&self) -> Result<&Self::Output>;
    fn try_join_or<E>(&self, err: E) -> std::result::Result<&Self::Output, E> {
        self.try_join().map_err(|_| err)
    }
    fn try_join_mut(&mut self) -> Result<&mut Self::Output>;
    fn try_join_mut_or<E>(&mut self, err: E) -> std::result::Result<&mut Self::Output, E> {
        self.try_join_mut().map_err(|_| err)
    }
}

impl<T> TryJoin for Join<T>
where
    T: Keyed,
{
    type Output = T;
    fn try_join(&self) -> Result<&Self::Output> {
        self.entity_or_err(ToqlError::NotFound)
    }
    fn try_join_mut(&mut self) -> Result<&mut Self::Output> {
        self.entity_mut_or_err(ToqlError::NotFound)
    }
}
impl<T> TryJoin for &mut Join<T>
where
    T: Keyed,
{
    type Output = T;
    fn try_join(&self) -> Result<&Self::Output> {
        self.entity_or_err(ToqlError::NotFound)
    }
    fn try_join_mut(&mut self) -> Result<&mut Self::Output> {
        self.entity_mut_or_err(ToqlError::NotFound)
    }
}

impl<T> TryJoin for Option<Join<T>>
where
    T: Keyed,
{
    type Output = T;
    fn try_join(&self) -> Result<&Self::Output> {
        self.as_ref()
            .ok_or(ToqlError::JoinExpected)?
            .entity_or_err(ToqlError::JoinExpected)
    }
    fn try_join_mut(&mut self) -> Result<&mut Self::Output> {
        self.as_mut()
            .ok_or(ToqlError::JoinExpected)?
            .entity_mut_or_err(ToqlError::JoinExpected)
    }
}
impl<T> TryJoin for Option<Option<Join<T>>>
where
    T: Keyed,
{
    type Output = T;
    fn try_join(&self) -> Result<&Self::Output> {
        self.as_ref()
            .ok_or(ToqlError::JoinExpected)?
            .as_ref()
            .ok_or(ToqlError::JoinExpected)?
            .entity_or_err(ToqlError::JoinExpected)
    }
    fn try_join_mut(&mut self) -> Result<&mut Self::Output> {
        self.as_mut()
            .ok_or(ToqlError::JoinExpected)?
            .as_mut()
            .ok_or(ToqlError::JoinExpected)?
            .entity_mut_or_err(ToqlError::JoinExpected)
    }
}
