use std::{thread::sleep, time::Duration};

use super::*;
use pairwriter::prelude::*;

pub fn client_connect(lua: &Lua, (url,username): (String,String)) -> LuaResult<()> {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(connect_as_client(url, username))

    });
    sleep(Duration::from_millis(3000));
    lua.globals().call_function("print", "Connected to server")?;
    export_functions::export_cmds(lua)?;
    hooks::start_hooks(lua)?;

    Ok(())
}

fn bufname_to_relativepath(path: impl Into<String>) -> String {
    let path: String = path.into();
    path.replacen("pairwriter://", "",1)
}
fn relativepath_to_bufname(path: impl Into<String>) -> String {
    let path: String = path.into();
    format!("pairwriter://{}", path)
}

mod export_functions;
mod hooks;
