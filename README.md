# pairwriter.nvim

pair programming

## Description

**pairwriter.nvim** is a pair programming plugin designed to work seamlessly with Neovim. It is written in Rust and supports both Neovim and VSCode (VSCode version still in work). This plugin leverages **automerge-rs** for real-time collaborative editing and **mlua** to integrate with Neovim's Lua ecosystem.

## Installation

To install **pairwriter.nvim**, ensure you have the following dependencies:

- Rust
- Lua 5.1
- Neovim 0.9.0

### Steps:

1. Add `m10090/pairwriter.nvim` to your package manager.
2. Compile the Rust code in `pairwriter_helper`.
3. Move the compiled program to:

   - For Unix: `~/.local/share/nvim/pairwriter/pairwriter.so`
   - For Windows: `~\AppData\Local\share\nvim-data\pairwriter\pairwriter.dll`

   **Note:** When compiling on Windows, add the path to `lua51.dll` in the environment variable `LUA_LIB`. This DLL is typically found in `C:\Program Files\Neovim\bin\`.

4. Finally, run the setup function.

Does this match what you had in mind? Any adjustments or additions?


## Usage

To use **pairwriter.nvim**, follow these instructions:

### Starting the Server

Run the following command in Neovim:

```lua
require('pairwriter').start_server(port)
```

This starts the server, allowing you to use Neovim as normal without any additional configuration.

### Connecting as a Client

To connect as a client, use the following command:

```lua
require('pairwriter').connect_as_client(websocket_url, username)
```

After connecting, you can utilize the Pairwriter command palette. The command you'll need to get started right away is:

```lua
PairwriterOpenfile ./{filepath}
```

This command allows you to open files within the pair programming session.


How does that look? Any changes or further details you’d like to add?

# Disclaimer

This project is in early development. Features may change, and there may be bugs. Your feedback and contributions are welcome as we work towards improving pairwriter.nvim.

# License

MIT
