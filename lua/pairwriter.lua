-- compile rust program and add it to the path

local pairwriter_dir = vim.fn.expand("~/.local/share/nvim/pairwriter")


package.cpath = pairwriter_dir .. "/?.so" .. ";" .. package.cpath

return require("pairwriter_helper")
