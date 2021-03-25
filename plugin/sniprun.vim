" Initialize the channel
if !exists('s:sniprunJobId')
  let s:sniprunJobId = 0
endif



function! s:configure_commands()
  command! -range SnipRun <line1>,<line2>call s:run()
  command! SnipTerminate lua require"sniprun".terminate()
  command! SnipReset lua require"sniprun".clean()
  command! SnipInfo :call s:showinfo()
  command! SnipReplMemoryClean :lua require"sniprun".clean_repl()

  " <Plug> mappings
  nnoremap <silent> <Plug>SnipRun :call <SID>run()<CR>
  vnoremap <silent> <Plug>SnipRun :'<'>call <SID>run()<CR>
  nnoremap <silent> <Plug>SnipTerminate :call <SID>terminate()<CR>
  nnoremap <silent> <Plug>SnipReset call <SID>clean()
  nnoremap <silent> <Plug>SnipInfo :call <SID>showinfo()<CR>
  nnoremap <silent> <Plug>SnipReplMemoryClean :call <SID>clearReplMemory()<CR>
endfunction

function s:run() range
  let s:fl = a:firstline
  let s:ll = a:lastline
  " lua print(vim.api.nvim_get_mode().mode)
  echo mode()
endfunction


function! s:showinfo()
  let s:scriptdir = resolve(expand('<sfile>:p:h') . '/..')
  silent execute '!sh'  s:scriptdir.'/ressources/infoscript.sh' s:scriptdir.'/src/interpreters' '>' s:scriptdir.'/ressources/infofile.txt'
  let infofile = s:scriptdir."/ressources/infofile.txt"
  let lines = readfile(infofile)
  for line in lines
    echomsg line
  endfor
endfunction

call s:configure_commands()

silent! call repeat#set("\<Plug>SnipRun", v:count)
