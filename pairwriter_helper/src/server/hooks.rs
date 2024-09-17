use super::*;

use std::panic;


pub(super) fn start_hooks(lua: &Lua) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;

    let augroup: i32 =
        vim_api.call_function("nvim_create_augroup", ("PairWriter", lua.create_table()))?;

    let common_table = lua.create_table()?;
    common_table.set("group", augroup)?;
    common_table.set("pattern", working_dir.clone() + "*")?;

    open_file_hook(lua, &common_table)?;
    file_edit_hook(lua, &common_table)?;
    file_save_hook(lua, &common_table)?;
    create_file_hook(lua, &common_table)?;

    lua.globals().set(
        "__outside_edit_hook",
        lua.create_thread(lua.create_async_function(outside_edit_hook)?)?,
    )?;

    lua.load(
        r#"
        local timer = vim.uv.new_timer()
        timer:start(0, 100, function()
            coroutine.resume(__outside_edit_hook)
        end)
        "#,
    )
    .exec()?;
    Ok(())
}

fn file_edit_hook(lua: &Lua, common_table: &LuaTable) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;

    let insert_change_table = lua.create_table()?;
    shallow_copy(common_table, &insert_change_table)?;

    insert_change_table.set(
        "callback",
        lua.create_function(move |lua, env: LuaTable| {
            let bufnr: i32 = env.get("buf")?;
            let path: String = env.get("file")?;

            // change path to relative path
            let relative_path = path.replacen(&*working_dir, "./", 1);

            let vim_api: LuaTable = lua.globals().get::<_, LuaTable>("vim")?.get("api")?;

            let lines: LuaTable =
                vim_api.call_function("nvim_buf_get_lines", (bufnr, 0, -1, false))?;

            let text = create_txt_from_lines(lines);

            let res = panic::catch_unwind(|| {
                RT.block_on(async move {
                    let text = text; // move text to the async block
                    server_api
                        .lock()
                        .await
                        .edit_buf(relative_path, None, None, &text)
                        .await;
                })
            });
            if res.is_err() {
                lua.globals().call_function("print", "Panic in edit_buf")?;
            }

            // check if rt paniced
            Ok(())
        })?,
    )?;
    vim_api.call_function("nvim_create_autocmd", ("InsertLeave", insert_change_table))?;
    Ok(())
}

fn open_file_hook(lua: &Lua, common_table: &LuaTable) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;

    let open_change_table = lua.create_table()?;
    shallow_copy(common_table, &open_change_table)?;

    open_change_table.set(
        "callback",
        lua.create_function(move |lua, env: LuaTable| {
            let path: String = env.get("file")?;
            let relative_path = path.replacen(&*working_dir, "./", 1);
            let _ = RT.block_on(async move {
                let res = server_api
                    .lock()
                    .await
                    .read_file_server(relative_path)
                    .await;
                if res.is_err() {
                    let _: LuaResult<()> =
                        lua.globals().call_function("print", "Error reading file");
                    return Ok::<(), ()>(());
                }
                Ok::<(), ()>(())
            });
            Ok(())
        })?,
    )?;

    vim_api.call_function("nvim_create_autocmd", ("BufEnter", open_change_table))?;

    Ok(())
}
fn create_file_hook(lua: &Lua, common_table: &LuaTable) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;

    let insert_change_table = lua.create_table()?;

    shallow_copy(common_table, &insert_change_table);

    insert_change_table.set(
        "callback",
        lua.create_function(move |_, env: LuaTable| {
            let path: String = env.get("file")?;

            let relative_path = path.replacen(&*working_dir, "./", 1);

            let _ = RT.block_on(async move {
                server_api
                    .lock()
                    .await
                    .send_rpc(RPC::CreateFile {
                        path: relative_path,
                    })
                    .await;

                Ok::<(), ()>(())
            });
            Ok(())
        })?,
    )?;
    vim_api.call_function("nvim_create_autocmd", ("BufNewFile", insert_change_table))?;
    Ok(())
}

pub fn file_save_hook(lua: &Lua, common_table: &LuaTable) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;

    let insert_change_table = lua.create_table()?;
    shallow_copy(common_table, &insert_change_table)?;

    insert_change_table.set(
        "callback",
        lua.create_function(move |_, env: LuaTable| {
            let path: String = env.get("file")?;

            let relative_path = path.replacen(&*working_dir, "./", 1);

            let _ = RT.block_on(async move {
                server_api
                    .lock()
                    .await
                    .send_rpc(RPC::ReqSaveFile {
                        path: relative_path,
                    })
                    .await;

                Ok::<(), ()>(())
            });
            Ok(())
        })?,
    )?;
    vim_api.call_function("nvim_create_autocmd", ("BufWrite", insert_change_table))?;
    Ok(())
}

async fn outside_edit_hook(lua: &Lua, _: ()) -> LuaResult<()> {

    let mut receiver = server_api.lock().await.take_receiver();

    while let Some(x) = receiver.recv().await {
        if let RPC::EditBuffer { path, .. } = x {
            lua.globals().call_function("print", path.clone())?;
            let relative_path = path; // meaning that the path is already relative
            RT.block_on(async move {
                let text = server_api
                    .lock()
                    .await
                    .read_file_server(relative_path.clone())
                    .await
                    .unwrap();

                let to_be_called = lua.create_function(move |lua, _: ()| {

                    let vim: LuaTable = lua.globals().get("vim")?;
                    let vim_fn: LuaTable = vim.get("fn")?;
                    let vim_api: LuaTable = vim.get("api")?;
                    let _: () = lua
                        .globals()
                        .call_function("print", relative_path.clone())?;
                    let bufnr: i32 = vim_fn.call_function("bufnr", (relative_path.clone(),))?;

                    let file = text_to_lines(lua, text.as_slice());
                    if bufnr != -1 { // if the buffer is opened
                        let _: () = vim_api
                            .call_function("nvim_buf_set_lines", (bufnr, 0, -1, false, file))?;
                    }
                    Ok(())
                })?;
                let vim: LuaTable = lua.globals().get("vim")?;
                vim.call_function("schedule", to_be_called)?;
                Ok::<(), LuaError>(())
            })?;
        }
    }
    Ok(())
}
