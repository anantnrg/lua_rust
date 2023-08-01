use mlua::{
	prelude::*,
	Lua,
	Result,
};
use std::{
	env::var,
	fs::read_to_string,
};

struct StrataState;

impl StrataState {
	fn spawn_terminal() -> String {
		"kitty --title Terminal".to_string()
	}
}

impl LuaUserData for StrataState {}

fn main() -> Result<()> {
	let lua = Lua::new();
	let config_path =
		format!("{}/.config/strata/strata.lua", var("HOME").expect("This should always be set!!!"));
	let config_str = read_to_string(config_path).unwrap();
	let strata_state = lua.create_userdata(StrataState)?;

    lua.globals().set("strata", strata_state)?;

	lua.load(&config_str).exec().unwrap();

    let config = lua.globals().get::<_, LuaTable>("config").unwrap();
    println!("{:#?}", config);


	Ok(())
}
