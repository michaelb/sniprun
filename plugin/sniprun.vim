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

" Entry point. Initialize RPC. If it succeeds, then attach commands to the `rpcnotify` invocations.
function! s:connect()
  let id = s:initRpc()
  if 0 == id
    echoerr "sniprun: cannot start rpc process"
  elseif -1 == id
    echoerr "sniprun: rpc process is not executable"
  else
    " Mutate our jobId variable to hold the channel ID
    let s:sniprunJobId = id

    call s:configureCommands()
  endif
endfunction




function! s:configureCommands()
  command! -range SnipRun <line1>,<line2>call s:run()
  command! SnipTerminate :call s:terminate()
  command! SnipReset :call s:clean()| :call s:terminate()
  command! SnipInfo :call s:showinfo()
  command! SnipReplMemoryClean :call s:clearReplMemory()

  " <Plug> mappings
  " command! -range SnipRun <line1>,<line2>call s:run()
  nnoremap <silent> <Plug>SnipRun :call <SID>run()<CR>
  vnoremap <silent> <Plug>SnipRun :'<'>call <SID>run()<CR>
  nnoremap <silent> <Plug>SnipTerminate :call <SID>terminate()<CR>
  nnoremap <silent> <Plug>SnipReset call <SID>clean()
  nnoremap <silent> <Plug>SnipInfo :call <SID>showinfo()<CR>
  nnoremap <silent> <Plug>SnipReplMemoryClean :call <SID>clearReplMemory()<CR>


endfunction


function! s:showinfo()
  silent execute '!sh'  s:scriptdir.'/ressources/infoscript.sh' s:scriptdir.'/src/interpreters' '>' s:scriptdir.'/ressources/infofile.txt'
  let infofile = s:scriptdir."/ressources/infofile.txt"
  let lines = readfile(infofile)
  for line in lines
    echomsg line
  endfor
endfunction

function! s:run() range
  let s:fl=a:firstline
  let s:ll=a:lastline
  call rpcnotify(s:sniprunJobId, s:SnipRun, str2nr(s:fl), str2nr(s:ll), s:scriptdir, s:SnipRun_select_interpreters, s:SnipRun_repl_behavior_enable, s:SnipRun_repl_behavior_disable)
endfunction

function! s:terminate()
  call jobstop(s:sniprunJobId)
  let s:sniprunJobId = 0
  call s:connect()
endfunction


function! s:clean()
  call rpcnotify(s:sniprunJobId, s:SnipClean)
  sleep 500m
  " necessary to give enough time to clean the sniprun work directory
  " in case diskis slow or something, will do something better later
  call terminate()
endfunction


function! s:clearReplMemory()
  call rpcnotify(s:sniprunJobId, s:SnipReplMemoryClean)
endfunction

" Initialize RPC
function! s:initRpc()
  if s:sniprunJobId == 0
    let jobid = jobstart([s:bin], { 'rpc': v:true })
    return jobid
  else
    return s:sniprunJobId
  endif
endfunction

call s:connect()


silent! call repeat#set("\<Plug>SnipRun", v:count)
