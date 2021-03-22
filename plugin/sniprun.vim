" Initialize the channel
if !exists('s:sniprunJobId')
  let s:sniprunJobId = 0
endif



" items sent through RPC to the rust program
let s:SnipRun = 'run'
let s:SnipTerminate = 'terminate'
let s:SnipClean = "clean"
let s:SnipInfo = "showinfo"
let s:SnipReplMemoryClean = "clearrepl"

let s:scriptdir = resolve(expand('<sfile>:p:h') . '/..')
let s:bin= s:scriptdir.'/target/release/sniprun'


let s:SnipRun_select_interpreters = get(g: ,'SnipRun_select_interpreters', [])
let s:SnipRun_repl_behavior_enable = get(g: ,'SnipRun_repl_behavior_enable', [])
let s:SnipRun_repl_behavior_disable = get(g: ,'SnipRun_repl_behavior_disable', [])
let s:SnipRun_inline_messages = get(g: ,'SnipRun_inline_messages', 0)





function! s:configure_commands()
  command! -range SnipRun <line1>,<line2>run()
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
  echo "lol"
endfunction


function! s:showinfo()
  silent execute '!sh'  s:scriptdir.'/ressources/infoscript.sh' s:scriptdir.'/src/interpreters' '>' s:scriptdir.'/ressources/infofile.txt'
  let infofile = s:scriptdir."/ressources/infofile.txt"
  let lines = readfile(infofile)
  for line in lines
    echomsg line
  endfor
endfunction

call s:configure_commands()

silent! call repeat#set("\<Plug>SnipRun", v:count)
