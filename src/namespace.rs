use crate::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
/// The namespace that a capability is valid for.
pub struct Namespace(String);

impl From<Namespace> for String {
    fn from(from: Namespace) -> String {
        from.0
    }
}

impl AsRef<str> for Namespace {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for Namespace {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut previous_char_was_alphanum = false;
        for c in s.chars() {
            if c.is_ascii_alphanumeric() {
                previous_char_was_alphanum = true;
                continue;
            }
            if c == '-' && previous_char_was_alphanum {
                previous_char_was_alphanum = false;
                continue;
            }
            if c == '-' {
                return Err(Error::InvalidNamespaceHyphens);
            }
            return Err(Error::InvalidNamespaceChars);
        }
        if !previous_char_was_alphanum {
            return Err(Error::InvalidNamespaceHyphens);
        }
        Ok(Namespace(s.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invalid_namespace() {
        "https://example.com/".parse::<Namespace>().unwrap_err();
    }

    #[test]
    fn valid_namespace() {
        "my-namespace".parse::<Namespace>().unwrap();
    }
}
