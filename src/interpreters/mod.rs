include!("Python3.rs");
include!("Rust.rs");
include!("import.rs");
include!("Bash.rs");
#[macro_export]
    macro_rules! iter_types {
    ($($code:tt)*) => {
{
            type Current = interpreters::Python3;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::Rust;
                $(
                    $code
                 )*
                };{
            type Current = interpreters::Bash;
                $(
                    $code
                 )*
                };
     };
}
