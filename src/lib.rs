use cache_control::CacheControl;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::time::Duration;
use std::{collections::HashMap, env, ffi::CStr, fmt, ops, sync::Mutex, time::SystemTime};
use ureq::{Error, ErrorKind, Response};

const URL: &str = "api.kokocares.org/keywords";
const CACHE_EXPIRATION_DEFAULT: Duration = Duration::from_secs(3600);

#[derive(Clone, Debug)]
pub struct Regex(regex::Regex);

impl ops::Deref for Regex {
    type Target = regex::Regex;
    fn deref(&self) -> &regex::Regex {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for Regex {
    fn deserialize<D>(de: D) -> Result<Regex, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Visitor};

        struct RegexVisitor;

        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = Regex;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a regular expression pattern")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Regex, E> {
                regex::Regex::new(v)
                    .map(Regex)
                    .map_err(|err| E::custom(err.to_string()))
            }
        }

        de.deserialize_str(RegexVisitor)
    }
}

type KokoResult<T> = Result<T, KokoError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KokoError {
    AuthOrUrlMissing = -1,
    InvalidCredentials = -2,
    ParseError = -4,
    InvalidUrl = -5,
}

#[derive(Deserialize, Debug)]
struct Keywords {
    pub keywords: Vec<Keyword>,
    pub preprocess: Regex,
}

#[derive(Deserialize, Debug)]
struct Keyword {
    pub regex: Regex,
    pub category: String,
    pub severity: String,
    pub confidence: String,
}

struct KeywordsCache {
    pub expires_at: SystemTime,
    pub keywords: Keywords,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    pub regexes: Keywords,
}

struct KokoKeywords {
    pub keywords: HashMap<String, KeywordsCache>,
    pub url: String,
    pub default: String,
}

impl KokoKeywords {
    pub fn new(url: String) -> Self {
        Self {
            keywords: HashMap::new(),
            url,
            default: include_str!("keywords.json").to_string()
        }
    }

    pub fn verify(
        &mut self,
        keyword: &str,
        filter: &str,
        version: Option<&str>,
    ) -> KokoResult<bool> {
        let cache_key = format!("{}", version.unwrap_or("latest"));

        let keyword_cache = if let Some(keyword_cache) = self.keywords.get(&cache_key) {
            if SystemTime::now() < keyword_cache.expires_at {
                keyword_cache
            } else {
                self.load_cache(version)?;

                match self.keywords.get(&cache_key) {
                    Some(keyword_cache) => keyword_cache,
                    None => panic!("Cache not loaded, this is a bug"),
                }
            }
        } else {
            self.load_cache(version)?;

            match self.keywords.get(&cache_key) {
                Some(keyword_cache) => keyword_cache,
                None => panic!("Cache not loaded, this is a bug"),
            }
        };


        let keyword = keyword_cache
            .keywords
            .preprocess
            .replace_all(keyword, "")
            .to_lowercase();

        'keyword_loop: for re_keyword in &keyword_cache.keywords.keywords {
            let filters: Vec<&str> = filter.split(":").collect();

            for filter in filters {
                if filter == "" {
                    continue;
                }

                let filter: Vec<&str> = filter.split("=").collect();
                let filter_key = filter[0];
                let filter_values = filter[1];

                let filter_matched = match filter_key {
                    "category" => filter_values.contains(&re_keyword.category),
                    "severity" => filter_values.contains(&re_keyword.severity),
                    "confidence" => filter_values.contains(&re_keyword.confidence),
                    _ => false,
                };

                if !filter_matched {
                    continue 'keyword_loop;
                }
            }

            if re_keyword.regex.is_match(&keyword) {
                return Ok(true);
            }
        }

