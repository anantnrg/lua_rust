use mlua::{
	Lua,
	Table,
	Value,
};

pub fn get_or_create_module<'lua>(lua: &'lua Lua, name: &str) -> anyhow::Result<mlua::Table<'lua>> {
	let globals = lua.globals();
	let package: Table = globals.get("package")?;
	let loaded: Table = package.get("loaded")?;

	let module = loaded.get(name)?;
	match module {
		Value::Nil => {
			let module = lua.create_table()?;
			loaded.set(name, module.clone())?;
			Ok(module)
		}
		Value::Table(table) => Ok(table),
		wat => {
			anyhow::bail!(
				"cannot register module {} as package.loaded.{} is already set to a value of type \
				 {}",
				name,
				name,
				wat.type_name()
			)
		}
	}
}

pub fn get_or_create_sub_module<'lua>(
    lua: &'lua Lua,
    name: &str,
) -> anyhow::Result<mlua::Table<'lua>> {
    let strata_mod = get_or_create_module(lua, "strata")?;
    let sub = strata_mod.get(name)?;
    match sub {
        Value::Nil => {
            let sub = lua.create_table()?;
            strata_mod.set(name, sub.clone())?;
            Ok(sub)
        }
        Value::Table(sub) => Ok(sub),
        wat => anyhow::bail!(
            "cannot register module strata.{name} as it is already set to a value of type {}",
            wat.type_name()
        ),
    }
}