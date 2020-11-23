/* 
use crate::{sql::Sql, sql_mapper_registry::SqlMapperRegistry, sql_builder::SqlBuilder, query::Query, error::{ToqlError, Result}, alias_translator::AliasTranslator, alias::AliasFormat, parameter::ParameterMap, sql_arg::SqlArg};
use std::collections::{HashSet, HashMap};


pub fn load_top_sql<M>(
    query: &Query<M>, 
type_name: &str, 
alias_format : AliasFormat, 
registry: &SqlMapperRegistry, 
aux_params: &HashMap<String, SqlArg>,
modifier: &str,
extra :&str
) -> Result<(Sql, HashSet<String>)> {

   

    

    let mut builder = SqlBuilder::new(&type_name, registry);
    let result = builder.build_select("", query)?;

    let mut alias_translator = AliasTranslator::new(alias_format);
    let aux_params = [aux_params];
    let aux_params = ParameterMap::new(&aux_params);

    

    Ok((
   result
        .to_sql_with_modifier_and_extra(
            &aux_params,
            &mut alias_translator,
            modifier,
            extra,
        )
        .map_err(ToqlError::from)?, 
        result.unmerged_paths().clone()
    ))

} */