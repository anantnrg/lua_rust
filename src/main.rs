use mlua::{Lua, Result, UserData, UserDataMethods};
use std::process::Command;

// Define the userdata for Strata
struct StrataState;

impl UserData for StrataState {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("spawn", |_, this, cmd: String| Ok(this.spawn(cmd)));
    }
}

impl StrataState {
    fn spawn(&self, cmd: String) {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .spawn()
            .expect("Failed to execute the command");
    }
}

fn main() -> Result<()> {
    let lua = Lua::new();

    let strata = StrataState;
    lua.globals().set("strata", lua.create_userdata(strata)?)?;

    let lua_script = r#"
        strata:spawn("echo Hello from Lua!")
    "#;

    lua.load(lua_script).exec()?;

    Ok(())
}