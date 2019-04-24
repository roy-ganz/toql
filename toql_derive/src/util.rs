

use heck::CamelCase;
use heck::ShoutySnakeCase;
use heck::MixedCase;
use heck::SnakeCase;

pub fn rename_sql_table(table: &str, renaming: &Option<&str>) -> String{
        if let Some(tables) = &renaming {
            match tables.as_ref() {
                "CamelCase" => table.to_camel_case(),
                "snake_case" => table.to_snake_case(),
                "SHOUTY_SNAKE_CASE" => table.to_shouty_snake_case(),
                "mixedCase" => table.to_mixed_case(),
                _ => table.to_owned()
           }
        } else {
            table.to_owned()
        }

    }
pub fn rename_sql_column( column: &str, renaming: &Option<&str>) -> String{
       if let Some(columns) = &renaming {
        match columns.as_ref() {
            "CamelCase" => column.to_camel_case(),
            "snake_case" => column.to_snake_case(),
            "SHOUTY_SNAKE_CASE" => column.to_shouty_snake_case(),
            "mixedCase" => column.to_mixed_case(),
            _ => column.to_owned()
        }
    } else {
        column.to_owned()
    }
}