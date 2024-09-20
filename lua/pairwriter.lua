-- compile rust program and add it to the path

local M = {}

local pairwriter_dir

M.setup = function(t)
	-- if unix use
	if vim.fn.has("unix") == 1 then
		local pairwriter_dir_path = t.dir_path or "~/.local/share/nvim/pairwriter"
		pairwriter_dir = vim.fn.expand(pairwriter_dir_path)
		package.cpath = pairwriter_dir .. "/?.so" .. ";" .. package.cpath
		require("pairwriter_helper")
	else
		local pairwriter_dir_path = t.dir_path or "~/.local/share/nvim/pairwriter"
		pairwriter_dir = vim.fn.expand(pairwriter_dir_path)
		package.cpath = pairwriter_dir .. "/?.so" .. ";" .. package.cpath
	end
	require("pairwriter_helper")
end

return M
