use super::*;

pub(super) fn export_cmds(lua: &Lua) -> LuaResult<()> {
    // as

    macro_rules! commands {
        ($lua:ident,
                $(
                ($command_name:expr , $command:ident , $auto_complete:ident)
                ,)+
                +
                $(
                    ($command_name2:expr, $command2:ident)
                ,)*
                ) => {
                let vim: LuaTable = $lua.globals().get("vim")?;
                let vim_api: LuaTable = vim.get("api")?;
                $(
                    let command = $lua.create_function($command)?;
                    let opts = $lua.create_table()?;
                    opts.set("complete", $lua.create_function($auto_complete)?)?;
                    opts.set("nargs", 1)?;
                    vim_api.call_function(
                        "nvim_create_user_command",
                        ($command_name, command, opts),
                    )?;
                )+
                $(
                    let command = $lua.create_function($command2)?;
                    let opts = $lua.create_table()?;
                    opts.set("nargs", 0)?;
                    vim_api.call_function(
                        "nvim_create_user_command",
                        ($command_name2, command, opts),
                    )?;

                )*

        };
    }
    commands!(
        lua,
        ("PairwriterRemoveFile", remove_file, auto_complete_file),
        ("PairwriterMoveFile", move_file, auto_complete_file),
        ("PairwriterCreateDir", create_dir, auto_complete_dir),
        ("PairwriterRemoveDir", remove_dir, auto_complete_dir),
        ("PairwriterMoveDir", move_dir, auto_complete_dir),
        ("PairwriterOpenFile", open_file, auto_complete_file),
        ("PairwriterCreateFile", create_file, auto_complete_file),
        +
        ("PairwriterSaveFile", save_file),
        ("PairwriterShowPrevildege", show_previledge),
    );

    Ok(())
}

// funciton for moving the file
fn move_file(_lua: &Lua, args: LuaTable) -> LuaResult<()> {
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
fn create_file(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let path = input.get::<_, String>("args")?.trim_end().to_string();
    std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            client_api
                .get()
                .unwrap()
                .lock()
                .await
                .send_rpc(RPC::CreateFile { path })
                .await;
        });
    });

    Ok(())
}
fn remove_file(lua: &Lua, path: LuaTable) -> LuaResult<()> {
    let path: String = path.get::<_,String>("args")?.trim_end().to_string();
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
                client_api
                    .get()
                    .unwrap()
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
    let relative_path: String = input.get::<_,String>("args")?.trim_end().to_string();
    let res = RT
        .block_on(async {
            let mut api = client_api.get().unwrap().lock().await;
            api.read_file(relative_path.clone()).await // this will request the buffer and if
                                                       // the buffer is preset will do nothing
        })
        .is_err();

    lua.load(format!(
        r#"
            vim.schedule(function()
                local buffer_name = "pairwriter://" .. "{relative_path}"

                if vim.fn.bufexists(buffer_name) == 1 then 
                    vim.api.nvim_command('buffer ' .. vim.fn.bufnr(buffer_name))
                    return 
                end
                if {res} then -- if the buffer is not here wait for 300ms
                    vim.uv.sleep(300) 
                end
                local buf = vim.api.nvim_create_buf(true, true)
                vim.api.nvim_set_option_value('buftype', 'nofile', {{ buf = buf }})
                vim.api.nvim_set_option_value('bufhidden', 'hide', {{ buf = buf }})
                vim.api.nvim_set_current_buf(buf)
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
fn show_previledge(lua: &Lua,_:()) -> LuaResult<()> {
    let k  = RT.block_on(async {
            match client_api.get().unwrap().lock().await.priviledge{
                Priviledge::ReadWrite => {
                    "write"
                }
                Priviledge::ReadOnly => {
                    "read"
                }
            }
        });
    lua.globals().call_function("print",k)?;
    Ok(())

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
