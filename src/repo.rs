use once_cell::sync::Lazy as LazyLock;

use regex::Regex;

static REPO_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
            (?P<registry>[0-9]+)\.
            dkr\.
            ecr\.
            ([a-z0-9-]+)\.
            amazonaws\.com/
            (?P<repository>.+)",
    )
    .unwrap()
});

pub fn extract(image: &str) -> Option<(String, String)> {
    REPO_PATTERN.captures(image).map(|cap| {
        (
            cap.name("registry").map(|r| r.as_str()).unwrap().to_owned(),
            cap.name("repository")
                .map(|r| r.as_str())
                .unwrap()
                .to_owned(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_ecr_url() {
        let (registry, name) = extract("123456789.dkr.ecr.us-east-1.amazonaws.com/a/b").unwrap();
        assert_eq!(registry, "123456789");
        assert_eq!(name, "a/b");
    }
}
