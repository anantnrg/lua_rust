use anyhow::Error;
use mlua::{
	prelude::*,
	Lua,
	Table,
	UserData,
	Value,
};
use serde::{
	Deserialize,
	Deserializer,
};
use serde_json;
use std::{
	env::var,
	fs::read_to_string,
};

#[derive(Debug, Deserialize)]
struct Config {
	autostart: Vec<String>,
	general: General,
	decorations: Decorations,
	tiling: Tiling,
	animations: Animations,
	rules: Rules,
}

#[derive(Debug, Deserialize)]
struct Bindings<'lua> {
	bindings: Vec<Binding<'lua>>,
}

#[derive(Debug, Clone)]
struct Binding<'lua> {
	keys: Vec<&'lua str>,
	cmd: LuaFunction<'lua>,
}

impl<'lua> Deserialize<'lua> for Binding<'lua> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'lua>,
	{
		#[derive(Deserialize)]
		struct BindingFields {
			keys: Vec<String>,
			cmd: FunctionWrapper, 
		}

		#[derive(Deserialize)]
		struct FunctionWrapper(String);

		let binding_fields: BindingFields = Deserialize::deserialize(deserializer)?;
		let keys_str_refs: Vec<&str> = binding_fields.keys.iter().map(|s| s.as_str()).collect();
		let lua = deserializer.lua(); 
		let cmd: LuaFunction = lua
			.globals()
			.get::<_, LuaFunction>(&binding_fields.cmd.0)
			.map_err(mlua::Error::external)?;

		Ok(Binding {
			keys: keys_str_refs,
			cmd,
		})
	}
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
	let config_table: LuaTable = globals.get("config")?;
	let bindings_table: LuaTable = globals.get("bindings")?;

	let config_str = serde_json::to_string(&config_table).map_err(mlua::Error::external)?;
	let config: Config = serde_json::from_str(&config_str).map_err(mlua::Error::external)?;

    let mut bindings: Vec<Binding> = Vec::new();
    for (_, binding_table) in bindings_table.pairs::<Value, LuaTable>() {
        let keys: Vec<String> = binding_table.get("keys")?;
        let cmd: String = binding_table.get("cmd")?;
        
        bindings.push(Binding { keys, cmd });
    }

    // Now you have the bindings as a Rust Vec<Binding>
    for binding in &bindings {
        println!("Keys: {:?}, Cmd: {}", binding.keys, binding.cmd);
    }
	println!("{:#?}", config);
	println!("{:#?}", bindings);

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
