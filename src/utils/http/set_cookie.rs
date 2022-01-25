use std::collections::HashMap;

use hyper::header::{self, HeaderName, HeaderValue};

#[derive(Default, Clone)]
pub struct SetCookieOptions {
    pub http_only: bool,
    pub secure: bool,
    // expires: ,
    /// Seconds
    pub max_age: Option<i64>,
    pub domain: Option<String>,
}

impl SetCookieOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;

        self
    }

    #[allow(dead_code)]
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;

        self
    }

    pub fn max_age(mut self, max_age: i64) -> Self {
        self.max_age.replace(max_age);

        self
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain.replace(domain.into());

        self
    }
}

impl SetCookieOptions {}

#[derive(Default)]
pub struct SetCookie {
    inner: HashMap<String, (String, SetCookieOptions)>,
}

impl SetCookie {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        options: SetCookieOptions,
    ) -> Self {
        self.inner.insert(key.into(), (value.into(), options));

        self
    }

    #[allow(dead_code)]
    pub fn remove(mut self, key: impl Into<String>) -> Self {
        self.inner.remove(&key.into());

        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (HeaderName, HeaderValue)> + '_ {
        self.inner
            .iter()
            .map(|(key, (value, options))| fmt(key, value, options))
            .map(|st| (header::SET_COOKIE, st.parse().unwrap()))
    }
}

fn fmt(
    key: &str,
    value: &str,
    SetCookieOptions {
        domain,
        max_age,
        http_only,
        secure,
    }: &SetCookieOptions,
) -> String {
    let mut base = format!("{}={}", key, value);

    if let Some(domain) = domain {
        base = format!("{}; Domain={}", base, domain);
    }

    if let Some(max_age) = max_age {
        base = format!("{}; Max-Age={}", base, max_age);
    }

    if *http_only {
        base = format!("{}; HttpOnly", base);
    }

    if *secure {
        base = format!("{}; Secure", base);
    }

    base
}
