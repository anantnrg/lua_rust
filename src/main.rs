mod libs;

use libs::lua::module::{
	get_or_create_module,
	get_or_create_sub_module,
};
use mlua::{
	prelude::*,
	Lua,
	Table,
	UserData,
	Value,
};
use serde::Deserialize;
use std::{
	env::var,
	fs::read_to_string,
};

struct StrataState;

impl StrataState {
	pub fn spawn(_: &Lua, cmd: String) -> LuaResult<()> {
		println!("Received command: {}", cmd);
		Ok(())
	}
	pub fn set_bindings<'a>(_: &'a Lua, bindings: LuaTable<'a>) -> LuaResult<LuaTable<'a>> {
		Ok(bindings)
	}
}

fn main() -> anyhow::Result<()> {
	let lua = Lua::new();
	let config_path =
		format!("{}/.config/strata/strata.lua", var("HOME").expect("This should always be set!!!"));
	let config_str = read_to_string(config_path).unwrap();

	// Create a new module
	let strata_mod = get_or_create_module(&lua, "strata")?;
	let cmd_submod = get_or_create_sub_module(&lua, "cmd")?;

	cmd_submod.set("spawn", lua.create_function(StrataState::spawn)?)?;
	strata_mod.set("set_bindings", lua.create_function(StrataState::set_bindings)?)?;

	lua.load(&config_str).exec()?;

	Ok(())
}
