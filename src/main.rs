use mlua::{
	prelude::*,
	Lua,
	Result,
	Table,
	UserData,
	Value,
};
use std::{
	env::var,
	fs::read_to_string,
};

struct StrataState;

impl StrataState {
	pub fn spawn() {
		println!("kitty --title Terminal");
	}
}

impl UserData for StrataState {}

fn main() -> anyhow::Result<()> {
	let lua = Lua::new();
	let config_path =
		format!("{}/.config/strata/strata.lua", var("HOME").expect("This should always be set!!!"));
	let config_str = read_to_string(config_path).unwrap();

	let state_mod = get_or_create_module(&lua, "strata")?;

	let spawn = lua.create_function(|_, _: ()| {
		Ok(StrataState::spawn())
	})?;

	state_mod.set("spawn", spawn)?;

	lua.load(&config_str).exec()?;

	Ok(())
}

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
