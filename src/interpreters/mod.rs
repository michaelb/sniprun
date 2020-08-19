include!("Python3.rs");
                                          #[macro_export]
        macro_rules! iter_types {
    ($($code:tt)*) => {
{type Current = interpreters::Python3;
                $(
                    $code
                 )*
                };
                             };
                             }