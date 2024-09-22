-- compile rust program and add it to the path

local M = {}

local pairwriter_dir

local function copy_function_pointers(src, dest)
	for key, value in pairs(src) do
		if type(value) == "function" then
			dest[key] = value -- Copy function pointer
		end
	end
end
M.setup = function(t)
	t = t or {}
  local old_pacakge_cpath = package.cpath
	if vim.fn.has("unix") == 1 then
		-- unix use
		local pairwriter_dir_path = t.dir_path or "~/.local/share/nvim/pairwriter"
		pairwriter_dir = vim.fn.expand(pairwriter_dir_path)
		package.cpath = pairwriter_dir .. "/?.so" .. ";" .. package.cpath
    -- set he log file path
		vim.cmd("let $LOGFILE='" .. pairwriter_dir .. "/log.log'")
	else
		-- windows
		local pairwriter_dir_path = t.dir_path or "~\\AppData\\.local\\share\\nvim-data\\pairwriter" -- i don't know windows correct path
		pairwriter_dir = vim.fn.expand(pairwriter_dir_path)
		package.cpath = pairwriter_dir .. "/?.dll;" .. package.cpath
    -- set he log file path
		vim.cmd("let $LOGFILE='" .. pairwriter_dir .. "\\log.log'")
	end
  t.username = t.username or "SERVER"
	vim.cmd("let $SERVER_USERNAME='" .. t.username .."'")
	copy_function_pointers(require("pairwriter_helper"), M)
  package.cpath = old_pacakge_cpath -- reset the package path 
end

return M
