local M = {}
M.ping_anwsered=0
M.custom_highlight=false
M.info_floatwin = {}

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

  interpreter_options = {},

  display = {
    "Classic",
    "VirtualTextOk",
    -- "VirtualTextErr",
    -- "LongTempFloatingWindow",
    -- "TempFloatingWindow",
    -- "Terminal",
    -- "TerminalWithCode",
    -- "Api",
    -- "NvimNotify"
  },

  live_display = { "VirtualTextOk" }, -- displayed only for live mode

  display_options = {
    terminal_scrollback = vim.o.scrollback, -- change terminal display scrollback lines
    terminal_line_number = false, -- whether show line number in terminal window
    terminal_signcolumn = false,  -- whether show signcolumn in terminal window
    terminal_width = 45,          -- change the terminal display option width
    terminal_persistence = true,  -- always keep the terminal open (true) or close it at every occasion (false)
    notification_timeout = 5      -- timeout for nvim_notify output
  },

  show_no_output = {
    "Classic",
    "TempFloatingWindow", -- implies LongTempFloatingWindow, which is not a correct key here
  },

  inline_messages = 0,
  borders = 'single',

  -- default highlight stuff goes here
  snipruncolors = {
    SniprunVirtualTextOk   =  {bg="#66eeff",fg="#000000",ctermbg="Cyan",ctermfg="Black"},
    SniprunFloatingWinOk   =  {fg="#66eeff",ctermfg="Cyan"},
    SniprunVirtualTextErr  =  {bg="#881515",fg="#000000",ctermbg="DarkRed",ctermfg="Black"},
    SniprunFloatingWinErr  =  {fg="#881515",ctermfg="DarkRed"},
  },

  -- whether the user can toggle the live_mode. It's kept as an option so it's not activated by chance
  -- by an user that would be unaware of the potentially dangerous behavior
  live_mode_toggle='off',

  -- auto-filled with the real nvim's PID, sniprun's bin and source locations
  neovim_pid=0,
  binary_path=binary_path,
  sniprun_path=sniprun_path,

}


M.config_up=0


function M.initial_setup()
  if M.config_up == 1 then return end
  M.setup()
  M.config_up = 0
end



function M.setup(opts)
  opts = opts or {}

  -- pre-process config keys
  for key,value in pairs(opts) do
    if M.config_values[key] == nil then
      error(string.format('[Sniprun] Key %s does not exist in config values',key))
      return
    end
    if key == 'snipruncolors' then
      M.custom_highlight = true
    end
    if key == 'live_mode_toggle' and opts[key] == 'enable' then
      require('sniprun.live_mode')
    end
  end

  -- merge user config into default config values
  M.config_values = vim.tbl_deep_extend("force", M.config_values, opts)

  M.configure_keymaps()
  M.setup_highlights()
  M.setup_autocommands()
  M.setup_display()

  M.config_values.neovim_pid = vim.fn.getpid()

  M.config_up = 1
end


local highlight = function(group, styles)
  local gui = styles.gui and 'gui='..styles.gui or 'gui=NONE'
  local sp = styles.sp and 'guisp='..styles.sp or 'guisp=NONE'
  local fg = styles.fg and 'guifg='..styles.fg or 'guifg=NONE'
  local bg = styles.bg and 'guibg='..styles.bg or 'guibg=NONE'
  local ctermbg = styles.ctermbg and 'ctermbg='..styles.ctermbg or 'ctermbg=NONE'
  local ctermfg = styles.ctermfg and 'ctermfg='..styles.ctermfg or 'ctermfg=NONE'
  -- This somehow works for default highlighting. with or even without cterm colors
  -- hacky way tho.Still I think better than !hlexists
  vim.cmd('highlight '..group..' '..gui..' '..sp..' '..fg..' '..bg..' '..ctermbg..' '..ctermfg)
  vim.api.nvim_command('autocmd ColorScheme * highlight '..group..' '..gui..' '..sp..' '..fg..' '..bg..' '..ctermbg..' '..ctermfg)
end


function M.setup_display()
    local D = require'sniprun.display'
    D.borders = M.config_values.borders
end


function M.setup_highlights()
  local colors_table = M.config_values["snipruncolors"]
  if M.custom_highlight then
    vim.cmd('augroup snip_highlights')
    vim.cmd('autocmd!')
    for group, styles in pairs(colors_table) do
      -- print('setting up for '..group,'with style :','bg :',styles.bg,'fg :',styles.fg)
      highlight(group, styles)
    end
    vim.cmd('augroup END')
  else 
    for group, styles in pairs(colors_table) do
      local gui = styles.gui and 'gui='..styles.gui or 'gui=NONE'
      local sp = styles.sp and 'guisp='..styles.sp or 'guisp=NONE'
      local fg = styles.fg and 'guifg='..styles.fg or 'guifg=NONE'
      local bg = styles.bg and 'guibg='..styles.bg or 'guibg=NONE'
      local ctermbg = styles.ctermbg and 'ctermbg='..styles.ctermbg or 'ctermbg=NONE'
      local ctermfg = styles.ctermfg and 'ctermfg='..styles.ctermfg or 'ctermfg=NONE'

      vim.cmd("if !hlexists('"..group.."') \n hi "..group.." "..gui.." "..sp.." "..fg.." "..bg.." "..ctermbg.." "..ctermfg)
    end
  end
