local M={}
M.term={}
M.fw_handle=0
M.term.opened = 0
M.term.buffer = -1
M.term.window_handle = 0
M.term.current_line = -1
M.term.chan = -1
M.borders = 'single'

local NAMESPACE = 'sniprun'

function M.fw_open(row, column, message, ok, temp)
  M.fw_close()

  local hl_ok = "SniprunFloatingWinOk"
  local hl_err = "SniprunFloatingWinErr"
  local hl = ok and hl_ok or hl_err

  local namespace_id = vim.api.nvim_create_namespace(NAMESPACE)

  local w = 0
  local h = -1
  local bp = {row , column}
  local message_map = {}
  local bufnr = vim.api.nvim_create_buf(false, true)
  for line in message:gmatch("([^\n]*)\n?") do
    h = h + 1
    w = math.max(w,string.len(line))
    vim.api.nvim_buf_set_lines(bufnr,h,h+1,false,{line})
    vim.api.nvim_buf_add_highlight(bufnr, namespace_id, hl, h,0,-1) -- highlight lines in floating window
  end
  M.fw_handle = vim.api.nvim_open_win(bufnr, false, {relative='win', width=w+1, height=h, bufpos=bp, focusable=false, style='minimal',border=M.borders})
end

function M.term_open()
  if M.term.opened ~= 0 then return end
  local open_term_cmd = ':rightb'.. require('sniprun').config_values.display_options.terminal_width .. 'vsplit'
  vim.cmd(open_term_cmd)
  local buf = vim.api.nvim_create_buf(false,true)
  local win = vim.api.nvim_get_current_win()
  vim.api.nvim_win_set_buf(win,buf)
  local chan = vim.api.nvim_open_term(buf, {})
  vim.cmd("set scrollback=1")
  vim.cmd('setlocal nonu')
  vim.cmd('setlocal signcolumn=no')

  vim.cmd("wincmd p")
  M.term.opened = 1
  M.term.window_handle = win
  M.term.buffer = buf
  M.term.chan = chan
end

function M.write_to_term(message, ok)
  M.term_open()

  local h = M.term.current_line or -1

  local status = "------"
  if ok then
    status = "--OK--"
  else
    status = "ERROR-"
  end
  
  local width = vim.api.nvim_win_get_width(M.term.window_handle)  
  local half_width = (width - 6 - 4 - 4) / 2
  message = "  "..string.rep("-",half_width)..status..string.rep("-", half_width).."  ".."\n"..message

  for line in message:gmatch("([^\n]*)\n?") do
    h = h +1
    vim.api.nvim_chan_send(M.term.chan, line)
    vim.api.nvim_chan_send(M.term.chan, "\n\r");
  end
  vim.api.nvim_chan_send(M.term.chan, "\n\r");
  M.term.current_line = h

end


function M.close_all()
  M.fw_close()
  M.clear_virtual_text()
  M.term_close()

  M.close_api()
end


function M.fw_close()
  if M.fw_handle == 0 then return end
  vim.api.nvim_win_close(M.fw_handle, true)
  M.fw_handle = 0
end


function M.clear_virtual_text()
  vim.cmd("let sniprun_namespace_id = nvim_create_namespace('sniprun')\n call nvim_buf_clear_namespace(0,sniprun_namespace_id, 0 ,-1)")
end

function M.term_close()
  if M.term.window_handle == 0 then return end
  vim.api.nvim_win_close(M.term.window_handle, true)
  M.term.opened = 0
  M.term.window_handle = 0
  M.term.buffer = -1
  M.term.current_line = 0
  M.term.chan=-1
end


function M.display_nvim_notify(message, ok)
    -- ok is a boolean variable for the status (true= ok, false= error)
    --
    -- test if nvim_notify is availablea
    if pcall(function() require('notify') end) then
	--ok
    else
	print("Sniprun: nvim_notify is not installed")
	return
    end

    if message == "" then return end

    local title = ok and "Sniprun: Ok" or "Sniprun: Error"
    local notif_style = ok and "info" or "error"
    require("notify")(message, notif_style, {title=title, timeout=require('sniprun').config_values.display_options.notification_timeout})
end

function M.display_extmark(ns,line, message, highlight)
    vim.api.nvim_buf_set_extmark(0,ns,line,-1,{virt_text={{message,highlight}}})
end


function M.send_api(message, ok)
    local d = {}
    d.message = message
    if ok then
	d.status = "ok"
    else
	d.status = "error"
    end
    local listeners = require('sniprun.api').listeners
    
    if type(next(listeners)) == "nil" then
	print("Sniprun: No listener registered")
    end

    for i,f in ipairs(listeners) do
	f(d)
    end
end

function M.close_api()
    local listeners = require('sniprun.api').closers
    for i,f in ipairs(listeners) do
	f()
    end
end


return M
