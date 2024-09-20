use std::{thread::sleep, time::Duration};

use super::*;
use pairwriter::client_import::*;
use pairwriter::prelude::RPC;

pub fn client_connect(lua: &Lua, (url, username): (String, String)) -> LuaResult<()> {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(connect_as_client(url, username))
    });
    let mut i = 0;

    while i < 5 {
        sleep(Duration::from_secs(1));
        if client_api.get().is_some() {
            break;
        }
        i += 1;
    }
    if i == 5 {
        lua.globals()
            .call_function("error", "Failed to connect to server")?;
        return Err(mlua::Error::external("Failed to connect to server"));
    }

    lua.globals()
        .call_function("print", "Connected to server")?;
    export_functions::export_cmds(lua)?;
    hooks::start_hooks(lua)?;

    Ok(())
}

fn bufname_to_relativepath(path: impl Into<String>) -> String {
    let path: String = path.into();
    path.replacen("pairwriter://", "", 1)
}
fn relativepath_to_bufname(path: impl Into<String>) -> String {
    let path: String = path.into();
    format!("pairwriter://{}", path)
}

pub fn undo(_lua: &Lua, path: String) -> LuaResult<()> {
    RT.block_on(async {
        client_api
            .get()
            .unwrap()
            .lock()
            .await
            .send_rpc(RPC::Undo { path })
            .await;
    });

    Ok(())
}
pub fn redo(_lua: &Lua, path: String) -> LuaResult<()> {
    RT.block_on(async {
        client_api
            .get()
            .unwrap()
            .lock()
            .await
            .send_rpc(RPC::Redo { path })
            .await;
    });

    Ok(())
}
mod export_functions;
mod hooks;
