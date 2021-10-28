local M = {}
M.listeners = {}
M.closers = {}

local sniprun = require('sniprun')
local sniprun_path = vim.fn.fnamemodify( vim.api.nvim_get_runtime_file("lua/sniprun.lua", false)[1], ":p:h") .. "/.."

function M.run_range(range_start, range_end, filetype, config)
  local override = {}
  override.filetype = filetype
  local lconfig = config or sniprun.config_values
  lconfig["sniprun_root_dir"] = sniprun_path
  sniprun.notify('run', range_start, range_end, lconfig, "", override)
end


function M.run_string(codestring, filetype, config)
  local override = {}
  override.codestring = codestring
  override.filetype = filetype or ""
  local lconfig = config or sniprun.config_values
  lconfig["sniprun_root_dir"] = sniprun_path
  sniprun.notify('run', 0, 0, lconfig, "", override)
end


function M.register_listener(f)
    if type(f) ~= 'function' then
	print("Only functions can be registered")
    end
    assert(type(f) == 'function')
    table.insert(M.listeners, f)
end

function M.register_closer(f)
    if type(f) ~= 'function' then
	print("Only functions can be registered")
    end
    assert(type(f) == 'function')
    table.insert(M.closers, f)
end


return M
