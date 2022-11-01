use cache_control::CacheControl;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::{env, ffi::CStr, fmt, ops, sync::Mutex};
use ureq::{Error, ErrorKind};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KokoError {
    AuthOrUrlMissing = -1,
    InvalidCredentials = -2,
    CacheRefreshError = -3,
    ParseError = -4,
    InvalidUrl = -5,
    InvalidFilter = -6,
}

lazy_static! {
    static ref KOKO_ERROR_DESCRIPTIONS: HashMap<isize, &'static str> = {
        HashMap::from([
            (KokoError::AuthOrUrlMissing as isize, "KOKO_KEYWORDS_AUTH must be set before importing the library\0"),
            (KokoError::InvalidCredentials as isize, "Invalid credentials. Please confirm you are using valid credentials, contact us at api@kokocares.org if you need assistance.\0"),
            (KokoError::CacheRefreshError as isize, "Unable to refresh cache. Please try again or contact us at api@kokocares.org if this issue persists.\0)"),
            (KokoError::ParseError as isize, "Unable to parse response from API. Please contact us at api@kokocares.org if this issue persists.\0)"),
            (KokoError::InvalidUrl as isize, "Invalid url. Please ensure the url used is valid.\0"),
            (KokoError::InvalidFilter as isize, "Invalid filter, please ensure it follows the format: category=value:another_category=value,value2\0"),
        ])
    };
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
    pub confidence: String,
}

impl Keyword {
    pub fn match_filter(&self, filter_key: &str, filter_values: &str) -> bool {
        match filter_key {
            "category" => filter_values.contains(&self.category),
            "confidence" => filter_values.contains(&self.confidence),
            _ => false,
        }
    }
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    pub regexes: Keywords,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CacheError<E> {
    FatalErr(E),
    RetryableErr(E),
}

type CacheResult<T> = Result<T, CacheError<KokoError>>;

#[derive(Debug)]
struct KeywordsCache {
    pub expires_at: Instant,
    pub keywords: Keywords,
}

impl KeywordsCache {
    pub fn new(json: String, expires_at: Instant) -> Self {
        let default_response: ApiResponse =
            serde_json::from_str(json.as_str()).expect("Unable to parse default json");

        Self {
            keywords: default_response.regexes,
            expires_at,
        }
    }
}

struct KokoKeywords {
    pub keywords: KeywordsCache,
    pub url: String,
}

impl KokoKeywords {
    pub fn new(url: String, default: KeywordsCache) -> Self {
        Self {
            keywords: default,
            url,
        }
    }

    pub fn verify(&mut self, keyword: &str, filter: &str) -> KokoResult<bool> {
        let keyword_cache = if Instant::now() < self.keywords.expires_at {
            &self.keywords
        } else {
            match self.load_cache() {
                Ok(_) => (),
                Err(err) => match err {
                    CacheError::FatalErr(err) => return Err(err),
                    CacheError::RetryableErr(err) => {
                        self.keywords.expires_at = Instant::now() + CACHE_EXPIRATION_DEFAULT;
                        eprintln!("[koko-keywords] Retry will happen in {} hour(s)", CACHE_EXPIRATION_DEFAULT.as_secs() / 3600);
                        return Err(err)
                    }
                }
            };

            &self.keywords
        };

        let keyword = keyword_cache
            .keywords
            .preprocess
            .replace_all(keyword, " ")
            .to_lowercase();

        let filters = filter
            .split(':')
            .filter(|f| !f.is_empty())
            .map(|f| {
                let mut splitparts = f.split('=');
                match (splitparts.next(), splitparts.next()) {
                    (Some(k), Some(v)) => Ok((k, v)),
                    _ => Err(KokoError::InvalidFilter),
                }
            })
            .collect::<KokoResult<Vec<_>>>()?;

        Ok(keyword_cache.keywords.keywords.iter().any(|re_keyword| {
            filters.iter().all(|(k, v)| re_keyword.match_filter(k, v))
                && re_keyword.regex.is_match(&keyword)
        }))
    }

