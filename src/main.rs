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
use std::{
	env::var,
	fs::read_to_string,
	sync::{
		Arc,
		Mutex,
	},
};

struct StrataState;
struct Config;

struct Keybinds<'a> {
	keybinds: Vec<Keybind<'a>>,
}

impl StrataState {
	pub fn spawn(_: &Lua, cmd: String) -> LuaResult<()> {
		println!("Spawning {}", cmd.to_string());
		Ok(())
	}
}

struct Keybind<'a> {
	keys: Vec<String>,
	cmd: LuaFunction<'a>,
}

impl Config {
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
}

impl<'a> Keybinds<'a> {
	pub fn new() -> Self {
		Keybinds { keybinds: Vec::new() }
	}

	pub fn set_bindings(&mut self, _: &Lua, bindings: Table<'a>) -> LuaResult<()> {
		for key in bindings.sequence_values::<Table>() {
			let table: Table<'a> = key?.clone();
			let keys: Vec<String> = table.get("keys")?;
			let cmd: LuaFunction<'a> = table.get("cmd")?;
			self.keybinds.push(Keybind { keys, cmd });
		}
		Ok(())
	}
}

fn main() -> anyhow::Result<()> {
	let lua = Lua::new();
	let config_path =
		format!("{}/.config/strata/strata.lua", var("HOME").expect("This should always be set!!!"));
	let config_str = read_to_string(config_path).unwrap();
	let keybinds = Arc::new(Mutex::new(Keybinds { keybinds: Vec::new() }));

	// Create the `strata` module
	let strata_mod = get_or_create_module(&lua, "strata")?;
	// Create the submodule that holds commonly used commands
	let cmd_submod = get_or_create_sub_module(&lua, "cmd")?;
	// Create a separate submodule which is not accessible to the user. this is used to call functions from the rust code
	let api_submod = get_or_create_sub_module(&lua, "api")?;

	// Create the function which the user will call from the config.
	cmd_submod.set("spawn", lua.create_function(Config::spawn)?)?;
	// Create the function that is called from the Rust code.
	api_submod.set("spawn", lua.create_function(StrataState::spawn)?)?;

	// Create the other functions.
	strata_mod.set(
		"set_bindings",
		lua.create_function(move |_, bindings: Table| {
			let cloned_keybinds = keybinds.clone();
			cloned_keybinds.lock().unwrap().set_bindings(&lua, bindings)?;
			Ok(())
		})?,
	)?;

	lua.load(&config_str).exec()?;

	Ok(())
}
