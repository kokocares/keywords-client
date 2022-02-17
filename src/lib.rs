// TODO: 
// - Expire cache using cache headers or default

//use std::ffi::c_void
use std::{ffi::CStr, sync::Mutex, env, collections::HashMap};
use lazy_static::lazy_static;

const URL: &str = "api.kokocares.org/keywords";

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
    pub regexes: HashMap<String, RegexResponse>,
    pub url: String,
}

impl KeywordMatcher {
    pub fn new() -> Self {
        let url = match (env::var("URL").ok(), env::var("AUTH").ok()) {
            (Some(_), Some(_)) => panic!("AUTH and URL are mutually exclusive. Put the auth in the URL itself"),
            (Some(url), None) => url,
            (None, Some(auth)) => format!("https://{}@{}", auth, URL),
            (None, None) => panic!("you must provide AUTH or URL"),
        };
        
        Self {
            regexes: HashMap::new(),
            url
        }
    }

    pub fn match_keyword(&mut self, keyword: &str, filter: &str) -> bool {

        if let Some(regex) = self.regexes.get(filter) {
            println!("Maching on '{}' with filter '{}'", keyword, filter);

            let re = Regex::new(&regex.preprocess).unwrap();
            let keyword = re.replace_all(keyword, "");

            for re_keyword in &regex.keywords {
                let re = Regex::new(re_keyword).unwrap();
                if re.is_match(&keyword) {
                    return true;
                }
            }

            false
        } else {
            println!("Loading regex for filter '{}'", filter);
            let r = ureq::get(&self.url)
                .query("filter", filter)
                .call()
                .expect("Can't fetch");

            let r: Response = serde_json::from_reader(r.into_reader()).expect("Can't parse");
            self.regexes.insert(filter.to_string(), r.regex);

            self.match_keyword(keyword, filter)
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_match_keyword() {
//         let x = KeywordMatcher { regex: RegexResponse {
//             keywords: vec!["blah".to_string()],
//             preprocess: "yes".to_string(),
//         }};

//         //assert!(x.match_keyword("yadiyada"));
//         assert!(!x.match_keyword("yadiyqweqweada"));
//     }
// }

lazy_static! {
    static ref MATCHER: Mutex<KeywordMatcher> =
        Mutex::new(KeywordMatcher::new());
}

#[no_mangle]
pub extern "C" fn match_keywords(input: *const i8, filter: *const i8) -> bool {
    let input = unsafe { CStr::from_ptr(input) };
    let input = input.to_str().expect("UTF8 string expected");
    let filter = unsafe { CStr::from_ptr(filter) };
    let filter = filter.to_str().expect("UTF8 string expected");

    println!("We are called with: '{}', '{}'", input, filter);

    MATCHER.lock().unwrap().match_keyword(&input, &filter)
}
