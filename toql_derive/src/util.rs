

use heck::CamelCase;
use heck::ShoutySnakeCase;
use heck::MixedCase;
use heck::SnakeCase;

pub fn rename_sql_table(table: &str, renaming: &Option<String>) -> Result<String, String>{
        if let Some(tables) = &renaming {
            match tables.as_ref() {
                "CamelCase" => Ok(table.to_camel_case()),
                "snake_case" => Ok(table.to_snake_case()),
                "SHOUTY_SNAKE_CASE" => Ok(table.to_shouty_snake_case()),
                "mixedCase" => Ok(table.to_mixed_case()),
                _ => Err("Invalid case. Use \"CamelCase\", \"snake_case\", \"SHOUTY_SNAKE_CASE\" or\"mixedCase\".".to_owned())
           }
        } else {
            Ok(table.to_owned())
        }

    }
pub fn rename_sql_column( column: &str, renaming: &Option<String>) -> Result<String, String>{
       if let Some(columns) = &renaming {
        match columns.as_ref() {
            "CamelCase" => Ok(column.to_camel_case()),
            "snake_case" => Ok(column.to_snake_case()),
            "SHOUTY_SNAKE_CASE" => Ok(column.to_shouty_snake_case()),
            "mixedCase" => Ok(column.to_mixed_case()),
            _ => Err("Invalid case. Use \"CamelCase\", \"snake_case\", \"SHOUTY_SNAKE_CASE\" or\"mixedCase\".".to_owned())
        }
    } else {
        Ok(column.to_owned())
    }
}