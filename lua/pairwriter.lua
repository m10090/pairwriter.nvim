-- compile rust program and add it to the path

local M = {}

local function copy_function_pointers(src, dest)
	for key, value in pairs(src) do
		if type(value) == "function" then
			dest[key] = value -- Copy function pointer
		end
	end
end
M.setup = function(t)
	t = t or {}
	t.username = t.username or "SERVER"
	local old_pacakge_cpath = package.cpath

	local pairwriter_dir
	if vim.fn.has("unix") == 1 then
		-- unix use
		t.dir_path = t.dir_path or "~/.local/share/nvim/pairwriter"
		pairwriter_dir = vim.fn.expand(t.dir_path)
		package.cpath = pairwriter_dir .. "/?.so" .. ";" .. package.cpath
		-- set the log file path
		vim.cmd("let $LOGFILE='" .. pairwriter_dir .. "/log.log'")
	else
		-- windows
		t.dir_path = t.dir_path or "~\\AppData\\.local\\share\\nvim-data\\pairwriter"
		pairwriter_dir = vim.fn.expand(t.dir_path)
		package.cpath = pairwriter_dir .. "\\?.dll;" .. package.cpath
		-- set the log file path
		vim.cmd("let $LOGFILE='" .. pairwriter_dir .. "\\log.log'")
	end

	vim.cmd("let $SERVER_USERNAME='" .. t.username .. "'")


	copy_function_pointers(require("pairwriter_helper"), M)
	package.cpath = old_pacakge_cpath -- reset the package path
end

return M
