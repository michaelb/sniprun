use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), std::io::Error> {
    //clarify this
    let out_dir = "src/interpreters";
    let dest_path = Path::new(&out_dir).join("mod.rs");

    let mut string_to_write = "".to_string();

    for path in fs::read_dir(out_dir).unwrap() {
        let plugin = path.unwrap().file_name().into_string().unwrap();
        if plugin == "mod.rs" {
            continue;
        }

        string_to_write.push_str(&format!(
            "include!(\"{}\");
                                          ",
            plugin
        ));
    }

    string_to_write.push_str(
        "#[macro_export]
        macro_rules! iter_types {
    ($($code:tt)*) => {
",
    );

    for path in fs::read_dir(out_dir).unwrap() {
        let mut plugin = path.unwrap().file_name().into_string().unwrap();
        if plugin == "mod.rs" {
            continue;
        }
        plugin = plugin[..plugin.len() - 3].to_string();

        string_to_write.push_str("{");
        string_to_write.push_str(&format!(
            "type Current = interpreters::{};
                $(
                    $code
                 )*
                ",
            plugin,
        ));
        string_to_write.push_str("};");
    }
    string_to_write.push_str(
        "
                             };
                             }",
    );

    // cargo stuff for rebuild

    for path in fs::read_dir(out_dir).unwrap() {
        let plugin_path = path.unwrap().path().display();
    }
    println!(
        "cargo:rerun-if-changed=build.rs
                             "
    );
    println!(
        "cargo:rerun-if-changed={}
                                      ",
        out_dir
    );
    for path in fs::read_dir(out_dir).unwrap() {
        println!(
            "cargo:rerun-if-changed={}
            ",
            path.unwrap().path().display()
        );
    }

    fs::write(&dest_path, string_to_write)
}
