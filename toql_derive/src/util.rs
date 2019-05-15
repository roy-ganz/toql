

use heck::CamelCase;
use heck::ShoutySnakeCase;
use heck::MixedCase;
use heck::SnakeCase;

use crate::annot::RenameCase;

pub(crate) fn rename(string: &str, renaming: &Option<RenameCase>) -> String{
        
            match renaming {
                Some(RenameCase::CamelCase) => string.to_camel_case(),
                Some(RenameCase::SnakeCase) => string.to_snake_case(),
                Some(RenameCase::ShoutySnakeCase) => string.to_shouty_snake_case(),
                Some(RenameCase::MixedCase) => string.to_mixed_case(),
                None => string.to_owned()
           }
    }
