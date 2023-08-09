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
	pub fn spawn(cmd: &str) {
		println!("Launching: {}", cmd);
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

	lua.load(&config_str).eval()?;

	Ok(())
}