        return Ok(false);
    }

    pub fn load_cache(&mut self, version: Option<&str>) -> KokoResult<()> {
        let cache_key = format!("{}", version.unwrap_or("latest"));

        eprintln!("[koko-keywords] Loading cache for '{}'", cache_key);

        let request = ureq::get(&self.url).set("X-API-VERSION", "v2");

        let request = if let Some(version) = version {
            request.query("version", version)
        } else {
            request
        };

        let response = match request.call() {
            Ok(response) => Ok(response),
            Err(Error::Transport(tranport_error)) => {
                if tranport_error.kind() == ErrorKind::InvalidUrl {
                    Err(KokoError::InvalidUrl)
                } else {
                    Ok(Response::new(200, "Success", self.default.as_str()).unwrap())
                }
            }
            Err(Error::Status(403, _)) => Err(KokoError::InvalidCredentials),
            Err(Error::Status(status, response)) => {
                eprintln!("[koko-keywords] Failed to load cache ({}: {})", status, response.status_text());
                Ok(Response::new(200, "Success", self.default.as_str()).unwrap())
            }
        }?;

        let expires_in = response
            .header("cache-control")
            .map(CacheControl::from_value)
            .flatten()
            .map(|cc| cc.max_age)
            .flatten()
            .unwrap_or(CACHE_EXPIRATION_DEFAULT);

        let json_body = match response.into_string() {
            Ok(json) => Ok(json),
            Err(_) => Err(KokoError::ParseError),
        }?;

        let api_response: ApiResponse = match serde_json::from_str(&json_body) {
            Ok(response) => Ok(response),
            Err(_) => {
                Err(KokoError::ParseError)
            }
        }?;

        let keywords_cache = KeywordsCache {
            keywords: api_response.regexes,
            expires_at: SystemTime::now() + expires_in,
        };
        self.keywords.insert(cache_key.to_string(), keywords_cache);
        self.default = json_body;

        Ok(())
    }
}

lazy_static! {
    static ref MATCHER: Mutex<KokoResult<KokoKeywords>> =
        Mutex::new(get_url().map(KokoKeywords::new));
}

pub fn get_url() -> KokoResult<String> {
    match (
        env::var("KOKO_KEYWORDS_URL").ok(),
        env::var("KOKO_KEYWORDS_AUTH").ok(),
    ) {
        (Some(_), Some(_)) => Err(KokoError::AuthOrUrlMissing),
        (Some(url), None) => Ok(url),
        (None, Some(auth)) => Ok(format!("https://{}@{}", auth, URL)),
        (None, None) => Err(KokoError::AuthOrUrlMissing),
    }
}

fn str_from_c<'a>(c_str: *const std::os::raw::c_char) -> Option<&'a str> {
    if c_str.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(c_str) }
                .to_str()
                .expect("Malformed UTF-8 string"),
        )
    }
}

pub fn koko_keywords_match(input: &str, filter: &str, version: Option<&str>) -> KokoResult<bool> {
    MATCHER
        .lock()
        .unwrap()
        .as_mut()
        .map_err(|e| e.clone())?
        .verify(input, filter, version)
}

