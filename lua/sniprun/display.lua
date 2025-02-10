local M = {}
M.term = {}
M.fw_handle = 0
M.term.buffer = -1
M.term.window_handle = 0
M.term.current_line = -1
M.term.chan = -1
M.borders = "single"

local NAMESPACE = "sniprun"

function M.fw_open(row, column, message, ok, temp)
    M.fw_close()

    local hl_ok = "SniprunFloatingWinOk"
    local hl_err = "SniprunFloatingWinErr"
    local hl = ok and hl_ok or hl_err

    local namespace_id = vim.api.nvim_create_namespace(NAMESPACE)

    local w = 0
    local h = -1
    local bp = { row, column }
    local bufnr = vim.api.nvim_create_buf(false, true)
    for line in message:gmatch("([^\n]*)\n?") do
        h = h + 1
        w = math.max(w, string.len(line))
        vim.api.nvim_buf_set_lines(bufnr, h, h + 1, false, { line })
        vim.api.nvim_buf_add_highlight(bufnr, namespace_id, hl, h, 0, -1) -- highlight lines in floating window
    end
    if h ~= 0 then
        M.fw_handle = vim.api.nvim_open_win(bufnr, false, {
            relative = "win",
            width = w + 1,
            height = h,
            bufpos = bp,
            focusable = false,
            style = "minimal",
            border = M.borders,
        })
    end
end

function M.term_set_window_handle()
    local winid = vim.fn.bufwinid(M.term.buffer)
    if winid ~= -1 then
        return
    end

    local location = require("sniprun").config_values.display_options.terminal_position
    if location == "horizontal" then
        local height = require("sniprun").config_values.display_options.terminal_height or 20
        vim.cmd(":rightb " .. height .. "split")
    else
        local width = require("sniprun").config_values.display_options.terminal_width or 45
        vim.cmd(":rightb " .. width .. "vsplit")
    end
    M.term.window_handle = vim.api.nvim_get_current_win()

    -- return to doc buffer
    vim.cmd("wincmd p")
end

function M.term_set_buffer_chan(winid)
    if M.term.buffer ~= -1 then
        vim.api.nvim_win_set_buf(winid, M.term.buffer)
        return
    end

    local buf = vim.api.nvim_create_buf(false, true)

    vim.api.nvim_win_set_buf(winid, buf)
    local display_options = require("sniprun").config_values.display_options
    vim.fn.win_execute(winid, "setlocal scrollback=" .. display_options.terminal_scrollback)

    local lnumber = display_options.terminal_line_number and "number" or "nonumber"
    vim.fn.win_execute(winid, "setlocal " .. lnumber)

    local scl = display_options.terminal_signcolumn and vim.o.signcolumn or "no"
    vim.fn.win_execute(winid, "setlocal signcolumn=" .. scl)

    M.term.buffer = buf
    M.term.chan = vim.api.nvim_open_term(buf, {})

    -- Set the cursor to the last line in the new buffer, this will make the terminal
    -- auto-scroll to the bottom.
    vim.fn.win_execute(winid, "normal! G<cr>")
end

function M.term_open()
    M.term_set_window_handle()
    M.term_set_buffer_chan(M.term.window_handle)
end

function M.write_to_term(message, ok)
    M.term_open()

    if ok then
        status = "OK"
    else
        status = "ERROR"
    end

    -- Get the window information, including width and number of columns
    -- occupied by foldcolumn, signcolumn, and line number columns.
    local wininfo = vim.fn.getwininfo(M.term.window_handle)[1]

    -- The number of dashes to display before and after the message is half of
    -- the window screen minus the length of the message and the two spaces on
    -- each side.
    local numdashes = (wininfo.width - wininfo.textoff - status:len() - 4) / 2
    -- If the status message is not even, then 'numdashes' will be a float, and
    -- the header has wrong number of dashes. So we round it down as the prefix,
    -- and up for the suffix.

    local header_prefix = string.rep("-", math.floor(numdashes))
    local header_suffix = string.rep("-", math.ceil(numdashes))

    -- It's valid for the message to contain null characters per the neovim
    -- specification, so we try to avoid performing string operations on it by
    -- calling nvim_chan_send miltiple times.
    vim.api.nvim_chan_send(M.term.chan, "  " .. header_prefix .. status .. header_suffix .. "\n")
    vim.api.nvim_chan_send(M.term.chan, message)
    vim.api.nvim_chan_send(M.term.chan, "\n")
end

function M.close_all()
    M.fw_close()
    M.clear_virtual_text()
    M.term_close()

    M.close_api()
end

function M.fw_close()
    if M.fw_handle == 0 then
        return
    end
    vim.api.nvim_win_close(M.fw_handle, true)
    M.fw_handle = 0
end

function M.clear_virtual_text()
    vim.cmd(
        "let sniprun_namespace_id = nvim_create_namespace('sniprun')\n call nvim_buf_clear_namespace(0,sniprun_namespace_id, 0 ,-1)"
    )
end

function M.term_close()
    if M.term.window_handle == 0 then
        return
    end
    vim.api.nvim_win_close(M.term.window_handle, true)
    M.term.window_handle = 0
    M.term.buffer = -1
    M.term.current_line = 0
    M.term.chan = -1
end

function M.display_nvim_notify(message, ok)
    -- ok is a boolean variable for the status (true= ok, false= error)
    --
    -- test if nvim_notify is availablea
    if pcall(function()
        require("notify")
    end) then
        --ok
    else
        print("Sniprun: nvim_notify is not installed")
        return
    end

    if message == "" then
        return
    end
    local title = ok and "Sniprun: Ok" or "Sniprun: Error"
    local notif_style = ok and "info" or "error"
    require("notify")(message, notif_style, {
        title = title,
        timeout = require("sniprun").config_values.display_options.notification_timeout * 1000,
        render = require("sniprun").config_values.display_options.notification_render or "default",
    })
end

function M.display_virt_text(ns, line, message, highlight)
    vim.api.nvim_buf_set_extmark(0, ns, line, -1, { virt_text = { { message, highlight } } })
end

function M.display_virt_line(ns, line_pos, message, highlight)
    local virt_lines = {}
    for line in message:gmatch("([^\n]*)\n?") do
        table.insert(virt_lines, { { line, highlight } })
    end
    vim.api.nvim_buf_set_extmark(0, ns, line_pos, 0, {
        virt_lines = virt_lines,
    })
end

function M.send_api(message, ok)
    local d = {}
    d.message = message
    if ok then
        d.status = "ok"
    else
        d.status = "error"
    end
    local listeners = require("sniprun.api").listeners

    if type(next(listeners)) == "nil" then
        print("Sniprun: No listener registered")
    end

    for i, f in ipairs(listeners) do
        f(d)
    end
end

function M.close_api()
    local listeners = require("sniprun.api").closers
    for i, f in ipairs(listeners) do
        f()
    end
end

return M
