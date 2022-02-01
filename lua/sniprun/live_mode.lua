local M = {}

vim.cmd [[
  function Test()
    lua require("sniprun.live_mode").run()
  endfunction
  function TestI()
    lua require("sniprun.live_mode").run()
  endfunction
]]

function M.run()
    local sa = require('sniprun.api')
    local line = vim.api.nvim_win_get_cursor(0)[1]
    local ft = vim.bo.filetype
    local opts = require('sniprun').config_values
    opts.display  = { "VirtualTextOk"}
    opts.show_no_output = {}
    sa.run_range(line,line, ft, opts)
end

function M.enable()
  vim.cmd [[
    augroup _sniprun
     autocmd!
     autocmd TextChanged * call Test()
     autocmd TextChangedI * call TestI()
    augroup end
    call Test()
  ]]
  vim.notify "Enabled Sniprun live mode"
end

function M.disable()
  M.remove_augroup "_sniprun"
  vim.cmd [[
    SnipClose
    SnipTerminate
    ]]
  vim.notify "Disabled Sniprun live mode"
end

function M.toggle()
  if vim.fn.exists "#_sniprun#TextChanged" == 0 then
    M.enable()
  else
    M.disable()
  end
end

function M.remove_augroup(name)
  if vim.fn.exists("#" .. name) == 1 then
    vim.cmd("au! " .. name)
  end
end

vim.cmd [[ command! SnipLive execute 'lua require("sniprun.live_mode").toggle()' ]]
vim.api.nvim_set_keymap("n", "<Plug>SnipLive", ":lua require'sniprun.live_mode'.toggle()<CR>",{silent=true})

return M
