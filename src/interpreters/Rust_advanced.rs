//Interpreter:| Rust_advanced       | rust        |
//############|_____________________|_____________|________________<- delimiters to help formatting,
//###########| Interpretername      | language    | comment
// Keep (but modify the first line after the :) if you wish to have this interpreter listedvia SnipList
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Rust_advanced {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    code_deps: String,
    resolved_range: Vec<Range>,
    unresolved_symbols: Vec<symbols>,

    ///specific to rust
    rust_work_dir: String,
    bin_path: String,
    main_file_path: String,
}

extern "C" {
    fn tree_sitter_rust() -> Language;
}
fn geq(point1: Point, point2: Point) -> bool {
    // return if point1 if after or eq point2
    return point1.row >= point2.row && point1.column >= point2.column;
}
fn leq(point1: Point, point2: Point) -> bool {
    // return if point1 precede or eq point2
    return point1.row <= point2.row && point1.column <= point2.column;
}

///symbols that need to be resolved
#[derive(Debug, Clone)]
enum symbols {
    Imports(String),
    Function(String),
    Variable(String),
    Method(String), //methods + static methods have to pull the whole class with them
    StaticMethod(String),
}

impl Rust_advanced {
    fn get_code_deps(&mut self) -> Result<(), SniprunError> {
        info!("creating parser & language");
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_rust() };
        parser.set_language(language).unwrap();
        let source_code = read_to_string(self.data.filepath.clone()).unwrap();

        info!("created parser & language");

        let tree = parser.parse(source_code.clone(), None).unwrap();
        info!("created tree");
        let root_node = tree.root_node();

        info!("created root node");
        let query = Query::new(
            language,
            "(call_expression
            function: (identifier) @capturename)",
        )
        .unwrap();
        info!("created query");

        let mut querycursor = QueryCursor::new();

        info!("tree {:?} ", tree);
        // info!(" root_node {:?}", root_node.to_sexp());
        info!("query {:?}", query);
        for qmatch in querycursor.captures(&query, root_node, |_| "") {
            info!(
                "capture: {:?}",
                qmatch.0.captures[0]
                    .node
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
            );
        }

        info!("captured_names : {:?}", query.capture_names());

        Ok(())
    }
}
impl Interpreter for Rust_advanced {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Rust_advanced> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/rust_advanced";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for rust-advanced");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd.clone() + "/main.rs";
        let bp = String::from(&mfp[..mfp.len() - 3]); // remove extension so binary is named 'main'
        Box::new(Rust_advanced {
            data,
            support_level,
            code: String::from(""),
            code_deps: String::from(""),
            unresolved_symbols: vec![],
            resolved_range: vec![],
            rust_work_dir: rwd,
            bin_path: bp,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("rust"),
            String::from("rust-lang"),
            String::from("rs"),
        ]
    }

    fn get_name() -> String {
        String::from("Rust_advanced")
    }

    fn get_current_level(&self) -> SupportLevel {
        self.support_level
    }
    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level;
    }

    fn get_data(&self) -> DataHolder {
        self.data.clone()
    }

    fn get_max_support_level() -> SupportLevel {
        SupportLevel::Unsupported
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        //add code from data to self.code
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.support_level >= SupportLevel::Bloc
        {
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(" ", "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }
        self.get_code_deps()
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        self.code = String::from("fn main() {") + &self.code + "}";
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for rust-advanced");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for rust-advanced");

        //compile it (to the bin_path that arleady points to the rigth path)
        let output = Command::new("rustc")
            .arg("-O")
            .arg("--out-dir")
            .arg(&self.rust_work_dir)
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            return Err(SniprunError::CompilationError("".to_string()));
        } else {
            return Ok(());
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new(&self.bin_path)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout).unwrap());
        } else {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ));
        }
    }
}
