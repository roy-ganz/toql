
# Row access control

Toql comes with role based access. Roles can only restrict access to columns but not to rows. 
For a full security model you also need restricted access to rows.

**Row access control should always be done in databases.** 

Reality is however that many databases (MySql) provide little support for that.

So if you really need to do it in Toql, here is a way to go:

Let's assume a table `Todo`:

|id | what | owner_id | done|
|---|------|----------|-----|
|1 | Clean kitchen | 5 | 10%|
|2 | Take garbage out | 5 | 100%|
|3 | Go shopping | 2 | 50%|

and a `Permission` table:

|entity | action | owner_only|
|-------|--------|-------|
|TODO | QUERY | true|
|TODO | UPDATE | true|


Roles can protect the column `done` in our `Todo` table so that only certain type of users see it.
To ensure that a user with id 5 can only see his own rows 1 + 2 set up a permission field and
build a custom field handler. Like so

```rust

use toql::prelude::{FieldHandler, BasicFieldHandler, \
	SqlExpr, ParameterMap, SqlBuilderError, SqlArg, sql_expr};

#[derive(Toql)]
struct Todo {
	#[toql(key)]
	id: u64,

	what: String, 

	#[toql(sql="", handler="permission_handler",  
			param(name = "entity", value = "TODO"))]
	permission: String
}

// Here comes our permission field handler
// We also want a custom filter function SC 
// so that we can filter for a specific permission.
//
// This allows the following toql queries
// Todos with any permissions -> `*, permission ne ''` 
// Todos with UPDATE permission -> `*, permission fn sc 'UPDATE'` 
struct PermissionFieldHandler{
	 default_handler: BasicFieldHandler, // The default field handler gives us default filters, such as `ne`
};

	pub fn permission_handler() -> impl FieldHandler {
    PermissionFieldHandler:{
		 default_handler: BasicFieldHandler::new(), 
	}
}
}

impl FieldHandler for PermissionFieldHandler
{
    fn build_select(
        &self,
        sql: SqlExpr,
        aux_params: &ParameterMap,
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        
		// Get user_id from aux params (would come from auth token)
		let user_id = aux_params.get("user_id").unwrap_or(&SqlArg::Null);

		// Get entity from aux params (locally provided with permission handler)
		let entity = aux_params.get("entity").unwrap_or(&SqlArg::Null).to_string();

		// Build subselect
		// Notice our special .. alias, it will be resolved later by the query builder
		// Build a string list with all permissions that we have as owners
		let sql = sql_expr!("(SELECT GROUP_CONCAT( p.action) FROM Permission p \
				WHERE p.entity = ? AND \
				(p.owner_only = false OR ..owner_id = ?))", entity, user_id);
		Ok(Some(sql))
    }
	 fn build_filter(
        &self,
        select: SqlExpr,
        filter: &FieldFilter,
        aux_params: &ParameterMap,
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        match filter {
			// Support our custom SC filter that maps to the MySQL FIND_IN_FIELD function
            FieldFilter::Fn(name, args) => match name.as_str() {
                "SC" => {
                    filter_sc(name, select, args)
                }
                name @ _ => Err(SqlBuilderError::FilterInvalid(name.to_string())),
            },
            _ => self.default_handler.build_filter(select, filter, aux_params),
        }
    }

}

pub fn filter_sc(
    name: &str,
    select: SqlExpr,
    args: &[SqlArg]
) -> Result<Option<SqlExpr>, SqlBuilderError> {
    if args.len() != 1 {
        return Err(SqlBuilderError::FilterInvalid(
            "filter `{}` expects exactly 1 argument",
            name
        )));
    }
	        
    Ok(Some(sql_expr!("FIND_IN_SET (? , {})", args[0], select)))
}

```