#[no_mangle]
pub extern "C" fn c_koko_keywords_match(
    input: *const std::os::raw::c_char,
    filter: *const std::os::raw::c_char,
    version: *const std::os::raw::c_char,
) -> isize {
    let input = str_from_c(input).expect("Input is required");
    let filter = str_from_c(filter).expect("Filter is required");
    let version = str_from_c(version);

    let result = koko_keywords_match(input, filter, version);
    match result {
        Ok(r) => {
            if r {
                1
            } else {
                0
            }
        }
        Err(e) => e as isize,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    const DEFAULT_KEYWORDS: &str = r#"{ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "severity":"high", "confidence":"high"}, {"regex":"^a4a$", "category":"eating", "severity": "medium", "confidence":"high"}], "preprocess": " "} }"#;

    #[test]
    fn test_failing_server() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(500);
        });

        let mut x = KokoKeywords {
            keywords: HashMap::new(),
            url: server.url("/keywords"),
            default: DEFAULT_KEYWORDS.to_string(),
        };

        assert_eq!(x.verify("hello", "", None), Ok(false));
        assert_eq!(x.verify("kms", "", None), Ok(true));
        keyword_mock.assert();
    }

    #[test]
    fn test_invalid_response() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({}));
        });

        let mut x = KokoKeywords {
            keywords: HashMap::new(),
            url: server.url("/keywords"),
            default: DEFAULT_KEYWORDS.to_string(),
        };

        assert_eq!(x.verify("hello", "", None), Err(KokoError::ParseError));
        keyword_mock.assert();
    }

    #[test]
    fn test_invalid_url() {
        let mut x = KokoKeywords {
            keywords: HashMap::new(),
            url: "".to_string(),
            default: DEFAULT_KEYWORDS.to_string(),
        };

        assert_eq!(x.verify("hello", "", None), Err(KokoError::InvalidUrl));
    }

    #[test]
    fn test_valid_response() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "severity":"high", "confidence":"high"}], "preprocess": " "}}));
        });

        let mut x = KokoKeywords {
            keywords: HashMap::new(),
            url: server.url("/keywords"),
            default: DEFAULT_KEYWORDS.to_string(),
        };

        assert_eq!(x.verify("hello", "", None), Ok(false));
        assert_eq!(x.verify("kms", "", None), Ok(true));
        assert_eq!(x.verify(" kms   ", "", None), Ok(true));
        keyword_mock.assert();
    }

    #[test]
    fn test_expired_cache() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "severity":"high", "confidence":"high"}, {"regex": "^suicide$", "category":"suicide", "severity":"high", "confidence":"high"}], "preprocess": " "}}));
        });

        let api_response: ApiResponse =
            serde_json::from_str(r#"{ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "severity":"high", "confidence":"high"}], "preprocess": " "} }"#).unwrap();
        let mut x = KokoKeywords {
            keywords: HashMap::from([(
                "latest".to_string(),
                KeywordsCache {
                    keywords: api_response.regexes,
                    expires_at: SystemTime::now() - Duration::new(1000, 0),
                },
            )]),
            url: server.url("/keywords"),
            default: DEFAULT_KEYWORDS.to_string(),
        };

        assert_eq!(x.verify("suicide", "", None), Ok(true));
        keyword_mock.assert();
    }

    #[test]
    fn test_expired_cache_with_failing_request() {
        let server = httpmock::MockServer::start();
        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(200)
                .header("cache-control", "max-age=0")
                .header("content-type", "application/json")
                .json_body(json!({ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "severity":"high", "confidence":"high"}, {"regex": "^suicide$", "category":"suicide", "severity":"high", "confidence":"high"}], "preprocess": " "}}));
        });

        let mut x = KokoKeywords {
            keywords: HashMap::new(),
            url: server.url("/keywords"),
            default: DEFAULT_KEYWORDS.to_string(),
        };

        assert_eq!(x.verify("suicide", "", None), Ok(true));
        keyword_mock.assert();

        let server = httpmock::MockServer::start();
        let keyword_failing_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(500);
        });
        x.url = server.url("/keywords");

        assert_eq!(x.verify("suicide", "", None), Ok(true));
        keyword_failing_mock.assert();
    }

    #[test]
    fn test_case_insensitive() {
        let api_response: ApiResponse =
            serde_json::from_str(r#"{ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "severity":"high", "confidence":"high"}], "preprocess": " "} }"#).unwrap();
        let mut x = KokoKeywords {
            keywords: HashMap::from([(
                "latest".to_string(),
                KeywordsCache {
                    keywords: api_response.regexes,
                    expires_at: SystemTime::now() + Duration::new(1000, 0),
                },
            )]),
            url: "http://localhost".to_string(),
            default: DEFAULT_KEYWORDS.to_string(),
        };

        assert_eq!(x.verify("kms", "", None), Ok(true));
    }

    #[test]
    fn test_filters() {
        let api_response: ApiResponse =
            serde_json::from_str(r#"{ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "severity":"high", "confidence":"high"}, {"regex":"^a4a$", "category":"eating", "severity": "medium", "confidence":"high"}, {"regex": "^suicidal$", "category":"suicide", "severity":"medium", "confidence":"high"}], "preprocess": " "} }"#).unwrap();
        let mut x = KokoKeywords {
            keywords: HashMap::from([(
                "latest".to_string(),
                KeywordsCache {
                    keywords: api_response.regexes,
                    expires_at: SystemTime::now() + Duration::new(1000, 0),
                },
            )]),
            url: "http://localhost".to_string(),
            default: DEFAULT_KEYWORDS.to_string(),
        };

        assert_eq!(x.verify("kms", "category=suicide", None), Ok(true));
        assert_eq!(x.verify("kms", "category=eating", None), Ok(false));
        assert_eq!(
            x.verify("kms", "category=suicide:severity=medium", None),
            Ok(false)
        );
        assert_eq!(
            x.verify("kms", "category=suicide:severity=high", None),
            Ok(true)
        );
        assert_eq!(
            x.verify("suicidal", "category=suicide:severity=high", None),
            Ok(false)
        );
    }
}
