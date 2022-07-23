-- debug only
-- local inspect = require('lib.inspect')

-- function on_delete() 
--     print('delete')
--     return ''
-- end

function on_print()
    print('5')
    return ''
end

function press(event)
	-- position = vim.api.nvim_win_get_position(0)
	-- if global pipe name is empty, editor-reader pipe-name <name>
	-- write to pipe file event and cursor	

    direction_text = ''
    if event == 'up' then
        direction_text = '<Up>'
    elseif event == 'down' then
        direction_text = '<Down>'
    elseif event == 'left' then
        direction_text = '<Left>'
    elseif event == 'right' then
        direction_text = '<Right>'
    end

    -- based on https://gitter.im/neovim/neovim?at=616755c62197144e84637618
    -- help by Sean Dewar

    

    direction = vim.api.nvim_replace_termcodes(direction_text, true, false, true)
    vim.api.nvim_input(direction_text) --, "n", true)
    -- TODO input 
    return 1
end

function send(event)
    -- vim.api.nvim_command('redraw!')
    -- vim.api.nvim_command('sleep 1000m')


    cursor = vim.api.nvim_win_get_cursor(0)
	buffer = vim.api.nvim_win_get_buf(0)
	name = vim.api.nvim_buf_get_name(buffer)

    -- based on https://stackoverflow.com/a/69051972/438099
    -- help by Anas sheshai https://stackoverflow.com/users/9500085/anas-sheshai
    path = vim.fn.expand(name)
    -- os.execute('/home/al/editor-reader/editor-reader/target/debug/editor-reader ' .. name)
    text_path = '/tmp/editor-reader' .. path
    file_text = io.open(text_path, 'a')
    file_text:write(event, " ")
    -- file1:write(name, " ")
    file_text:write(cursor[1], " ", cursor[2], "\n")
    file_text:close()

    pipe_path = '/tmp/editor-reader' .. path .. '.pipe'
	file1 = io.open(pipe_path, 'w')
    file1:write(event, " ")
    -- file1:write(name, " ")
    file1:write(cursor[1], " ", cursor[2], "\n")
    file1:close()

    -- write_event(..)
    -- print(event)
    return 1
end

-- print(0)

-- credit to https://github.com/nvim-lua/completion-nvim/issues/307#issuecomment-753524686
-- workaround a bit with ? : , not sure if there is a better way
vim.api.nvim_set_keymap('i', '5', 'v:lua.send("describe")', {expr = true, noremap = true})
-- vim.api.nvim_set_keymap('i', '<Up>', 'v:lua.press("up") ? v:lua.send("up") : ""', {expr = true, noremap = true})
-- vim.api.nvim_set_keymap('i', '<Down>', 'v:lua.press("down") ? v:lua.send("down") : ""', {expr = true, noremap = true})
-- v:lua.send("left")'
-- v:lua.send("right")'
vim.api.nvim_set_keymap('i', '<Left>', 'v:lua.send("left") ? "\\<Left>" : "\\<Left>"', {expr = true, noremap = true})
vim.api.nvim_set_keymap('i', '<Right>', 'v:lua.send("right") ? "\\<Right>" : "\\<Right>"', {expr = true, noremap = true})
-- vim.api.nvim_set_keymap('i', '<Tab>', 'v:lua.on_delete()', {expr = true, noremap = true})
-- _G['on_delete'] = on_delete
