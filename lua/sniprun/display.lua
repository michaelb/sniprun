local M = {}
print("aaa34")
M.fw_handle = 0

function M.fw_open(row, column, message, ok)
  hl_ok = "SniprunVirtualTextOk"
  hl_err = "SniprunVirtualTextErr"
  if ok then
    hl = hl_ok
  else
    hl = hl_err
  end

  namespace_id = vim.api.nvim_create_namespace("")

  buf = 0 -- buffer 
  w = 0
  h = -1
  bp = {row, column}
  message_map = {}
  bufnr = vim.api.nvim_create_buf(false, true)
  for line in message:gmatch("([^\n]*)\n?") do
    table.insert(message_map, line)
    h = h + 1
    w = math.max(w,string.len(line)) 
    vim.api.nvim_buf_set_lines(bufnr,h,h+1,false,{""})
    vim.api.nvim_buf_set_virtual_text(bufnr, namespace_id, h, {{line, hl}}, {})
  end
  M.fw_handle = vim.api.nvim_open_win(bufnr, false, {relative='win', width=w+1, height=h, bufpos=bp, focusable=false, style='minimal', border='single'})
end



function M.fw_close()
  if M.fw_handle == 0 then return end
  vim.api.nvim_win_hide(M.fw_handle)
  M.fw_handle = 0
end



return M