end

function M.setup_autocommands()
  vim.cmd("function! Sniprun_fw_close_wrapper()\n lua require'sniprun.display'.fw_close()\n endfunction")

  vim.cmd("augroup sniprun_fw_close")
  vim.cmd("autocmd!")
  vim.cmd("autocmd CursorMoved,BufWinLeave ?* call Sniprun_fw_close_wrapper()")
  vim.cmd("augroup END")

  vim.cmd("function! Sniprun_clear_vt_on_leave()\n lua require'sniprun.display'.clear_virtual_text()\n endfunction")
  vim.cmd("augroup sniprun_clear_vt")
  vim.cmd("autocmd!")
  vim.cmd("autocmd BufWinLeave ?* call Sniprun_clear_vt_on_leave()")
  vim.cmd("augroup END")

  vim.cmd("function! Sniprun_close_term_on_leave()\n lua require'sniprun.display'.term_close()\n endfunction")
  vim.cmd("augroup sniprun_close_term")
  vim.cmd("autocmd!")
  vim.cmd("autocmd VimLeave,QuitPre ?* call Sniprun_close_term_on_leave()")
  vim.cmd("augroup END")

  vim.cmd("function! Sniprun_close_term_on_bufleave()\n lua require'sniprun.display'.term_autoclose()\n endfunction")
  vim.cmd("augroup sniprun_close_term")
  vim.cmd("autocmd!")
  vim.cmd("autocmd BufWinLeave ?* call Sniprun_close_term_on_bufleave()")
  vim.cmd("augroup END")
end


