use anyhow::Error;
use mlua::{
	prelude::*,
	Lua,
	Table,
	UserData,
	Value,
};
use serde::{Deserialize, Deserializer};
use std::{
	env::var,
	fs::read_to_string,
};
use serde_json;

#[derive(Debug, Deserialize)]
struct Config {
	autostart: Vec<String>,
	general: General,
	decorations: Decorations,
	tiling: Tiling,
	animations: Animations,
	rules: Rules,
	bindings: Vec<Binding>,
}

#[derive(Debug, Deserialize, Clone)]
struct Binding {
	keys: Vec<String>,
	cmd: String, // Assuming the cmd is a string representing a function name
}

#[derive(Debug, Deserialize, Clone)]
struct General {
	workspaces: i32,
	gaps_in: i32,
	gaps_out: i32,
	kb_repeat: Vec<i32>,
}

#[derive(Debug, Deserialize, Clone)]
struct Border {
	width: i32,
	active: String,
	inactive: String,
	radius: i32,
}

#[derive(Debug, Deserialize, Clone)]
struct Window {
	opacity: f32,
}

#[derive(Debug, Deserialize, Clone)]
struct Blur {
	enabled: bool,
	size: i32,
	passes: i32,
	optimize: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct Shadow {
	enabled: bool,
	size: i32,
	blur: i32,
	color: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Decorations {
	border: Border,
	window: Window,
	blur: Blur,
	shadow: Shadow,
}

#[derive(Debug, Deserialize, Clone)]
struct Tiling {
	layout: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Animations {
	enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct WorkspacesRule {
	workspace: i32,
	class_name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct FloatingRule {
	class_name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Rules {
	workspaces: Vec<WorkspacesRule>,
	floating: Vec<FloatingRule>,
}

struct StrataState;

impl StrataState {
	pub fn spawn() {
		println!("kitty --title Terminal");
	}
}

impl UserData for StrataState {}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Config, D::Error>
    where
        D: Deserializer<'de>,
    {
        let lua_value = Value::deserialize(deserializer)?;
        let lua = Lua::new();

        // Convert LuaValue to LuaTable
        let lua_table = match lua_value {
            Value::Table(table) => table,
            _ => return Err(D::Error::custom("Invalid Lua table format")),
        };

        // Convert LuaTable to Config using from_lua function
        let config = lua.from_lua(lua_table, ())?;
        Ok(config)
    }
}

fn main() -> anyhow::Result<()> {
	let lua = Lua::new();
	let config_path =
		format!("{}/.config/strata/strata.lua", var("HOME").expect("This should always be set!!!"));
	let config_str = read_to_string(config_path).unwrap();

	let state_mod = get_or_create_module(&lua, "strata")?;

	let spawn = lua.create_function(|_, _: ()| Ok(StrataState::spawn()))?;

	state_mod.set("spawn", spawn)?;

	lua.load(&config_str).exec()?;

	let globals = lua.globals();
	let config: LuaTable = globals.get("config")?;

	let json_str = serde_json::to_string(&config).map_err(mlua::Error::external)?;
    println!("{}", json_str);

    // Deserialize JSON back into Config struct
    let config: Config = serde_json::from_str(&json_str).map_err(mlua::Error::external)?;
    println!("{:#?}", config);


	println!("{:?}", config);

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
