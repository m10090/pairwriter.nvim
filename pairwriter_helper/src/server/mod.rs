use std::{borrow::Cow, fmt::Formatter, fs, sync::OnceLock, thread::sleep, time::Duration};

use super::*;

lazy_static::lazy_static! {
    static ref working_dir: String = std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/"; // this would work if you open it in the root directory (who does that?)

}
static ONCE: OnceLock<()> = OnceLock::new();
pub fn lua_start_server(lua: &Lua, port: u16) -> LuaResult<LuaTable> {
    if ONCE.get().is_none() {
        let _ = ONCE.set(());

        #[cfg(target_os = "windows")]
        {
            working_dir = working_dir.replace("\\", "/");
        }

        std::thread::spawn(move || {
            // this is the only why that works for now
            let rt = Runtime::new().unwrap();
            rt.block_on(start_server(port));
        });

        let out_funcs = lua.create_table()?;
        // here we should hooks to the nvim
        hooks::start_hooks(lua)?;

        Ok(out_funcs)
    } else {
        Err(mlua::Error::RuntimeError(
            "Server already started".to_string(),
        ))
    }
}

mod hooks;

mod exported_functions;
