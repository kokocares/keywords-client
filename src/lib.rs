// TODO: 
// - Passing version
// - Error handling
// - Config for setting auth and url instead of using env vars
// - Packaging up python lib
// - CI/CD
// - Tests
// - Documentation

use std::{ffi::CStr, sync::Mutex, env, collections::HashMap};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use cache_control::CacheControl;
use chrono::Utc;

const URL: &str = "api.kokocares.org/keywords";
const CACHE_EXPIRATION_DEFAULT: i64 = 60;

#[derive(Deserialize, Debug)]
struct Keywords {
    pub keywords: Vec<String>,
    pub preprocess: String,
}

struct KeywordsCache {
    pub expires_at: i64,
    pub keywords: Keywords
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    pub regex: Keywords,
}

struct KokoKeywords {
    pub keywords: HashMap<String, KeywordsCache>,
    pub url: String,
}

impl KokoKeywords {
    pub fn new() -> Self {
        let url = match (env::var("KOKO_KEYWORDS_URL").ok(), env::var("KOKO_KEYWORDS_AUTH").ok()) {
            (Some(_), Some(_)) => panic!("AUTH and URL are mutually exclusive. Put the auth in the URL itself"),
            (Some(url), None) => url,
            (None, Some(auth)) => format!("https://{}@{}", auth, URL),
            (None, None) => panic!("you must provide AUTH or URL"),
        };
        
        Self {
            keywords: HashMap::new(),
            url
        }
    }

    pub fn verify(&mut self, keyword: &str, filter: &str, version: Option<&str>) -> bool {
        let cache_key = format!("{}{}", filter, version.unwrap_or_default());

        if let Some(keyword_cache) = self.keywords.get(&cache_key) {
            if Utc::now().timestamp() < keyword_cache.expires_at  {
                let re = Regex::new(&keyword_cache.keywords.preprocess).unwrap();
                let keyword = re.replace_all(keyword, "");

                for re_keyword in &keyword_cache.keywords.keywords {
                    let re = Regex::new(re_keyword).unwrap();
                    if re.is_match(&keyword) {
                        return true;
                    }
                }

                return false
            } else {
                self.load_cache(filter, version);
                self.verify(keyword, filter, version)
            }
        } else {
            self.load_cache(filter, version);
            self.verify(keyword, filter, version)
        }
    }

    pub fn load_cache(&mut self, filter: &str, version: Option<&str>) {
        let cache_key = format!("{}{}", filter, version.unwrap_or_default());

        println!("Loading cache for key '{}'", cache_key);

        let request = ureq::get(&self.url);

        let request = request.query("filter", filter);
        let request = if let Some(version) = version {
            request.query("version", version)
        } else {
            request
        };

        let response = request.call().expect("Can't fetch");

        let expires_in = response.header("cache-control")
            .map(CacheControl::from_value)
            .flatten()
            .map(|cc| cc.max_age)
            .flatten()
            .map(|max_age| max_age.as_secs() as i64)
            .unwrap_or(CACHE_EXPIRATION_DEFAULT);

        let api_response: ApiResponse = serde_json::from_reader(response.into_reader()).expect("Can't parse");
        let keywords_cache = KeywordsCache {
            keywords: api_response.regex,
            expires_at: Utc::now().timestamp() + expires_in,
        };
        self.keywords.insert(cache_key.to_string(), keywords_cache);
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
    static ref MATCHER: Mutex<KokoKeywords> =
        Mutex::new(KokoKeywords::new());
}

#[no_mangle]
pub extern "C" fn koko_keywords_match(input: *const i8, filter: *const i8, version: *const i8,) -> bool {
    let input = str_from_c(input).expect("Input is required");
    let filter = str_from_c(filter).expect("Filter is required");
    let version = str_from_c(version);

    println!("Calling with {:?}, {:?}, {:?}", input, filter, version);

    MATCHER.lock().unwrap().verify(input, filter, version)
}

pub fn str_from_c<'a>(c_str: *const i8) -> Option<&'a str> {
    if c_str.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(c_str) }
                .to_str().expect("Malformed UTF-8 string")
        )
    }
}
