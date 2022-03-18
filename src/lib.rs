use cache_control::CacheControl;
use std::{ffi::CStr, sync::Mutex, env, collections::HashMap, time::SystemTime, ops, fmt};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::time::Duration;
use ureq::{Error, ErrorKind};

const URL: &str = "api.kokocares.org/keywords";
const CACHE_EXPIRATION_DEFAULT: Duration = Duration::from_secs(3600);

#[derive(Clone, Debug)]
pub struct Regex(regex::Regex);

impl ops::Deref for Regex {
    type Target = regex::Regex;
    fn deref(&self) -> &regex::Regex { &self.0 }
}

impl<'de> serde::Deserialize<'de> for Regex {
    fn deserialize<D>(de: D) -> Result<Regex, D::Error>
    where D: serde::Deserializer<'de>
    {
        use serde::de::{Error, Visitor};

        struct RegexVisitor;

        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = Regex;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a regular expression pattern")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Regex, E> {
                regex::Regex::new(v).map(Regex).map_err(|err| {
                    E::custom(err.to_string())
                })
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
    CacheRefreshError = -3,
    ParseError = -4,
    InvalidUrl = -5,
}

#[derive(Deserialize, Debug)]
struct Keywords {
    pub keywords: Vec<Regex>,
    pub preprocess: Regex,
}

struct KeywordsCache {
    pub expires_at: SystemTime,
    pub keywords: Keywords,
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
    pub fn new(url: String) -> Self {
        Self {
            keywords: HashMap::new(),
            url,
        }
    }

    pub fn verify(
        &mut self,
        keyword: &str,
        filter: &str,
        version: Option<&str>,
    ) -> KokoResult<bool> {
        let cache_key = format!("{}_{}", filter, version.unwrap_or("latest"));

        if let Some(keyword_cache) = self.keywords.get(&cache_key) {
            if SystemTime::now() < keyword_cache.expires_at {
                let keyword = keyword_cache.keywords.preprocess.replace_all(keyword, "").to_lowercase();

                for re_keyword in &keyword_cache.keywords.keywords {
                    if re_keyword.is_match(&keyword) {
                        return Ok(true);
                    }
                }

                return Ok(false);
            } else {
                self.load_cache(filter, version)?;
                self.verify(keyword, filter, version)
            }
        } else {
            self.load_cache(filter, version)?;
            self.verify(keyword, filter, version)
        }
    }

    pub fn load_cache(&mut self, filter: &str, version: Option<&str>) -> KokoResult<()> {
        let cache_key = format!("{}_{}", filter, version.unwrap_or("latest"));

        eprintln!("[koko-keywords] Loading cache for '{}'", cache_key);

        let request = ureq::get(&self.url);

        let request = request.query("filter", filter);
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
                    Err(KokoError::CacheRefreshError)
                }
            },
            Err(Error::Status(403, _)) => Err(KokoError::InvalidCredentials),
            Err(response) => {
                eprintln!("{:?}", response);
                Err(KokoError::CacheRefreshError)
            },
        }?;

        let expires_in = response
            .header("cache-control")
            .map(CacheControl::from_value)
            .flatten()
            .map(|cc| cc.max_age)
            .flatten()
            .unwrap_or(CACHE_EXPIRATION_DEFAULT);

        let api_response: ApiResponse =
            match serde_json::from_reader(response.into_reader()) {
                Ok(response) => Ok(response),
                Err(response) => {
                    eprintln!("{:?}", response);
                    Err(KokoError::ParseError)
                },
            }?;

        let keywords_cache = KeywordsCache {
            keywords: api_response.regex,
            expires_at: SystemTime::now() + expires_in,
        };
        self.keywords.insert(cache_key.to_string(), keywords_cache);

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
    input: *const std::os::raw::c_char ,
    filter: *const std::os::raw::c_char ,
    version: *const std::os::raw::c_char ,
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

    #[test]
    fn test_empty_cache() {
        let mut x = KokoKeywords {
            keywords: HashMap::new(),
            url: "http://localhost".to_string(),
        };

        assert_eq!(
            x.verify("hello", "", None),
            Err(KokoError::CacheRefreshError)
        );
    }

    #[test]
    fn test_invalid_url() {
        let mut x = KokoKeywords {
            keywords: HashMap::new(),
            url: "".to_string(),
        };

        assert_eq!(
            x.verify("hello", "", None),
            Err(KokoError::InvalidUrl)
        );
    }

    #[test]
    fn test_unexpired_cache() {
        let api_response: ApiResponse =
            serde_json::from_str("{ \"regex\": {\"keywords\": [\"^badword$\"], \"preprocess\": \" \"} }").unwrap();
        let mut x = KokoKeywords {
            keywords: HashMap::from([(
                "_latest".to_string(),
                KeywordsCache {
                    keywords: api_response.regex,
                    expires_at: SystemTime::now() + Duration::new(1000, 0),
                },
            )]),
            url: "http://localhost".to_string(),
        };

        assert_eq!(
            x.verify("hello", "", None),
            Ok(false)
        );

        assert_eq!(
            x.verify("badword", "", None),
            Ok(true)
        );

        assert_eq!(
            x.verify(" badword   ", "", None),
            Ok(true)
        );
    }

    #[test]
    fn test_case_insensitive() {
        let api_response: ApiResponse =
            serde_json::from_str("{ \"regex\": {\"keywords\": [\"^badword$\"], \"preprocess\": \" \"} }").unwrap();
        let mut x = KokoKeywords {
            keywords: HashMap::from([(
                "_latest".to_string(),
                KeywordsCache {
                    keywords: api_response.regex,
                    expires_at: SystemTime::now() + Duration::new(1000, 0),
                },
            )]),
            url: "http://localhost".to_string(),
        };

        assert_eq!(
            x.verify("Badword", "", None),
            Ok(true)
        );
    }
}
