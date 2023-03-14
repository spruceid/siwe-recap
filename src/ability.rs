use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    cmp::Ordering,
    fmt::{Display, Error as FmtError, Formatter},
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AbilityNamespace(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AbilityName(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone, SerializeDisplay, DeserializeFromStr)]
pub struct Ability(AbilityNamespace, AbilityName);

impl Ability {
    pub fn namespace(&self) -> &AbilityNamespace {
        &self.0
    }

    pub fn name(&self) -> &AbilityName {
        &self.1
    }
}

impl Ability {
    pub fn new(namespace: AbilityNamespace, name: AbilityName) -> Self {
        Self(namespace, name)
    }

    pub fn len(&self) -> usize {
        self.0.as_ref().len() + self.1.as_ref().len() + 1
    }

    pub fn is_empty(&self) -> bool {
        self.0.as_ref().is_empty() && self.1.as_ref().is_empty()
    }
}

impl PartialOrd for Ability {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ability {
    fn cmp(&self, other: &Self) -> Ordering {
        // if length is equal, compare by bytes, including '/' separator, without allocating
        let ar_self = [self.0.as_ref(), "/", self.1.as_ref()];
        let ar_other = [other.0.as_ref(), "/", other.1.as_ref()];

        let iter_self = ar_self.iter().flat_map(|s| s.as_bytes());
        let iter_other = ar_other.iter().flat_map(|s| s.as_bytes());

        for (a, b) in iter_self.zip(iter_other) {
            match a.cmp(b) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }

        self.len().cmp(&other.len())
    }
}

impl Display for AbilityNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", &self.0)
    }
}

impl Display for AbilityName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", &self.0)
    }
}

impl Display for Ability {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}/{}", &self.0, &self.1)
    }
}

impl AsRef<str> for AbilityNamespace {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AbilityName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AbilityParseError {
    #[error("Invalid Characters: {0}")]
    InvalidCharacter(String),
}

const ALLOWED_CHARS: &str = "-_.+*";

fn not_allowed(c: char) -> bool {
    !c.is_alphanumeric() && !ALLOWED_CHARS.contains(c)
}

impl FromStr for AbilityNamespace {
    type Err = AbilityParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.contains(not_allowed) {
            Err(AbilityParseError::InvalidCharacter(s.into()))
        } else {
            Ok(Self(s.into()))
        }
    }
}

impl FromStr for AbilityName {
    type Err = AbilityParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.contains(not_allowed) {
            Err(AbilityParseError::InvalidCharacter(s.into()))
        } else {
            Ok(Self(s.into()))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ActionParseError {
    #[error("Missing '/' separator")]
    MissingSeparator,
    #[error(transparent)]
    Namespace(<AbilityNamespace as FromStr>::Err),
    #[error(transparent)]
    Name(<AbilityName as FromStr>::Err),
}

impl FromStr for Ability {
    type Err = ActionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once('/')
            .ok_or(ActionParseError::MissingSeparator)
            .and_then(|(ns, name)| {
                Ok(Self(
                    ns.parse().map_err(ActionParseError::Namespace)?,
                    name.parse().map_err(ActionParseError::Name)?,
                ))
            })
    }
}

impl TryFrom<&str> for AbilityNamespace {
    type Error = <Self as FromStr>::Err;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl TryFrom<&str> for AbilityName {
    type Error = <Self as FromStr>::Err;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl TryFrom<&str> for Ability {
    type Error = <Self as FromStr>::Err;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invalid_namespace() {
        for s in [
            "https://example.com/",
            "-my-namespace:",
            "my-namespace-/",
            "my--namespace[]",
            "not a valid namespace",
            "",
        ] {
            s.parse::<AbilityNamespace>().unwrap_err();
        }
    }

    #[test]
    fn valid_namespace() {
        for s in ["my-namespace", "My-nAmespac3-2"] {
            s.parse::<AbilityNamespace>().unwrap();
        }
    }

    #[test]
    fn valid_abilities() {
        for s in [
            "credential/present",
            "kv/list",
            "some-ns/some-name",
            "msg/*",
        ] {
            s.parse::<Ability>().unwrap();
        }
    }

    #[test]
    fn invalid_abilities() {
        for s in [
            "credential ns/present",
            "kv-list",
            "some:ns/some-name",
            "msg/wrong/str",
            "/",
        ] {
            s.parse::<Ability>().unwrap_err();
        }
    }

    #[test]
    fn ordering() {
        let ab0: Ability = "a/b".parse().unwrap();
        let ab1: Ability = "a/c".parse().unwrap();
        let ab2: Ability = "aa/a".parse().unwrap();
        let ab3: Ability = "b/a".parse().unwrap();
        let ab4: Ability = "kv*/read".parse().unwrap();
        let ab5: Ability = "kv/list".parse().unwrap();
        let ab6: Ability = "kv/read".parse().unwrap();
        let ab7: Ability = "kva/get".parse().unwrap();

        assert!(ab0 < ab1, "abilities are sorted by byte value");
        assert!(ab1 < ab2, "abilities are sorted by byte value");
        assert!(ab2 < ab3, "abilities are sorted by byte value");
        assert!(ab3 < ab4, "abilities are sorted by byte value");
        assert!(ab4 < ab5, "* is sorted before /");
        assert!(ab5 < ab6, "abilities are sorted by byte value");
        assert!(ab6 < ab7, "abilities are sorted by byte value");
    }
}