    pub fn load_cache(&mut self) -> CacheResult<()> {
        eprintln!("[koko-keywords] Loading cache ({})", self.url);

        let agent = ureq::builder()
            .user_agent("koko-keywords/0.3.1")
            .build();

        let request = agent.get(&self.url).set("X-API-VERSION", "v3");

        let response = match request.call() {
            Ok(response) => Ok(response),
            Err(Error::Transport(tranport_error))
                if tranport_error.kind() == ErrorKind::InvalidUrl =>
            {
                Err(CacheError::FatalErr(KokoError::InvalidUrl))
            }
            Err(Error::Transport(tranport_error)) => {
                eprintln!(
                    "[koko-keywords] Failed to load cache: {}",
                    tranport_error.message().unwrap_or("Unknown")
                );
                Err(CacheError::RetryableErr(KokoError::CacheRefreshError))
            }
            Err(Error::Status(401, _)) => Err(CacheError::FatalErr(KokoError::InvalidCredentials)),
            Err(Error::Status(status, response)) => {
                eprintln!(
                    "[koko-keywords] Failed to load cache: ({}) {}",
                    status,
                    response.status_text()
                );
                Err(CacheError::RetryableErr(KokoError::CacheRefreshError))
            }
        }?;

        let expires_in = response
            .header("cache-control")
            .and_then(CacheControl::from_value)
            .and_then(|cc| cc.max_age)
            .unwrap_or(CACHE_EXPIRATION_DEFAULT);

        let api_response: ApiResponse = match serde_json::from_reader(response.into_reader()) {
            Ok(response) => response,
            Err(err) => {
                eprintln!("[koko-keywords] Failed to parse: {}", err);
                return Err(CacheError::RetryableErr(KokoError::ParseError))
            }
        };

        self.keywords.keywords = api_response.regexes;
        self.keywords.expires_at = Instant::now() + expires_in;

        CacheResult::Ok(())
    }
}

lazy_static! {
    static ref MATCHER: Mutex<KokoResult<KokoKeywords>> = {
        match get_url() {
            Ok(url) => Mutex::new(Ok(KokoKeywords::new(
                url,
                KeywordsCache::new(include_str!("keywords.json").to_string(), Instant::now()),
            ))),
            Err(err) => Mutex::new(Err(err)),
        }
    };
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

pub fn koko_keywords_match(input: &str, filter: &str) -> KokoResult<bool> {
    MATCHER
        .lock()
        .unwrap()
        .as_mut()
        .map_err(|e| *e)?
        .verify(input, filter)
}

#[no_mangle]
pub extern "C" fn c_koko_keywords_match(
    input: *const std::os::raw::c_char,
    filter: *const std::os::raw::c_char,
) -> isize {
    let input = str_from_c(input).expect("Input is required");
    let filter = str_from_c(filter).expect("Filter is required");

    let result = koko_keywords_match(input, filter);
    match result {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => e as isize,
    }
}

#[no_mangle]
pub extern "C" fn c_koko_keywords_error_description(error: isize) -> *const std::os::raw::c_char {
    KOKO_ERROR_DESCRIPTIONS[&error].as_ptr() as *const std::os::raw::c_char
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    const DEFAULT_RESPONSE: &str = r#"{ "regexes": {"keywords": [{"regex": "^ *kms *$", "category":"suicide", "confidence":"high"}], "preprocess": "\\W"} }"#;

    #[test]
    fn test_koko_keywords_match_wtih_failing_server() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(500);
        });

        env::set_var("KOKO_KEYWORDS_URL", server.url("/keywords"));

        assert_eq!(
            koko_keywords_match("a4a", ""),
            Err(KokoError::CacheRefreshError)
        );
        keyword_mock.assert();

