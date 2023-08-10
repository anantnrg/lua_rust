mod libs;

use libs::lua::module::{get_or_create_module, get_or_create_sub_module};
use mlua::{prelude::*, Lua, Table, UserData, Value};
use std::{env::var, fs::read_to_string};

struct StrataState;

impl StrataState {
    pub fn spawn(_: &Lua, cmd: String) -> LuaResult<()> {
        println!("Spawning {}", cmd.to_string());
        Ok(())
    }
}

struct StrataConfig;

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
        // table.set("cmd", cmd)?;
        Ok(func)
    }
    pub fn set_bindings<'a>(_: &'a Lua, bindings: Table<'a>) -> LuaResult<()> {
        println!("{:#?}", bindings);
        for key in bindings.sequence_values::<Table>() {
            // Debug for each settings This should Deserialize to Bindings struct
            let table: Table = key?.clone();
            let keys: Vec<String> = table.get("keys")?;
            let cmd: LuaFunction = table.get("cmd")?;
            println!("{:#?}", keys);
            println!("{:#?}", cmd);
            // Try to call lua functions
            println!("{:?}", cmd.call(0)?)
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

    // Create a new module
    let strata_mod = get_or_create_module(&lua, "strata")?;
    let cmd_submod = get_or_create_sub_module(&lua, "cmd")?;
    let api_submod = get_or_create_sub_module(&lua, "api")?;

    // Create "spawn config" for strata.cmd to construct LuaFunction and use it later.
    cmd_submod.set("spawn", lua.create_function(StrataConfig::spawn)?)?;
    // Create "spawn api" for strata.api that can triggers LuaFunction as needed.
    api_submod.set("spawn", lua.create_function(StrataState::spawn)?)?;

    strata_mod.set(
        "set_bindings",
        lua.create_function(StrataConfig::set_bindings)?,
    )?;

    lua.load(&config_str).exec()?;

    Ok(())
}
