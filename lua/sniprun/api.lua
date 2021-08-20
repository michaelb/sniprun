local M = {}

local sniprun = require('sniprun')
local sniprun_path = vim.fn.fnamemodify( vim.api.nvim_get_runtime_file("lua/sniprun.lua", false)[1], ":p:h") .. "/.."

function M.run_range(range_start, range_end, filetype)
  local override = {}
  override.filetype = filetype
  sniprun.config_values["sniprun_root_dir"] = sniprun_path
  sniprun.notify('run', range_start, range_end, sniprun.config_values, override)
end


function M.run_string(codestring, filetype)
  local override = {}
  override.codestring = codestring
  override.filetype = filetype or ""
  sniprun.config_values["sniprun_root_dir"] = sniprun_path
  sniprun.notify('run', 0, 0, sniprun.config_values, override)
end

return M
