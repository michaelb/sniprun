use super::*;
use interpreter::{Interpreter, InterpreterUtils, SupportLevel};
use interpreters::JS_original;

#[test]
fn test_implements() {
    let data = DataHolder::new();
    iter_types! {
        let mut interpreter = Current::new(data.clone());
        let _ = Current::get_name();
        let _ = Current::default_for_filetype();
        let _ = Current::get_supported_languages();
        let max_level = Current::get_max_support_level();
        let current_level = interpreter.get_current_level();
        assert_eq!(max_level, current_level);
        interpreter.set_current_level(SupportLevel::Selected);
        assert_eq!(SupportLevel::Selected, interpreter.get_current_level());
        let _ = interpreter.get_data();
        // let _ = interpreter.fallback(); // don't test, this is a 'run' hidden
        let _ = Current::behave_repl_like_default();
        let _ = Current::has_repl_capability();
        let _ = Current::has_treesitter_capability();
    }
}

#[test]
fn test_interpreter_utils() {
    let mut data = DataHolder::new();
    data.interpreter_data = Some(Arc::new(Mutex::new(InterpreterData {
        owner: String::new(),
        content: String::new(),
        pid: Some(0),
    })));
    data.current_bloc = String::from("console.log(\"Hello, World!\");");
    let mut interpreter = JS_original::new(data);
    interpreter.save_code(String::from("let a = 3;"));
    assert_eq!(
        String::from("let a = 3;"),
        interpreter.read_previous_code().trim_matches('\n')
    );
    interpreter.clear();
    assert!(interpreter.read_previous_code().is_empty());

    interpreter.set_pid(15);
    assert_eq!(Some(15), interpreter.get_pid());

    // actually run the JS_original interpreter since we highjacked its test
    let res = interpreter.run();
    let string_result = res.unwrap();
    assert_eq!(string_result, "Hello, World!\n");
}