function M.configure_keymaps()

  vim.api.nvim_set_keymap("v", "<Plug>SnipRun", ":lua require'sniprun'.run('v')<CR>", {silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipRun", ":lua require'sniprun'.run()<CR>",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipRunOperator", ":set opfunc=SnipRunOperator<CR>g@",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipReset", ":lua require'sniprun'.reset()<CR>", {silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipInfo", ":lua require'sniprun'.info()<CR>",{})
  vim.api.nvim_set_keymap("n", "<Plug>SnipReplMemoryClean", ":lua require'sniprun'.clear_repl()<CR>",{silent=true})
  vim.api.nvim_set_keymap("n", "<Plug>SnipClose", ":lua require'sniprun.display'.close_all()<CR>",{silent=true})

  vim.cmd("command! SnipReset :lua require'sniprun'.reset()")
  vim.cmd("command! SnipReplMemoryClean :lua require'sniprun'.clear_repl()")
  vim.cmd("function! SnipRunOperator(...) \n lua require'sniprun'.run('n') \n endfunction")
  vim.cmd("command! SnipClose :lua require'sniprun.display'.close_all()")

  vim.cmd("function! ListInterpreters(A,L,P) \n let l = split(globpath('".. M.config_values.sniprun_path .."/docs/sources/interpreters', '*.md'),'\\n') \n let rl = [] \n for e in l \n let rl += [split(e,'/')[-1][:-4]] \n endfor \n return rl \n endfunction")
  vim.cmd("command! -nargs=* -complete=customlist,ListInterpreters SnipInfo :lua require'sniprun'.info(<q-args>)")

  vim.cmd("function! SnipRunLauncher(...) range \nif a:firstline == a:lastline \n lua require'sniprun'.run() \n elseif a:firstline == 1 && a:lastline == line(\"$\")\nlet g:sniprun_cli_args_list = a:000\n let g:sniprun_cli_args = join(g:sniprun_cli_args_list,\" \") \n lua require'sniprun'.run('w') \n else \n lua require'sniprun'.run('v') \n endif \n endfunction")
  vim.cmd("command! -range -nargs=? SnipRun <line1>,<line2>call SnipRunLauncher(<q-args>)")


end

function M.start()
  if M.job_id ~= nil then return end
  M.job_id = vim.fn.jobstart({ M.config_values.binary_path }, { rpc = true })
end

function M.notify(method, ...)
  M.start()
  local status, err = pcall(vim.rpcnotify, M.job_id, method, ...)
  if not status then
    M.terminate()
    M.start()
    vim.rpcnotify(M.job_id, method, ...)
  end
end

function M.run(mode)
  local range_begin, range_end = M.get_range(mode)
  M.config_values["sniprun_root_dir"] = M.config_values.sniprun_path
  M.notify('run', range_begin, range_end, M.config_values, vim.g.sniprun_cli_args or "" )
end


function M.get_range(mode)
  local line1, line2
  if not mode then
    line1 = vim.api.nvim_win_get_cursor(0)[1]
    line2 = line1
  elseif mode:match("[w]") then
    line1 = 1
    line2 = vim.fn.eval("line('$')")
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

function M.ping()
  M.notify("ping")
end

function M.terminate()
  vim.fn.jobstop(M.job_id)
  M.job_id = nil
end


-- get all lines from a file, returns an empty
-- list/table if the file does not exist
local function lines_from(filename)
 local file = io.open(filename, "r")
 local arr = {}
 for line in file:lines() do
    table.insert (arr, line)
 end
    return arr
end

function M.display_lines_in_floating_win(lines)
    -- Create window.
    local width = math.ceil(vim.o.columns * 0.8)
    local height = math.ceil(vim.o.lines * 0.9)
    M.info_floatwin.buf = vim.api.nvim_create_buf(false, true)


    M.info_floatwin.win = vim.api.nvim_open_win(M.info_floatwin.buf, true, {
	relative = 'editor',
	style = 'minimal',
	width = width,
	height = height,
	col = math.ceil((vim.o.columns - width) / 2),
	row = math.ceil((vim.o.lines - height) / 2 - 1),
	border = 'single'
    })
    -- vim.api.nvim_win_set_option(M.info_floatwin.win, 'winhighlight', 'Normal:CursorLine')

    -- local namespace_id = vim.api.nvim_create_namespace("sniprun_info")
	vim.api.nvim_buf_set_lines(M.info_floatwin.buf,0,500,false, lines)
	-- vim.api.nvim_buf_add_highlight(M.info_floatwin.buf, namespace_id, hl, h,0,-1) -- highlight lines in floating window
end

function dir_exists(path)
   local ok, err, code = os.rename(path, path)
   if not ok then
      if code == 13 then
         -- Permission denied, but it exists
         return true
      end
   end
   return ok, err
end

function M.info(arg)
  if arg == nil or arg == "" then
    M.config_values["sniprun_root_dir"] = M.config_values.sniprun_path
    M.notify("info",1,1,M.config_values, "")

    vim.wait(500) -- let enough time for the sniprun binary to generate the file
    print(" ")
    -- default cache dir is different on Linux and MacOS
    local default_cache_dir = os.getenv("HOME").."/.cache"
    if dir_exists(os.getenv("HOME").."/Library/Caches") then -- we're (probably) on MacOS
        default_cache_dir = os.getenv("HOME").."/Library/Caches"
    end

    local cache_dir = os.getenv("XDG_CACHE_HOME") or default_cache_dir
    local sniprun_cache_dir = cache_dir.."/sniprun"
    local lines = lines_from(sniprun_cache_dir.."/infofile.txt")
    -- print all lines content
    M.display_lines_in_floating_win(lines)
    else --help about a particular interpreter
      local lines = lines_from(M.config_values.sniprun_path.."/docs/sources/interpreters/"..string.gsub(arg,"%s+","")..".md")
      M.display_lines_in_floating_win(lines)
  end
end

function M.health()
  local health_start = vim.fn["health#report_start"]
  local health_ok = vim.fn['health#report_ok']
  local health_error = vim.fn['health#report_error']
  local health_warn = vim.fn['health#report_warn']
  health_start('Installation')

  if vim.fn.executable('cargo') == 0 then health_warn("Rust toolchain not available", {"[optionnal] Install the rust toolchain https://www.rust-lang.org/tools/install"})
  else health_ok("Rust toolchain found") end

  if vim.fn.executable(M.config_values.binary_path) == 0 then health_error("sniprun binary not found!")
  else health_ok("sniprun binary found at ".. M.config_values.binary_path) end

  local terminate_after = M.job_id == nil
  local path_log_file = os.getenv('HOME').."/.cache/sniprun/sniprun.log"
  local path_log_file_mac = os.getenv('HOME').."/Library/Caches/sniprun/sniprun.log"
  os.remove(path_log_file)

  -- check if the log is recreated
  if pcall(M.ping) then health_ok("Sent a ping to the sniprun binary")
  else health_warn("Could not send a ping to the sniprun binary - is it present, executable and compatible with your CPU architecture?") end
    

  os.execute("sleep 0.2")
  if not M.file_exists(path_log_file) and not M.file_exists(path_log_file_mac)  then health_error("sniprun binary incompatible or crash at start", {"Compile sniprun locally, with a clean reinstall and 'bash ./install.sh 1' as post-install command."})
  else health_ok("sniprun binary runs correctly")
  end
end

function M.file_exists(name)
   local f=io.open(name,"r")
   if f~=nil then io.close(f) return true else return false end
end



return M

