local M = {}

-- See https://github.com/tjdevries/rofl.nvim/blob/632c10f2ec7c56882a3f7eda8849904bcac6e8af/lua/rofl.lua
local binary_path = vim.fn.fnamemodify(
  vim.api.nvim_get_runtime_file("lua/sniprun.lua", false)[1], ":h:h")
  .. "/target/release/sniprun"

local sniprun_path = vim.fn.fnamemodify( vim.api.nvim_get_runtime_file("lua/sniprun.lua", false)[1], ":p:h") .. "/.." 



-- default config
M.config_values = {
  selected_interpreters = {},
  repl_enable = {},
  repl_disable = {},

  interpreter_options = {
    ["example_original"] = {
      example_option = 2,
    }
  },

  inline_messages = 0
}

function M.load_vimscript_config()
  vimscript_config = {}
  vimscript_config["repl_enable"] = vim.g.SnipRun_repl_behavior_enable or M.config_values["repl_enable"]
  vimscript_config["repl_disable"] = vim.g.SnipRun_repl_behavior_disable or M.config_values["repl_disable"]
  vimscript_config["selected_interpreters"] = vim.g.SnipRun_select_interpreters or M.config_values["selected_interpreters"]
  vimscript_config["inline_messages"] = vim.g.SnipRun_inline_messages or M.config_values["inline_messages"]

  return vimscript_config
end
  


function M.setup(opts)
  opts = opts or M.load_vimscript_config()
  if next(opts) == nil then return end
  for key,value in pairs(opts) do
    if M.config_values[key] == nil then
      error(string.format('[Sniprun] Key %s not exist in config values',key))
      return
    end
    if type(M.config_values[key]) == 'table' then
      for k,v in pairs(value) do
        if type(M.config_values[key][k]) == 'table' then
          for k2,v2 in pairs(v) do
            M.config_values[key][k][k2] = v2
          end
        else
          M.config_values[key][k] = v
        end
      end
    else
      M.config_values[key] = value
    end
  end
end


local function start()
  if M.job_id ~= nil then return end
  M.job_id = vim.fn.jobstart({ binary_path }, { rpc = true })
end

function M.notify(method, ...)
  start()
  vim.rpcnotify(M.job_id, method, ...)
end

function M.run()
  range_from_lua_begin, range_from_lua_end = M.get_range() 
  range_begin = range_b or range_from_lua_begin
  range_end = range_e or range_from_lua_end
  M.config_values["sniprun_root_dir"] = sniprun_path
  M.notify('run', range_begin, range_end, M.config_values)
end


-- this does not yet works great with lua/nvim
function M.get_range() 
  local _, csrow, cscol, _ = unpack(vim.fn.getpos("'<"))
  local _, cerow, cecol, _ = unpack(vim.fn.getpos("'>"))
  if csrow < cerow or (csrow == cerow and cscol <= cecol) then
    return csrow , cerow
  else
    return cerow , csrow 
  end
end




function M.clean()
  notify("clean")
  vim.wait(200) -- let enough time for the rust binary to delete the cache before killing its process
  M.terminate()
end
  
function M.clear_repl()
  norify("clearrepl")
end

function M.terminate()
  vim.fn.jobstop(M.job_id)
  M.job_id = nil
end

function M.test()
  print(vim.api.nvim_get_mode()['mode'])
end

return M
