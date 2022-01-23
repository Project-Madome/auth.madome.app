use std::collections::HashMap;

use hyper::{Body, Request};

pub struct Cookie {
    inner: HashMap<String, String>,
}

impl Cookie {
    pub fn parse(cookie_str: &str) -> Self {
        let mut inner = HashMap::new();

        // madome_access_token=avchdef; madome_refresh_token=qwehkdfsjd
        for a in cookie_str.split(';') {
            let mut a = a.split('=');

            if let Some(key) = a.next() {
                let value = a.next().unwrap_or("");

                inner.insert(key.trim().to_string(), value.to_string());
            }
        }

        Self { inner }
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|st| st as &str)
    }

    pub fn take(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key)
    }
}

impl From<&Request<Body>> for Cookie {
    fn from(request: &Request<Body>) -> Self {
        let cookie_str = request
            .headers()
            .get("Cookie")
            .and_then(|a| a.to_str().ok())
            .unwrap_or("");

        Self::parse(cookie_str)
    }
}
