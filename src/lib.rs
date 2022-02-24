// TODO: 
// - Expire cache using cache headers or default
// - Error handling
// - Passing version
// - Config for setting auth and url instead of using env vars

//use std::ffi::c_void
use std::{ffi::CStr, sync::Mutex, env, collections::HashMap};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use cache_control::CacheControl;
use chrono::{Utc, DateTime};

const URL: &str = "api.kokocares.org/keywords";
const CACHE_EXPIRATION_DEFAULT: i64 = 60;

#[derive(Deserialize, Debug)]
struct Keywords {
    pub regexes: Vec<String>,
    pub preprocess: String,
}

struct KeywordsCache {
    pub expires_at: i64,
    pub keywords: Keywords
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    pub keywords: Keywords,
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

    pub fn verify(&mut self, keyword: &str, filter: &str) -> bool {
        let cache_key = filter;

        if let Some(keyword_cache) = self.keywords.get(cache_key) {
            if Utc::now().timestamp() > keyword_cache.expires_at  {
                let re = Regex::new(&keyword_cache.keywords.preprocess).unwrap();
                let keyword = re.replace_all(keyword, "");

                for re_keyword in &keyword_cache.keywords.regexes {
                    let re = Regex::new(re_keyword).unwrap();
                    if re.is_match(&keyword) {
                        return true;
                    }
                }

                return false
            } else {
                self.load_cache(cache_key);
                self.verify(keyword, filter)
            }
        } else {
            self.load_cache(cache_key);
            self.verify(keyword, filter)
        }
    }

    pub fn load_cache(&mut self, cache_key: &str) -> u8 {
        println!("Loading regex for filter '{}'", cache_key);
        let response = ureq::get(&self.url)
            .query("filter", cache_key)
            .call()
            .expect("Can't fetch");

        let expires_in = match CacheControl::from_value(response.header("cache-control").unwrap_or_default())  {
            Some(cache_control) => match cache_control.max_age {
                Some(max_age) => max_age.as_secs() as i64,
                None => CACHE_EXPIRATION_DEFAULT,
            },
            None => CACHE_EXPIRATION_DEFAULT,
        };

        let api_response: ApiResponse = serde_json::from_reader(response.into_reader()).expect("Can't parse");
        let keywords_cache = KeywordsCache {
            keywords: api_response.keywords,
            expires_at: Utc::now().timestamp() + expires_in,
        };
        self.keywords.insert(cache_key.to_string(), keywords_cache);
        0
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
pub extern "C" fn koko_keywords_match(input: *const i8, filter: *const i8) -> bool {
    let input = unsafe { CStr::from_ptr(input) };
    let input = input.to_str().expect("UTF8 string expected");
    let filter = unsafe { CStr::from_ptr(filter) };
    let filter = filter.to_str().expect("UTF8 string expected");

    println!("We are called with: '{}', '{}'", input, filter);

    MATCHER.lock().unwrap().verify(input, filter)
}
