let s:scriptdir = expand('<sfile>:p:h')."/../.." "root of the project
let s:bin= s:scriptdir.'/target/release/sniprun'

function health#sniprun#check()
  lua require'sniprun'.health()
endfunction
