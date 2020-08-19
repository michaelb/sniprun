struct Foo;
impl Foo {
    fn count() -> u8 {
        1
    }
}

struct Bar;
impl Bar {
    fn count() -> u8 {
        2
    }
}

struct FooBar;
impl FooBar {
    fn count() -> u8 {
        3
    }
}

}
macro_rules! iter_types {
    ($($code:tt)*) => {
        {
            type Current = Foo;
            $(
                $code
             )*
        };
        {
            type Current = Bar;
            $(
                $code
             )*
        };
        {
            type Current = FooBar;
            $(
                $code
             )*
        };
    };
}

fn main() {
    iter_types! {
        println!("count = {}", Current::count());
    }
}

#[macro_export]
macro_rules! include_proto {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}
