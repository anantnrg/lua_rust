use mlua::{Lua, Result};
use std::{
	env::var,
	fs::read_to_string,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    autostart: Vec<Vec<String>>,
    general: GeneralConfig,
    decorations: DecorationsConfig,
    tiling: TilingConfig,
    animations: AnimationsConfig,
    rules: RulesConfig,
    bindings: Vec<BindingConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeneralConfig {
    workspaces: u32,
    gaps_in: u32,
    gaps_out: u32,
    kb_repeat: Vec<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DecorationsConfig {
    border: BorderConfig,
    window: WindowConfig,
    blur: BlurConfig,
    shadow: ShadowConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct BorderConfig {
    width: u32,
    active: String,
    inactive: String,
    radius: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct WindowConfig {
    opacity: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct BlurConfig {
    enabled: bool,
    size: u32,
    passes: u32,
    optimize: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ShadowConfig {
    enabled: bool,
    size: u32,
    blur: u32,
    color: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TilingConfig {
    layout: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnimationsConfig {
    enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct RulesConfig {
    workspaces: Vec<WorkspaceRuleConfig>,
    floating: Vec<FloatingRuleConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkspaceRuleConfig {
    workspace: u32,
    class_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FloatingRuleConfig {
    class_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct BindingConfig {
    keys: Vec<String>,
    cmd: String,
}

fn main() -> Result<()> {
    let lua = Lua::new();
	let config_path =
		format!("{}/.config/strata/strata.lua", var("HOME").expect("This should always be set!!!"));
	let config_str = read_to_string(config_path).unwrap();

    lua.load(&config_str).exec()?;

	let config: mlua::Table = lua.globals().get("config")?;

	println!("{:#?}", config);

    Ok(())
}

