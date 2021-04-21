local M = {}

M.fw_handle = 0

function M.fw_open()
  buf = 0 -- buffer to display
  w = 15
  h = 15
  bp = {5,5}
  M.fw_handle = vim.api.nvim_open_win(buf, false, {relative='win', width=w, height=h, bufpos={12,12},border='single'})
end


return M
