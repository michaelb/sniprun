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
  M.configure_keymaps()
end

function M.configure_keymaps()
  vim.api.nvim_set_keymap("v", "<Plug>SnipRun", ":lua require'sniprun'.run('v')<CR>", {silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipRun", ":lua require'sniprun'.run()<CR>",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipRTerminate", ":lua require'sniprun'.terminate()<CR>",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipReset", ":lua require'sniprun'.reset()<CR>",{silent=true})
  -- vim.api.nvim_set_keymap("n", "<Plug>SnipInfo", ":lua require'sniprun'.run()<CR>",{})
  vim.api.nvim_set_keymap("n", "<Plug>SnipReplMemoryClean", ":lua require'sniprun'.clear_repl()<CR>",{silent=true})

end

local function start()
  if M.job_id ~= nil then return end
  M.job_id = vim.fn.jobstart({ binary_path }, { rpc = true })
end

function M.notify(method, ...)
  start()
  vim.rpcnotify(M.job_id, method, ...)
end

function M.run(mode)
  range_begin, range_end = M.get_range(mode)
  M.config_values["sniprun_root_dir"] = sniprun_path
  M.notify('run', range_begin, range_end, M.config_values)
end


function M.get_range(mode) 
  if not mode then
    line1 = vim.api.nvim_win_get_cursor(0)[1]
    line2 = line1
  elseif mode:match("[vV]") then
    line1 = vim.api.nvim_buf_get_mark(0, "<")[1]
    line2 = vim.api.nvim_buf_get_mark(0, ">")[1]
  end
  if line1 > line2 then
    line1, line2 = line2, line1
  end
  return line1, line2
end


function M.reset()
  M.notify("clean")
  vim.wait(200) -- let enough time for the rust binary to delete the cache before killing its process
  M.terminate()
end
  
function M.clear_repl()
  M.notify("clearrepl")
end

function M.terminate()
  vim.fn.jobstop(M.job_id)
  M.job_id = nil
end

-- get all lines from a file, returns an empty 
-- list/table if the file does not exist
local function lines_from(file)
  lines = {}
  for line in io.lines(file) do 
    lines[#lines + 1] = line
  end
  return lines
end

function M.info()
  M.config_values["sniprun_root_dir"] = sniprun_path
  M.notify("info",1,1,M.config_values)

  local sniprun_path = vim.fn.fnamemodify( vim.api.nvim_get_runtime_file("lua/sniprun.lua", false)[1], ":p:h") .. "/.." 

  if M.config_values.inline_messages ~= 0 then
    vim.wait(500) -- let enough time for the sniprun binary to generate the file
    print(" ")
    local lines = lines_from(sniprun_path.."/ressources/infofile.txt")
    -- print all line numbers and their contents
    for k,v in pairs(lines) do
      print(v)
    end
  end
end


return M
