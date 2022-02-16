//use std::ffi::c_void
use std::{ffi::CStr, sync::Mutex};
use lazy_static::lazy_static;

const URL: &str = "api.kokocares.org/keywords";
const AUTH: &str = "koko:5e6c5a52580bcfeb5ee5c3997322946c186a001e848a3f755074a0a337e2565d";

use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct RegexResponse {
    pub keywords: Vec<String>,
    pub preprocess: String,
}

#[derive(Deserialize, Debug)]
struct Response {
    pub version: String,
    pub regex: RegexResponse,
}

struct KeywordMatcher {
    pub regex: RegexResponse,
}


impl KeywordMatcher {
    pub fn new() -> Self {
        let url = format!("https://{}@{}", AUTH, URL);
        let r = ureq::get(&url)
            .call().expect("Can't fetch");

        let r: Response = serde_json::from_reader(r.into_reader()).expect("Can't parse");

        Self { regex: r.regex }
    }

    pub fn match_keyword(&self, keyword: &str) -> bool {
        println!("Maching on {}", keyword);

        let re = Regex::new(&self.regex.preprocess).unwrap();
        let keyword = re.replace_all(keyword, "");

        for re_keyword in &self.regex.keywords {
            let re = Regex::new(re_keyword).unwrap();
            if re.is_match(&keyword) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_match_keyword() {
        let x = KeywordMatcher { regex: RegexResponse {
            keywords: vec!["blah".to_string()],
            preprocess: "yes".to_string(),
        }};

        //assert!(x.match_keyword("yadiyada"));
        assert!(!x.match_keyword("yadiyqweqweada"));
    }
}

lazy_static! {
    static ref MATCHER: Mutex<KeywordMatcher> =
        Mutex::new(KeywordMatcher::new());
}

#[no_mangle]
pub extern "C" fn match_keywords(input: *const i8) -> bool {
    let input = unsafe { CStr::from_ptr(input) };
    let input = input.to_str().expect("UTF8 string expected");

    println!("We are called with: '{}'", input);

    MATCHER.lock().unwrap().match_keyword(&input)
}
