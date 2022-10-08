local M = {}

function deepcopy(orig)
    local orig_type = type(orig)
    local copy
    if orig_type == 'table' then
        copy = {}
        for orig_key, orig_value in next, orig, nil do
            copy[deepcopy(orig_key)] = deepcopy(orig_value)
        end
        setmetatable(copy, deepcopy(getmetatable(orig)))
    else -- number, string, boolean, etc
        copy = orig
    end
    return copy

end

function M.run()
    local sa = require('sniprun.api')
    local line = vim.api.nvim_win_get_cursor(0)[1]
    local ft = vim.bo.filetype
    local opts = deepcopy(require('sniprun').config_values)
    opts.display  = opts.live_display
    opts.show_no_output = {}
    sa.run_range(line,line, ft, opts)
end

function M.enable()
  vim.cmd [[
    augroup _sniprunlive
     autocmd!
     autocmd TextChanged * lua require'sniprun.live_mode'.run()
     autocmd TextChangedI * lua require'sniprun.live_mode'.run()
    augroup end
    lua require'sniprun.live_mode'.run()
  ]]
  vim.notify "Enabled Sniprun live mode"
end

function M.disable()
  M.remove_augroup "_sniprunlive"
  require('sniprun.display').clear_virtual_text()
  vim.notify "Disabled Sniprun live mode"
end

function M.toggle()
  if vim.fn.exists "#_sniprunlive#TextChanged" == 0 then
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
