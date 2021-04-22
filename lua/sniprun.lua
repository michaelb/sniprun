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

  display = {
    -- "Classic",
    "VirtualTextOk",
    -- "LongTempFloatingWindow",
    -- "TempFloatingWindow",
    -- "VirtualTextErr",
    "Terminal"
    },

  inline_messages = 0
}

M.config_up=0

function M.load_vimscript_config()
  vimscript_config = {}
  vimscript_config["repl_enable"] = vim.g.SnipRun_repl_behavior_enable or M.config_values["repl_enable"]
  vimscript_config["repl_disable"] = vim.g.SnipRun_repl_behavior_disable or M.config_values["repl_disable"]
  vimscript_config["selected_interpreters"] = vim.g.SnipRun_select_interpreters or M.config_values["selected_interpreters"]
  vimscript_config["inline_messages"] = vim.g.SnipRun_inline_messages or M.config_values["inline_messages"]

  return vimscript_config
end


function M.initial_setup()
  if M.config_up == 1 then return end
  M.setup()
  M.config_up = 0
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
  M.setup_highlights()
  M.setup_autocommands()

  M.config_up = 1
end

function M.setup_highlights()
  vim.cmd("if !hlexists('SniprunVirtualTextOk')  \n hi SniprunVirtualTextOk	ctermbg=Cyan guibg=#66eeff ctermfg=Black guifg=#000000 \nendif")
  vim.cmd("if !hlexists('SniprunVirtualTextErr') \n hi SniprunVirtualTextErr	ctermbg=DarkRed guibg=#881515 ctermfg=Black guifg=#000000 ")
  vim.cmd("if !hlexists('SniprunFloatingWinErr') \n hi SniprunFloatingWinErr	guifg=#881515 ctermfg=DarkRed")
  vim.cmd("if !hlexists('SniprunFloatingWinOk')  \n hi SniprunFloatingWinOk	ctermfg=Cyan guifg=#66eeff")
  vim.cmd("if !hlexists('SniprunTermOk')  \n hi SniprunTermOk	ctermfg=Cyan guifg=#66eeff")
  vim.cmd("if !hlexists('SniprunTermErr')  \n hi SniprunTermErr	guifg=#881515 ctermfg=DarkRed")

end

function M.setup_autocommands()
  vim.cmd("function! Sniprun_fw_close_wrapper()\n lua require'sniprun.display'.fw_close()\n endfunction")

  vim.cmd("augroup sniprun_fw_close")
  vim.cmd("autocmd!")
  vim.cmd("autocmd CursorMoved,BufWinLeave * call Sniprun_fw_close_wrapper()")
  vim.cmd("augroup END")

  vim.cmd("function! Sniprun_clear_vt_on_leave()\n lua require'sniprun.display'.clear_virtual_text()\n endfunction")
  vim.cmd("augroup sniprun_clear_vt")
  vim.cmd("autocmd!")
  vim.cmd("autocmd BufWinLeave * call Sniprun_clear_vt_on_leave()")
  vim.cmd("augroup END")

  vim.cmd("function! Sniprun_close_term_on_leave()\n lua require'sniprun.display'.term_close()\n endfunction")
  vim.cmd("augroup sniprun_close_term")
  vim.cmd("autocmd!")
  vim.cmd("autocmd VimLeave,QuitPre,BufWinLeave * call Sniprun_close_term_on_leave()")
  vim.cmd("augroup END")
end


