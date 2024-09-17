use super::*;

pub(super) fn export_cmds(lua: &Lua) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;

    // Open file command
    let command = lua.create_function(open_file)?;
    let opts = lua.create_table()?;
    opts.set("complete", lua.create_function(auto_complete_file)?)?;
    opts.set("nargs", 1)?;
    vim_api.call_function(
        "nvim_create_user_command",
        ("PairwriterOpenFile", command, opts),
    )?;
    // End open file command

    let command = lua.create_function(create_file)?;
    let opts = lua.create_table()?;
    opts.set("nargs", 1)?;

    vim_api.call_function(
        "nvim_create_user_command",
        ("PairwriterCreateFile", command, opts),
    )?;

    // Save file command

    let command = lua.create_function(save_file)?;
    let opts = lua.create_table()?;
    opts.set("nargs", 0)?;
    vim_api.call_function(
        "nvim_create_user_command",
        ("PairwriterSaveFile", command, opts),
    )?;

    // End save file command

    // remove file command
    let command = lua.create_function(remove_file)?;
    let opts = lua.create_table()?;
    opts.set("complete", lua.create_function(auto_complete_file)?)?;
    opts.set("nargs", 1)?;
    vim_api.call_function(
        "nvim_create_user_command",
        ("PairwriterRemoveFile", command, opts),
    )?;
    // End remove file command

    // move file command
    let command = lua.create_function(move_file)?;
    let opts = lua.create_table()?;
    opts.set("complete", lua.create_function(auto_complete_file)?)?;
    opts.set("nargs", 1)?;
    vim_api.call_function(
        "nvim_create_user_command",
        ("PairwriterMoveFile", command, opts),
    )?;
    // End move file command

    // create directory command

    let command = lua.create_function(create_dir)?;
    let opts = lua.create_table()?;
    opts.set("complete", lua.create_function(auto_complete_dir)?)?;
    opts.set("nargs", 1)?;
    vim_api.call_function(
        "nvim_create_user_command",
        ("PairwriterCreateDir", command, opts),
    )?;

    // End create directory command

    // remove directory command

    let command = lua.create_function(remove_dir)?;
    let opts = lua.create_table()?;

    opts.set("complete", lua.create_function(auto_complete_dir)?)?;
    opts.set("nargs", 1)?;

    vim_api.call_function(
        "nvim_create_user_command",
        ("PairwriterRemoveDir", command, opts),
    )?;

    // End remove directory command

    // move directory command

    let command = lua.create_function(move_dir)?;
    let opts = lua.create_table()?;

    opts.set("complete", lua.create_function(auto_complete_dir)?)?;
    opts.set("nargs", 1)?;

    vim_api.call_function(
        "nvim_create_user_command",
        ("PairwriterMoveDir", command, opts),
    )?;

    // End move directory command

    Ok(())
}

// funciton for moving the file
fn move_file(lua: &Lua, args: LuaTable) -> LuaResult<()> {
    // if args.get::<_, i32>("nargs")? != 2 {
    //     return Err(LuaError::external("expected 2 arguments"));
    // }
    let mut fargs: Vec<String> = args
        .get::<_, String>("args")?
        .split(" ")
        .map(|x| x.to_string())
        .collect();
    let to = fargs.pop().unwrap();
    let from = fargs.pop().unwrap();
    // move file from one location to another
    {
        let to = to.clone();
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            rt.block_on(async move {
                client_api
                    .get()
                    .unwrap()
                    .lock()
                    .await
                    .send_rpc(RPC::MoveFile {
                        path: from,
                        new_path: to,
                    })
                    .await;
            });
        });
    }
    lua.load(format!(
        r#"
            vim.schedule(function()
                vim.api.nvim_command('bd!' .. 'Pairwriter://' .. '{to}')
            end)
        
        "#
    ))
    .exec()?;
    Ok(())
}

