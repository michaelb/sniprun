local M = {}

-- function M.floating_win_ask(message)
--
--     local w = 0
--     local h = -1
--     local bp = vim.api.nvim_win_get_cursor(0) -- message at current cursor position
--     local bufnr = vim.api.nvim_create_buf(false, true)
--     for line in message:gmatch("([^\n]*)\n?") do
--         h = h + 1
--         w = math.max(w, string.len(line))
--         vim.api.nvim_buf_set_lines(bufnr, h, h + 1, false, { line })
--     end
--     if h ~= 0 then
--         M.fw_handle = vim.api.nvim_open_win(bufnr, false,
--             {
--                 relative = 'win',
--                 width = w + 1,
--                 height = h,
--                 bufpos = bp,
--                 focusable = false,
--                 style = 'minimal',
--                 border = "single"
--             })
--         vim.api.nvim_win_call(M.fw_handle, function()
--             vim.api.nvim_exec_autocmds("BufWinEnter", { buffer = bufnr, modeline = false })
--         end)
--         vim.api.nvim_set_current_win(M.fw_handle)
--     end
-- end

function M.vim_input(message)
    print(vim.fn.input(message))
end

return M