function M.configure_keymaps()

  vim.api.nvim_set_keymap("v", "<Plug>SnipRun", ":lua require'sniprun'.run('v')<CR>", {silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipRun", ":lua require'sniprun'.run()<CR>",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipRunOperator", ":set opfunc=SnipRunOperator<CR>g@",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipRTerminate", ":lua require'sniprun'.terminate()<CR>",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipReset", ":lua require'sniprun'.reset()<CR>",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipInfo", ":lua require'sniprun'.info()<CR>",{})
  vim.api.nvim_set_keymap("n", "<Plug>SnipReplMemoryClean", ":lua require'sniprun'.clear_repl()<CR>",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipClose", ":lua require'sniprun.display'.close_all()<CR>",{silent=true})

  vim.cmd("command! SnipTerminate :lua require'sniprun'.terminate()")
  vim.cmd("command! SnipReset :lua require'sniprun'.reset()")
  vim.cmd("command! SnipReplMemoryClean :lua require'sniprun'.clear_repl()")
  vim.cmd("function! SnipRunOperator(...) \n lua require'sniprun'.run('n') \n endfunction")

  vim.cmd("function! ListInterpreters(A,L,P) \n let l = split(globpath('"..sniprun_path.."/doc/', '*.md'),'\\n') \n let rl = [] \n for e in l \n let rl += [split(e,'/')[-1][:-4]] \n endfor \n return rl \n endfunction")
  vim.cmd("command! -nargs=* -complete=customlist,ListInterpreters SnipInfo :lua require'sniprun'.info(<q-args>)")

  vim.cmd("function! SnipRunLauncher() range \n if a:firstline == a:lastline \n lua require'sniprun'.run() \n else \n lua require'sniprun'.run('v') \n endif \n endfunction")
  vim.cmd("command! -range SnipRun <line1>,<line2>call SnipRunLauncher()")

  vim.cmd("command! SnipClose :lua require'sniprun.display'.close_all()")

end

local function start()
  if M.job_id ~= nil then return end
  M.job_id = vim.fn.jobstart({ binary_path }, { rpc = true })
end

function M.notify(method, ...)
  start()
  local status, err = pcall(vim.rpcnotify, M.job_id, method, ...)
  if not status then
    M.terminate()
    start()
    vim.rpcnotify(M.job_id, method, ...)
  end
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
  elseif mode:match("[n]") then
    line1 = vim.api.nvim_buf_get_mark(0, '[')[1]
    line2 = vim.api.nvim_buf_get_mark(0, ']')[1]
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

function M.info(arg)
  if arg == nil or arg == "" then
    M.config_values["sniprun_root_dir"] = sniprun_path
    M.notify("info",1,1,M.config_values)

    local sniprun_path = vim.fn.fnamemodify( vim.api.nvim_get_runtime_file("lua/sniprun.lua", false)[1], ":p:h") .. "/.."

    if M.config_values.inline_messages ~= 0 then
      vim.wait(500) -- let enough time for the sniprun binary to generate the file
      print(" ")
      local lines = lines_from(sniprun_path.."/ressources/infofile.txt")
      -- print all lines content
      for k,v in pairs(lines) do
        print(v)
      end
    end
  else --help about a particular interpreter
      local lines = lines_from(sniprun_path.."/doc/"..string.gsub(arg,"%s+","")..".md")
    for k,v in pairs(lines) do
      print(v)
    end
  end
end

function M.health()
  local health_start = vim.fn["health#report_start"]
  local health_ok = vim.fn['health#report_ok']
  local health_error = vim.fn['health#report_error']
  local health_warn = vim.fn['health#report_warn']
  health_start('Installation')
  if vim.fn.executable('tree-sitter') == 0 then
    health_warn('`tree-sitter` executable not found (parser generator, only needed for :TSInstallFromGrammar,'..
                ' not required for :TSInstall)')
  else
    local handle = io.popen('tree-sitter  -V')
    local result = handle:read("*a")
    handle:close()
    local version = vim.split(result,'\n')[1]:match('[^tree%psitter].*')
    health_ok('`tree-sitter` found '..version..' (parser generator, only needed for :TSInstallFromGrammar)')
  end

  if vim.fn.executable(binary_path) == 0 then health_error("sniprun binary not found!")
  else health_ok("sniprun binary found") end


  if vim.fn.executable('cargo') == 0 then health_warn("Rust toolchain not available", {"[optionnal] Install the rust toolchain https://www.rust-lang.org/tools/install"})
  else health_ok("Rust toolchain found") end
end

return M
