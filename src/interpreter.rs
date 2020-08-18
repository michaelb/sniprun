enum SupportLevel {
    Smart = 0,
    Line = 1,
    Bloc = 2,
    File = 10,
    Project = 20,
    System = 30,
}

trait Interpreter {
    //create
    fn new(data: DataHolder) -> Self;

    fn get_support_level(&self) -> SupportLevel {
        return SupportLevel::Line;
    }

    fn get_data(&self) -> DataHolder;

    fn fetch_code(&self);
    fn add_boilerplate(&self);
    fn build(&self) -> String; //return path to executable
    fn run(&self) -> Result(String, String); //return the output of the function
}
