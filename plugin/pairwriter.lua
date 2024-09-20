-- compile rust program and add it to the path
local path = vim.fn.expand(
 "~/.local/share/nvim/pairwriter/pairwriter_helper.so"
)
local pairwriter_dir = vim.fn.expand("~/.local/share/nvim/pairwriter")

local cmd = string.format(
 "mkdir -p %s && cd %s && cargo b -r && cp target/release/libpairwriter_helper.dylib %s",
  pairwriter_dir,
 "pairwriter_helper",
  path
)

if vim.fn.has("win32") == 1 then
  path = vim.fn.expand(
    "~\\AppData\\Local\\nvim\\site\\pack\\pairwriter\\pairwriter-lua.so"
  )
  -- command for windows 
  cmd = string.format(
    "cd %s && cargo b -r && copy target\\release\\pairwriter.exe %s",
    "pairwriter_lua",
    path
  )
end

if vim.fn.empty(vim.fn.glob(pairwriter_dir))  then
  -- run cmd to compile the rust program
  vim.fn.system(cmd)
end

-- set shellslash for windows 
if vim.fn.has("win32") == 1 then
  vim.o.shellslash = true
else

package.cpath = pairwriter_dir.."/?.so" .. ";".. package.cpath
end



return require("pairwriter_helper")
