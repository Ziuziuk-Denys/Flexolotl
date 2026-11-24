use std::sync::Arc;

pub type Fetch = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;

const LOCAL: [&str; 2] = ["localhost", "127.0.0.1"];

pub struct Network {
    hosts: Vec<String>,
    fetch: Fetch,
}

impl Network {
    pub fn new(hosts: &[String], fetch: Fetch) -> Self {
        Self {
            hosts: hosts.to_vec(),
            fetch,
        }
    }

    pub fn get(&self, url: &str) -> Option<Option<String>> {
        self.may_visit(url).then(|| (self.fetch)(url))
    }

    fn may_visit(&self, url: &str) -> bool {
        let Some((scheme, rest)) = url.split_once("://") else {
            return false;
        };
        let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
        let name = host.split(':').next().unwrap_or_default();
        let is_safe = scheme == "https" || scheme == "http" && LOCAL.contains(&name);
        let is_plain = !url.contains(['@', '\\', ' ', '\t', '\r', '\n']);
        is_safe && is_plain && self.hosts.iter().any(|allowed| allowed == host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visits_only_the_hosts_it_asked_for() {
        let hosts = ["api.example.com".to_owned(), "localhost:8080".to_owned()];
        let network = Network::new(&hosts, Arc::new(|url| Some(url.to_owned())));
        for good in ["https://api.example.com/a?b=c", "http://localhost:8080/x"] {
            assert_eq!(network.get(good), Some(Some(good.to_owned())), "{good}");
        }
        for bad in [
            "http://api.example.com/",
            "https://api.example.com.evil.example/",
            "https://api.example.com@evil.example/",
            "https://evil.example/#api.example.com",
            "https://evil.example\\@api.example.com/",
            "http://localhost:9000/",
            "file:///etc/passwd",
            "api.example.com",
        ] {
            assert_eq!(network.get(bad), None, "{bad}");
        }
    }
}
