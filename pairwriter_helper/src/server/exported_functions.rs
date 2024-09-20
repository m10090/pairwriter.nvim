use super::*;
pub(super) fn export_cmds(lua: &Lua) -> LuaResult<()> {
    // as
    macro_rules! commands {
        ($lua:ident,
                $(
                ($command_name:expr , $command:ident , $auto_complete:ident)
                )+
                +
                $(
                    ($command_name2:expr, $command2:ident)
                )*
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
        ("PairwriterRemoveFile", remove_file, auto_complete_file)(
            "PairwriterMoveFile",
            move_file,
            auto_complete_file
        )("PairwriterCreateDir", create_dir, auto_complete_dir)(
            "PairwriterRemoveDir",
            remove_dir,
            auto_complete_dir
        )("PairwriterMoveDir", move_dir, auto_complete_dir)(
            "PairwriterDisconnectUser",
            disconnect_user,
            auto_complete_users
        )(
            "PairwriterChangePreviledge",
            change_prevlidge,
            auto_complete_users
        ) + ("PairwriterListUsers", list_users)
    );

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
                server_api
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

fn remove_file(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let mut fargs: Vec<String> = input
        .get::<_, String>("args")?
        .split(" ")
        .map(|x| x.to_string())
        .collect();
    let path = fargs.pop().unwrap();
    {
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

fn remove_dir(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let relative_path: String = input.get("args")?;

    RT.block_on(async {
        let mut api = server_api.lock().await;
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
                server_api
                    .lock()
                    .await
                    .send_rpc(RPC::MoveDirectory {
                        path: from,
                        new_path: to,
                    })
                    .await;
            })
        });
        Ok(())
    }
}

fn create_dir(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let relative_path: String = input.get::<_,String>("args")?.trim_end().to_string();

    RT.block_on(async {
        let mut api = server_api.lock().await;
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
        let api = server_api.lock().await;
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
fn auto_complete_users(_lua: &Lua, arg_lead: String) -> LuaResult<Vec<String>> {
    RT.block_on(async {
        let api = server_api.lock().await;
        let users = api.list_users().await;
        if let Err(start) = users.binary_search(&arg_lead) {
            // check if the directory is empty
            let mut r = users.len();
            let mut l = start;
            // binary search for the end of the directory
            while l < r {
                let mid = l + (r - l) / 2;
                if users[mid].starts_with(&arg_lead) {
                    l = mid + 1;
                } else {
                    r = mid;
                }
            }
            let end = r;
            Ok(users[start..end].to_vec())
        } else {
            Ok(vec![])
        }
    })
}

fn list_users(_lua: &Lua, _: ()) -> LuaResult<Vec<String>> {
    Ok(RT.block_on(async {
        let api = server_api.lock().await;
        api.list_users().await
    }))
}

fn disconnect_user(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let user = input.get::<_, String>("args")?.trim_end().to_string();
    RT.block_on(async {
        let api = server_api.lock().await;
        let _ = api.close_connection(user.as_str()).await;
    });
    Ok(())
}
fn change_prevlidge(_lua: &Lua, input: LuaTable) -> LuaResult<()> {
    let farg = input
        .get::<_, String>("args")?
        .split(" ")
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let user = farg[0].to_string();
    let privileged = farg[1].to_string();
    RT.block_on(async {
        let api = server_api.lock().await;
        let _ = api
            .change_priviledge(
                &user,
                match privileged.as_str() {
                    "read" => Priviledge::ReadOnly,
                    "write" => Priviledge::ReadWrite,
                    _ => Priviledge::ReadOnly,
                },
            )
            .await;
    });
    Ok(())
}
