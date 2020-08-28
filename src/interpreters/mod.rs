include!("Bash_original.rs");
include!("Lua_nvim.rs");
include!("Generic.rs");
include!("Rust_original.rs");
include!("Python3_original.rs");
include!("import.rs");
include!("C_original.rs");
#[macro_export]
    macro_rules! iter_types {
    ($($code:tt)*) => {
{
            type Current = interpreters::Bash_original;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::Lua_nvim;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::Generic;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::Rust_original;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::Python3_original;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::C_original;
                $(
                    $code
                 )*
                };
     };
}
