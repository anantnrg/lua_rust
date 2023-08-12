mod libs;
use crate::libs::config::*;
use lazy_static::lazy_static;
use libs::lua::module::{
	get_or_create_module,
	get_or_create_sub_module,
};
use mlua::{
	prelude::*,
	Lua,
	Table,
	// UserData,
	// Value,
};
use std::{
	env::var,
	fs::read_to_string,
	sync::{
		Arc,
		Mutex,
	},
};

lazy_static! {
	static ref CONFIG: Arc<Mutex<Config>> =
		Arc::new(Mutex::new(Config::default()));
}

struct StrataState;
struct StrataConfig;

impl StrataState {
	pub fn spawn(_: &Lua, cmd: String) -> LuaResult<()> {
		println!("Spawning {}", cmd.to_string());
		Ok(())
	}
}

impl StrataConfig {
	pub fn spawn(lua: &Lua, cmd: String) -> LuaResult<LuaFunction> {
		let func = lua
			.load(format!(
				r#"
            local strata = require("strata")
            strata.api.spawn("{}")"#,
				cmd
			))
			.into_function()?;
		Ok(func)
	}

	pub fn set_bindings(lua: &Lua, bindings: Table) -> LuaResult<()> {
		for key in bindings.sequence_values::<Table>() {
			let table: Table = key?.clone();
			let keys: Vec<String> = table.get("keys")?;
			let cmd: LuaFunction = table.get("cmd")?;
			let _ = lua
				.globals()
				.get::<&str, Table>("package")?
				.get::<&str, Table>("loaded")?
				.get::<&str, Table>("strata")?
				.get::<&str, Table>("bindings")?
				.set(keys.clone().concat(), cmd)?;
			CONFIG.lock().unwrap().bindings.push(Keybinding {
				keys: keys.clone(),
				func: keys.clone().concat(),
			});
		}
		Ok(())
	}
}

fn main() -> anyhow::Result<()> {
	let lua = Lua::new();
	let config_path = format!(
		"{}/.config/strata/strata.lua",
		var("HOME").expect("This should always be set!!!")
	);
	let config_str = read_to_string(config_path).unwrap();

	// Create the `strata` module
	let strata_mod = get_or_create_module(&lua, "strata")?;
	// Create the submodule that holds commonly used commands
	let cmd_submod = get_or_create_sub_module(&lua, "cmd")?;
	// Create a separate submodule which is not accessible to the user. this is used to call functions from the rust code
	let api_submod = get_or_create_sub_module(&lua, "api")?;
	let _binding_submod = get_or_create_sub_module(&lua, "bindings");

	// Create the function which the user will call from the config.
	cmd_submod.set("spawn", lua.create_function(StrataConfig::spawn)?)?;
	// Create the function that is called from the Rust code.
	api_submod.set("spawn", lua.create_function(StrataState::spawn)?)?;

	// Create the other functions.
	strata_mod.set(
		"set_bindings",
		lua.create_function(StrataConfig::set_bindings)?,
	)?;

	lua.load(&config_str).exec()?;

	for func in CONFIG.lock().unwrap().bindings.iter() {
		let cmd = func.func.clone();
		lua.globals()
			.get::<&str, Table>("package")?
			.get::<&str, Table>("loaded")?
			.get::<&str, Table>("strata")?
			.get::<&str, Table>("bindings")?
			.get::<&str, LuaFunction>(&cmd)?
			.call::<i32, _>(0)?;
	}

	Ok(())
}
