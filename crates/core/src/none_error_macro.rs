
#[macro_export]
macro_rules! none_error {
    () => {
        toql::error::ToqlError::NoneError(format!(
            "Expected value, but found `None` on {}:{}",
            file!(),
            line!()
        ))
    };
}