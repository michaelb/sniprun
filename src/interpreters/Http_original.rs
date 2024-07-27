use crate::interpreters::import::*;

use http_rest_file::model::{FileParseResult, HttpMethod, RequestTarget, WithDefault};
use std::io::Cursor;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Http_original {
    data: DataHolder,
    support_level: SupportLevel,
    code: String,
}

impl ReplLikeInterpreter for Http_original {}

impl Interpreter for Http_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Http_original> {
        Box::new(Http_original {
            data,
            support_level,
            code: String::new(),
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("HTTP"),
            String::from("http"),
            String::from("rest"),
        ]
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
            self.code.clone_from(&self.data.current_bloc);

        // if there is only data on current line / or Line is the max support level
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code.clone_from(&self.data.current_line);
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
        let FileParseResult { requests, errs } = http_rest_file::Parser::parse(&self.code, false);

        if !errs.is_empty() {
            return Err(SniprunError::RuntimeError(format!("{errs:?}")));
        }

        if requests.is_empty() {
            return Err(SniprunError::RuntimeError("No requests".to_string()));
        }

        let mut responses = Vec::new();

        for req in requests.into_iter() {
            let line = req.request_line;

            let url = match line.target {
                RequestTarget::Absolute { uri } => uri,
                RequestTarget::RelativeOrigin { uri } => uri,
                _ => return Err(SniprunError::RuntimeError("Invalid url".to_string())),
            };

            let mut r = match line.method {
                WithDefault::Some(HttpMethod::DELETE) => ureq::delete(&url),
                WithDefault::Some(HttpMethod::GET) => ureq::get(&url),
                WithDefault::Some(HttpMethod::PATCH) => ureq::patch(&url),
                WithDefault::Some(HttpMethod::POST) => ureq::post(&url),
                WithDefault::Some(HttpMethod::PUT) => ureq::put(&url),
                _ => {
                    return Err(SniprunError::InterpreterLimitationError(
                        "Unsupported method".to_string(),
                    ))
                }
            };

            for header in req.headers.into_iter() {
                r = r.set(&header.key, &header.value);
            }

            match r.send(Cursor::new(req.body.to_string())) {
                Ok(resp) => {
                    let status = resp.status();
                    responses.push(
                        resp.into_string().unwrap_or("".to_string())
                            + "--- status : "
                            + &status.to_string()
                            + " ---",
                    );
                }
                Err(why) => {
                    return Err(SniprunError::CustomError(format!(
                        "Error sending request: {why}"
                    )));
                }
            }
        }

        Ok(responses.join("\n---\n\n"))
    }
}

#[cfg(test)]
mod test_http_original {
    use super::*;
    use serial_test::serial;
    use ureq::serde_json;

    #[test]
    #[serial]
    fn simple_http_get() {
        let mut data = DataHolder::new();

        data.current_bloc = String::from("GET https://httpbin.org/get");

        let mut interpreter = Http_original::new(data);
        let res = interpreter.run();

        assert!(res.is_ok(), "Could not run http interpreter");
        let data = res.ok().unwrap();
        let (body, status) = data.split_once("---").unwrap();

        let v: serde_json::Value = serde_json::from_str(body).unwrap();
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["url"], "https://httpbin.org/get".to_owned());

        assert!(status.contains("200"));
    }

    #[test]
    #[serial]
    fn simple_http_get_long() {
        let data = DataHolder {
            current_bloc: String::from("GET https://httpbin.org/get"),
            ..Default::default()
        };

        println!("{:?}", data.interpreter_options);
        println!("{:?}", Http_original::error_truncate(&data));

        let mut interpreter = Http_original::new(data);
        let res = interpreter.run();

        assert!(res.is_ok(), "Could not run http interpreter");
        let data = res.ok().unwrap();
        let (body, status) = data.split_once("---").unwrap();

        let v: serde_json::Value = serde_json::from_str(body).unwrap();
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["url"], "https://httpbin.org/get".to_owned());

        assert!(status.contains("200"));
    }

    #[test]
    #[serial]
    fn simple_http_get_multiple() {
        let data = DataHolder {
            current_bloc: String::from(
                r####"
GET https://httpbin.org/get
###
GET https://httpbin.org/anything
"####,
            ),
            ..Default::default()
        };

        let mut interpreter = Http_original::new(data);
        let res = interpreter.run();

        assert!(res.is_ok(), "Could not run http interpreter");

        let data = res.ok().unwrap();

        let v: Vec<&str> = data.split("---").collect();

        println!("{v:?}");

        // Body + Status + newline per request
        assert_eq!(v.len(), 6);
    }

    #[test]
    #[serial]
    fn simple_http_post() {
        let mut data = DataHolder::new();

        data.current_bloc = String::from(
            r#"
POST https://httpbin.org/post

{
    "foo": "bar"
}
"#,
        );
        let mut interpreter = Http_original::new(data);
        let res = interpreter.run();

        assert!(res.is_ok(), "Could not run http interpreter");
        let data = res.ok().unwrap();
        let (body, status) = data.split_once("---").unwrap();

        let v: serde_json::Value = serde_json::from_str(body).unwrap();
        // println!("{}", serde_json::to_string_pretty(&v).unwrap());

        let j: serde_json::Value = serde_json::from_str(&v["json"].to_string()).unwrap();
        // println!("{}", serde_json::to_string_pretty(&foo).unwrap());

        assert_eq!(v["url"], "https://httpbin.org/post".to_owned());
        assert_eq!(j["foo"], "bar".to_owned());

        assert!(status.contains("200"));
    }
}
