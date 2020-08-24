include!("Python3_original.rs");
include!("Rust_original.rs");
include!("Bash_original.rs");
include!("C_original.rs");
include!("import.rs");
#[macro_export]
    macro_rules! iter_types {
    ($($code:tt)*) => {
{
            type Current = interpreters::Python3_original;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::Rust_original;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::Bash_original;
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