        assert_eq!(koko_keywords_match("a4a", ""), Ok(true));
        keyword_mock.assert_hits(1);
    }

    #[test]
    fn test_header_v3() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords").header("X-API-VERSION", "v3");
            then.status(200)
                .header("content-type", "application/json")
                .body(DEFAULT_RESPONSE);
        });

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        x.verify("x", "").unwrap();
        keyword_mock.assert();
    }

    #[test]
    fn test_failing_server() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(500);
        });

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(
            x.verify("kms", ""),
            Err(KokoError::CacheRefreshError)
        );

        assert_eq!(x.verify("hello", ""), Ok(false));
        assert_eq!(x.verify("kms", ""), Ok(true));
        keyword_mock.assert_hits(1);
    }

    #[test]
    fn test_invalid_credentials() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(401);
        });

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(
            x.verify("a4a", ""),
            Err(KokoError::InvalidCredentials)
        );
        assert_eq!(
            x.verify("hello", ""),
            Err(KokoError::InvalidCredentials)
        );
        keyword_mock.assert_hits(2);
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

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(
            x.verify("a4a", ""),
            Err(KokoError::ParseError)
        );

        assert_eq!(x.verify("kms", ""), Ok(true));
        keyword_mock.assert_hits(1);
    }

    #[test]
    fn test_invalid_url() {
        let mut x = KokoKeywords::new(
            "".to_string(),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(x.verify("hello", ""), Err(KokoError::InvalidUrl));
        assert_eq!(x.verify("hello", ""), Err(KokoError::InvalidUrl));
    }

    #[test]
    fn test_valid_response() {
        let server = httpmock::MockServer::start();

        let keyword_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({ "regexes": {"keywords": [{"regex": "^sewerslide$", "category":"suicide", "confidence":"high"}], "preprocess": " "}}));
        });

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(x.verify("sewerslide", ""), Ok(true));
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
                .json_body(json!({ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "confidence":"high"}, {"regex": "^suicide$", "category":"suicide", "confidence":"high"}], "preprocess": "@"}}));
        });

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(x.verify("suicide", ""), Ok(true));
        keyword_mock.assert();

        let server = httpmock::MockServer::start();
        let keyword_failing_mock = server.mock(|when, then| {
            when.path("/keywords");
            then.status(500);
        });
        x.url = server.url("/keywords");

        assert_eq!(x.verify("suicide", ""), Err(KokoError::CacheRefreshError));
        assert_eq!(x.verify("suicide", ""), Ok(true));
        keyword_failing_mock.assert_hits(1);
    }

    #[test]
    fn test_case_insensitive() {
        let server = httpmock::MockServer::start();
        let _ = server.mock(|when, then| {
            when.path("/keywords");
            then.status(200)
                .header("cache-control", "max-age=0")
                .header("content-type", "application/json")
                .body(DEFAULT_RESPONSE);
        });

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(x.verify("KMS", ""), Ok(true));
    }

    #[test]
    fn test_preprocessing() {
        let server = httpmock::MockServer::start();
        let _ = server.mock(|when, then| {
            when.path("/keywords");
            then.status(200)
                .header("cache-control", "max-age=0")
                .header("content-type", "application/json")
                .body(DEFAULT_RESPONSE);
        });

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(x.verify("kms.@#", ""), Ok(true));
    }

    #[test]
    fn test_filters() {
        let response = r#"{ "regexes": {"keywords": [{"regex": "^kms$", "category":"suicide", "confidence":"high"}, {"regex":"^a4a$", "category":"eating", "confidence":"high"}, {"regex": "^suicidal$", "category":"suicide", "confidence":"medium"}], "preprocess": " "} }"#.to_string();
        let server = httpmock::MockServer::start();
        let _ = server.mock(|when, then| {
            when.path("/keywords");
            then.status(200)
                .header("cache-control", "max-age=0")
                .header("content-type", "application/json")
                .body(response);
        });

        let mut x = KokoKeywords::new(
            server.url("/keywords"),
            KeywordsCache::new(DEFAULT_RESPONSE.to_string(), Instant::now()),
        );

        assert_eq!(x.verify("kms", "category=suicide"), Ok(true));
        assert_eq!(x.verify("kms", "category=eating"), Ok(false));
        assert_eq!(
            x.verify("kms", "category=suicide:confidence=medium"),
            Ok(false)
        );
        assert_eq!(x.verify("kms", "category=suicide:confidence=high"), Ok(true));
        assert_eq!(
            x.verify("suicidal", "category=suicide:confidence=high"),
            Ok(false)
        );
    }
}