fn save_file(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim: LuaTable = lua.globals().get("vim")?;
    let vim_api: LuaTable = vim.get("api")?;
    let path: String =
        bufname_to_relativepath(vim_api.call_function::<_, String>("nvim_buf_get_name", 0)?);
    lua.globals().call_function("print", path.clone())?;

    std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            client_api
                .get()
                .unwrap()
                .lock()
                .await
                .send_rpc(RPC::ReqSaveFile { path })
                .await;
        });
    });
    Ok(())
}
fn create_file(_lua: &Lua, path: String) -> LuaResult<()> {
    std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            server_api
                .lock()
                .await
                .send_rpc(RPC::CreateFile { path })
                .await;
        });
    });

    Ok(())
}
fn remove_file(lua: &Lua, path: String) -> LuaResult<()> {
    // remove file from the filesystem
    lua.load(format!(
        r#"
            vim.schedule(function()
                vim.api.nvim_command('bd! {path}')
            end)
        "#,
    ))
    .exec()?;
    {
        let path = path.clone();
        std::thread::spawn(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                server_api
                    .lock()
                    .await
                    .send_rpc(RPC::DeleteFile { path })
                    .await;
            });
        });
    }

    Ok(())
}

fn open_file(lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let relative_path: String = input.get("args")?;
    RT.block_on(async {
        let mut api = client_api.get().unwrap().lock().await;
        let _ = api.read_file(relative_path.clone()).await; // this will request the buffer
    });
    lua.load(format!(
        r#"
            vim.schedule(function()
                local buf = vim.api.nvim_create_buf(true, true)
                vim.api.nvim_set_option_value('buftype', 'nofile', {{ buf = buf }})
                vim.api.nvim_set_option_value('bufhidden', 'hide', {{ buf = buf }})
                vim.api.nvim_set_current_buf(buf)
                local buffer_name = "pairwriter://" .. "{relative_path}"
                vim.api.nvim_buf_set_name(buf, buffer_name)

                vim.api.nvim_command('e!')
            end)
        "#,
    ))
    .exec()?;

    Ok(())
}

fn remove_dir(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let relative_path: String = input.get("args")?;

    RT.block_on(async {
        let mut api = client_api.get().unwrap().lock().await;
        api.send_rpc(RPC::DeleteDirectory {
            path: relative_path,
        })
        .await;
        Ok(())
    })
}

fn move_dir(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let mut fargs: Vec<String> = input
        .get::<_, String>("args")?
        .split(" ")
        .map(|x| x.to_string())
        .collect();
    let to = fargs.pop().unwrap();
    let from = fargs.pop().unwrap();
    // move file from one location to another
    {
        let to = to.clone();
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            rt.block_on(async move {
                client_api
                    .get()
                    .unwrap()
                    .lock()
                    .await
                    .send_rpc(RPC::MoveDirectory {
                        path: to,
                        new_path: from,
                    })
                    .await;
            })
        });
        Ok(())
    }
}

fn create_dir(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let relative_path: String = input.get("args")?;

    RT.block_on(async {
        let mut api = client_api.get().unwrap().lock().await;
        api.send_rpc(RPC::CreateDirectory {
            path: relative_path,
        })
        .await;
        Ok(())
    })
}

fn auto_complete_dir(_lua: &Lua, _arg_lead: String) -> LuaResult<Vec<String>> {
    // todo
    Ok(vec![])
}
fn auto_complete_file(_lua: &Lua, arg_lead: String) -> LuaResult<Vec<String>> {
    RT.block_on(async {
        let api = client_api.get().unwrap().lock().await;
        let (files, _) = api.get_file_maps().await;
        if let Err(start) = files.binary_search(&arg_lead) {
            // check if the directory is empty
            let mut r = files.len();
            let mut l = start;
            // binary search for the end of the directory
            while l < r {
                let mid = l + (r - l) / 2;
                if files[mid].starts_with(&arg_lead) {
                    l = mid + 1;
                } else {
                    r = mid;
                }
            }
            let end = r;
            Ok(files[start..end].to_vec())
        } else {
            Ok(vec![])
        }
    })
}
