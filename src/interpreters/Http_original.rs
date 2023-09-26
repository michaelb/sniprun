use http_rest_file::model::{WithDefault, RequestTarget, HttpMethod };

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Http_original {
    data: DataHolder,
    support_level: SupportLevel,
    code: String,
}

impl ReplLikeInterpreter for Http_original {}

impl Interpreter for Http_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Http_original>{
        Box::new(Http_original{
            data,
            support_level,
            code: String::new(),
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("http")]
    }

    fn get_name() -> String {
        String::from("Http_original")
    }

    fn behave_repl_like_default() -> bool {
        false
    }
    fn has_repl_capability() -> bool {
        false
    }

    fn default_for_filetype() -> bool {
        true
    }

    fn get_current_level(&self) -> SupportLevel{
        self.support_level
    }

    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level;
    }

    fn get_data(&self) -> DataHolder {
        self.data.clone()
    }

    fn get_max_support_level() -> SupportLevel {
        // TODO: idk what this is
        SupportLevel::Bloc
    }

    // copy-pasted from example
    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.support_level >= SupportLevel::Bloc
        {
            // if bloc is not pseudo empty and has Bloc current support level,
            // add fetched code to self
            self.code = self.data.current_bloc.clone();

        // if there is only data on current line / or Line is the max support level
        } else if !self.data.current_line.replace(" ", "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            // no code was retrieved
            self.code = String::from("");
        }

        // now self.code contains the line or bloc of code wanted :-)
        Ok(())    
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let model = http_rest_file::Parser::parse(&self.code, false);

        for req in model.requests.into_iter() {
            let line = req.request_line;

            let url = match line.target {
                RequestTarget::Absolute { uri } => uri,
                RequestTarget::RelativeOrigin { uri } => uri,
                _ => return Err(SniprunError::CustomError(format!("Invalid url"))),
            };

            let resp = match line.method {
                WithDefault::Some(HttpMethod::GET) => ureq::get(&url).call(),
                WithDefault::Some(HttpMethod::POST) => ureq::post(&url).send_string(&req.body.to_string()),
                _ => return Err(SniprunError::CustomError(format!("Invalid method"))),
            };

            match resp {
                Ok(resp) => match resp.into_string() {
                    Ok(text) => {
                        return Ok(text);
                    }
                    Err(why) => {
                        return Err(SniprunError::CustomError(format!("Error getting text: {why}")));
                    }
                },
                Err(why) => {
                    return Err(SniprunError::CustomError(format!("Error getting text: {why}")));
                }
            }
        }

        return Err(SniprunError::CustomError(format!("No requests")))
    }
}

#[cfg(test)]
mod test_http_original {
    use super::*;
    use ureq::serde_json;
    use serial_test::serial;

    #[test]
    #[serial]
    fn simple_http_get() {
        let mut data = DataHolder::new();

        data.current_bloc = String::from("GET https://httpbin.org/get");
        let mut interpreter = Http_original::new(data);
        let res = interpreter.run();

        assert!(!res.is_err(), "Could not run http interpreter");
        let data = res.ok().unwrap();

        let v: serde_json::Value = serde_json::from_str(&data).unwrap();
        // println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["url"], "https://httpbin.org/get".to_owned());
    }

    #[test]
    #[serial]
    fn simple_http_post() {
        let mut data = DataHolder::new();

        data.current_bloc = String::from(r#"POST https://httpbin.org/post

{
    "foo": "bar"
}
"#);
        let mut interpreter = Http_original::new(data);
        let res = interpreter.run();

        assert!(!res.is_err(), "Could not run http interpreter");
        let data = res.ok().unwrap();

        let v: serde_json::Value = serde_json::from_str(&data).unwrap();
        // println!("{}", serde_json::to_string_pretty(&v).unwrap());

        let j: serde_json::Value = serde_json::from_str(&v["json"].to_string()).unwrap();
        // println!("{}", serde_json::to_string_pretty(&foo).unwrap());

        assert_eq!(v["url"], "https://httpbin.org/post".to_owned());
        assert_eq!(j["foo"], "bar".to_owned());
    }
}
