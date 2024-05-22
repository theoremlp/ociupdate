use std::fmt::Display;

use once_cell::sync::Lazy as LazyLock;

use regex::{Captures, Regex};

static VERSION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
            (?P<major>[0-9]+)\.
            (?P<minor>[0-9]+)\.
            (?P<patch>[0-9]+)
            (-(?P<distance>[0-9]+)-g(?P<commitish>[A-Za-z0-9]+))?",
    )
    .unwrap()
});

#[derive(Debug)]
pub enum Error {
    InvalidVersionString,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GitDescribeVersion {
    major: u32,
    minor: u32,
    patch: u32,
    distance: u32,
    commitish: Option<String>,
}

fn extract(cap: &Captures, item: &str) -> Option<u32> {
    cap.name(item).map(|m| m.as_str().parse::<u32>().unwrap())
}

impl Display for GitDescribeVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.distance == 0 {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            write!(
                f,
                "{}.{}.{}-{}{}",
                self.major,
                self.minor,
                self.patch,
                self.distance,
                self.commitish
                    .as_ref()
                    .map(|c| "-g".to_owned() + c)
                    .unwrap_or("".to_owned())
            )
        }
    }
}

impl GitDescribeVersion {
    pub fn from_str(version: &str) -> Result<GitDescribeVersion, Error> {
        VERSION_PATTERN
            .captures(version)
            .map(|cap| {
                let major = extract(&cap, "major").unwrap();
                let minor = extract(&cap, "minor").unwrap();
                let patch = extract(&cap, "patch").unwrap();
                let distance = extract(&cap, "distance").unwrap_or(0);
                let commitish = cap
                    .name("commitish")
                    .map(|commitish| commitish.as_str().to_owned());
                Ok(GitDescribeVersion {
                    major,
                    minor,
                    patch,
                    distance,
                    commitish,
                })
            })
            .unwrap_or(Err(Error::InvalidVersionString))
    }

    pub fn is_release(&self) -> bool {
        self.distance == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tag() {
        let a = GitDescribeVersion::from_str("1.2.3").unwrap();
        assert_eq!(a.major, 1);
        assert_eq!(a.minor, 2);
        assert_eq!(a.patch, 3);
        assert_eq!(a.distance, 0);
        assert_eq!(a.commitish, None);
    }

    #[test]
    fn parse_snapshot() {
        let a = GitDescribeVersion::from_str("1.2.3-4-gabc123").unwrap();
        assert_eq!(a.major, 1);
        assert_eq!(a.minor, 2);
        assert_eq!(a.patch, 3);
        assert_eq!(a.distance, 4);
        assert_eq!(a.commitish.unwrap(), "abc123");
    }

    #[test]
    fn check_sort_order() {
        let a = GitDescribeVersion::from_str("1.2.3").unwrap();
        let b = GitDescribeVersion::from_str("1.2.4").unwrap();
        let c = GitDescribeVersion::from_str("1.2.4-1-gabcdef").unwrap();
        let d = GitDescribeVersion::from_str("1.3.0").unwrap();

        let truth = vec![&a, &b, &c, &d];

        let mut v = vec![&c, &b, &d, &a];
        v.sort();

        assert_eq!(truth, v, "test derived sort is correct");
    }
}
