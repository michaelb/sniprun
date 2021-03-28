
function! s:configure_commands()
  command! SnipRun :lua require'sniprun'.run()
  command! SnipTerminate :lua require'sniprun'.terminate()
  command! SnipReset :lua require'sniprun'.reset()
  command! SnipInfo :call s:showinfo()
  command! SnipReplMemoryClean :lua require'sniprun'.clear_repl()

  " " <Plug> mappings
  " nnoremap <silent> <Plug>SnipRun :lua require'sniprun'.run()
  " vnoremap <silent> <Plug>SnipRun lua require'sniprun'.run('v')
  " nnoremap <silent> <Plug>SnipTerminate lua require'sniprun'.terminate()
  " nnoremap <silent> <Plug>SnipReset lua require'sniprun'.clean()
  nnoremap <silent> <Plug>SnipInfo :call <SID>showinfo()<CR>
  " nnoremap <silent> <Plug>SnipReplMemoryClean lua require'sniprun'.clean_repl()
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
