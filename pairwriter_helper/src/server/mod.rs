use std::sync::OnceLock;

use pairwriter::{prelude::RPC, server_import::*};

use super::*;

lazy_static::lazy_static! {
    static ref working_dir: String = std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            .replace("\\", "/")
            + "/"; // this would work if you open it in the root directory (who does that?)

}
static ONCE: OnceLock<()> = OnceLock::new();
pub fn lua_start_server(lua: &Lua, port: u16) -> LuaResult<LuaTable> {
    if ONCE.get().is_none() {
        let _ = ONCE.set(());


        std::thread::spawn(move || {
            // this is the only why that works for now
            let rt = Runtime::new().unwrap();
            rt.block_on(start_server(port));
        });

        let out_funcs = lua.create_table()?;
        // here we should hooks to the nvim
        hooks::start_hooks(lua)?;
        exported_functions::export_cmds(lua)?;

        Ok(out_funcs)
    } else {
        Err(mlua::Error::RuntimeError(
            "Server already started".to_string(),
        ))
    }
}
pub fn undo(lua: &Lua, path: String) -> LuaResult<()> {
    RT.block_on(async {
        server_api
            .lock()
            .await
            .send_rpc(RPC::Undo { path: path.clone() })
            .await;
    });
    lua.load(format!(
        r#"
        vim.schedule(function()
            vim.api.nvim_command('e! ' .. '{path}')
        end)
        
        "#
    ))
    .exec()?;
    Ok(())
}
pub fn redo(lua: &Lua, path: String) -> LuaResult<()> {
    RT.block_on(async {
        server_api
            .lock()
            .await
            .send_rpc(RPC::Redo { path: path.clone() })
            .await;
    });
    lua.load(format!(
        r#"
        vim.schedule(function()
            vim.api.nvim_command('e! ' .. '{path}')
        end)
        
        "#
    ))
    .exec()?;
    Ok(())
}
mod hooks;

mod exported_functions;
