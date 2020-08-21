" Initialize the channel
if !exists('s:sniprunJobId')
  let s:sniprunJobId = 0
endif


let s:SnipRun = 'run'
let s:SnipTerminate = 'terminate'

let s:scriptdir = resolve(expand('<sfile>:p:h') . '/..')
let s:bin= s:scriptdir.'/target/release/sniprun'

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
  command! Terminate :call s:terminate()
endfunction


function! s:run() range
  let s:fl=a:firstline
  let s:ll=a:lastline
  call rpcnotify(s:sniprunJobId, s:SnipRun, str2nr(s:fl), str2nr(s:ll))
endfunction

function! s:terminate()
  call jobstop(s:sniprunJobId)
  let s:sniprunJobId = 0
  call s:connect()
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


