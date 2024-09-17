use std::error::Error;

use mlua::prelude::*;
use pairwriter::prelude::{start_server, *};
use tokio::runtime::Runtime;

lazy_static::lazy_static! {
    static ref RT: Runtime = Runtime::new().unwrap();
}

#[mlua::lua_module]
fn pairwriter_helper(lua: &Lua) -> LuaResult<LuaTable> {
    let out = lua.create_table()?;
    out.set(
        "start_server",
        lua.create_function(server::lua_start_server).unwrap(),
    )?;
    out.set(
        "connect_as_client",
        lua.create_function(client::client_connect)?,
    )?;
    Ok(out)
}

/// using this to copy the table
/// don't use it for nested tables
fn shallow_copy(src: &LuaTable, dest: &LuaTable) -> LuaResult<()> {
    for key in src.clone().pairs::<String, LuaValue>() {
        let (k, v) = key?;
        dest.set(k, v)?;
    }
    Ok(())
}
fn create_txt_from_lines(lines: LuaTable) -> String {
    let mut out = String::new();
    let mut i = 1; // lua is 1-indexed
    loop {
        let line: Result<String, LuaError> = lines.get(i);
        if line.is_err() {
            break;
        }
        let line = line.unwrap();
        out.push_str(&line);
        out.push('\n');
        i += 1;
    }
    out
}
fn text_to_lines(lua: &Lua, text: impl std::io::BufRead) -> LuaResult<LuaTable> {
    let out = lua.create_table()?;
    // use std::io::BufRead as _;
    for (i, line) in text.lines().enumerate() {
        out.set(i + 1, line?)?;
    }
    Ok(out)
}

mod client;
mod server;
