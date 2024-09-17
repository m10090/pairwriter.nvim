use super::*;

use std::{io::BufRead, panic};

pub(super) fn start_hooks(lua: &Lua) -> LuaResult<()> {
    let vim = lua.globals().get::<_, LuaTable>("vim")?;
    let vim_api = vim.get::<_, LuaTable>("api")?;
    let augroup: i32 =
        vim_api.call_function("nvim_create_augroup", ("PairWriter", lua.create_table()))?;

    let common_table = lua.create_table()?;
    common_table.set("group", augroup)?;
    common_table.set("pattern", "pairwriter://./*")?;

    open_file_hook(lua, &common_table)?;
    file_edit_hook(lua, &common_table)?;

    lua.globals().set(
        "__outside_edit_hook",
        lua.create_thread(lua.create_async_function(outside_edit_hook)?)?,
    )?;

    lua.load(
        r#"
        local timer = vim.uv.new_timer()
        timer:start(0, 300, function()
            coroutine.resume(__outside_edit_hook)
        end)
        "#,
    )
    .exec()?;

    Ok(())
}

pub fn file_edit_hook(lua: &Lua, common_table: &LuaTable) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;

    let insert_change_table = lua.create_table()?;
    shallow_copy(common_table, &insert_change_table)?;

    insert_change_table.set(
        "callback",
        lua.create_function(move |lua, env: LuaTable| {
            let bufnr: i32 = env.get("buf")?;
            let path: String = env.get("file")?;

            let bufname = bufname_to_relativepath(path);

            let vim_api: LuaTable = lua.globals().get::<_, LuaTable>("vim")?.get("api")?;

            let lines: LuaTable =
                vim_api.call_function("nvim_buf_get_lines", (bufnr, 0, -1, false))?;

            let text = create_txt_from_lines(lines);
            RT.block_on(async move {
                let client_api_ = &mut client_api.get().unwrap().lock().await;
                if client_api_.priviledge == Priviledge::ReadOnly {
                    // print error you don't have the permission to write
                    lua.globals()
                        .call_function("error", "you don't have the permission to write")?;
                } else {
                    client_api_
                        .edit_buf(bufname, None, None, text.as_str())
                        .await;
                }
                Ok::<(), mlua::Error>(())
            })?;

            Ok(())
        })?,
    )?;

    vim_api.call_function("nvim_create_autocmd", ("InsertLeave", insert_change_table))?;
    Ok(())
}

pub(super) fn open_file_hook(lua: &Lua, common_table: &LuaTable) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;

    let open_file_table = lua.create_table()?;
    shallow_copy(common_table, &open_file_table)?;

    open_file_table.set(
        "callback",
        lua.create_function(move |lua, env: LuaTable| {
            let bufnr: i32 = env.get("buf")?;
            let path: String = env.get("file")?;

            // change path to relative path

            let relative_path = bufname_to_relativepath(path);

            let vim_api: LuaTable = lua.globals().get::<_, LuaTable>("vim")?.get("api")?;

            let text = RT.block_on(async move {
                client_api
                    .get()
                    .unwrap()
                    .lock()
                    .await
                    .read_file(relative_path)
                    .await
            });
            match text {
                Ok(text) => {
                    let lines = text_to_lines(lua, text.as_slice())?;
                    vim_api.call_function("nvim_buf_set_lines", (bufnr, 0, -1, false, lines))?;
                }
                Err(e) => {
                    lua.globals().call_function("print", e.to_string())?;
                }
            }
            Ok(())
        })?,
    )?;

    vim_api.call_function("nvim_create_autocmd", ("BufEnter", open_file_table))?;

    Ok(())
}

async fn outside_edit_hook(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut receiver = client_api
        .get()
        .unwrap()
        .lock()
        .await
        .get_receiver()
        .unwrap();

    while let Some(x) = receiver.recv().await {
        if let RPC::EditBuffer { path, .. } = x {
            let bufname = relativepath_to_bufname(path.clone());

            let to_be_called = lua.create_function(move |lua, _: ()| {
                let vim: LuaTable = lua.globals().get("vim")?;
                let vim_fn: LuaTable = vim.get("fn")?;
                let vim_api: LuaTable = vim.get("api")?;
                let _: () = lua.globals().call_function("print", bufname.clone())?;

                let bufnr: i32 = vim_fn.call_function("bufnr", (bufname.clone(),))?;
                if bufnr != -1 {
                    // in the case of the file is  opened
                    let path = path.clone();
                    let file = RT.block_on(async move {
                        let text = client_api
                            .get()
                            .unwrap()
                            .lock()
                            .await
                            .read_file(path)
                            .await?;
                        text_to_lines(lua, text.as_slice())
                    })?;
                    vim_api.call_function("nvim_buf_set_lines", (bufnr, 0, -1, false, file))?;
                }
                Ok(())
            })?;
            let vim: LuaTable = lua.globals().get("vim")?;
            vim.call_function("schedule", to_be_called)?;
        }
    }
    Ok(())
}
