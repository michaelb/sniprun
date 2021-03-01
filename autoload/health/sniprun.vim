let s:scriptdir = expand('<sfile>:p:h')."/../.." "root of the project
let s:bin= s:scriptdir.'/target/release/sniprun'

function health#sniprun#check()


  if !empty(glob(s:bin)) 
    "binary is present
    call health#report_ok("Sniprun binary found")
  else
    call health#report_error("Sniprun binary not found")
  endif

  if executable('cargo')
    call health#report_ok("Rust toolchain available")
  else
    call health#report_warn("Rust toolchain not available", ["[optionnal] Install the rust toolchain https://www.rust-lang.org/tools/install"])
  endif

   
endfunction
