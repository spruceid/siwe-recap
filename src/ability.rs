use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    fmt::{Display, Error as FmtError, Formatter},
    str::FromStr,
};

#[derive(Debug, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AbilityNamespace(String);

#[derive(Debug, Eq, Hash, Clone, Serialize, Deserialize)]
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

#[derive(thiserror::Error, Debug)]
pub enum AbilityParseError {
    #[error("Invalid Characters: {0}")]
    InvalidCharacter(String),
}

impl FromStr for AbilityNamespace {
    type Err = AbilityParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(|c: char| {
            !c.is_alphanumeric() || c != '-' || c != '.' || c != '_' || c != '+' || c != '*'
        }) {
            Err(AbilityParseError::InvalidCharacter(s.into()))
        } else {
            Ok(Self(s.into()))
        }
    }
}

impl FromStr for AbilityName {
    type Err = AbilityParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(|c: char| {
            !c.is_alphanumeric() || c != '-' || c != '.' || c != '_' || c != '+' || c != '*'
        }) {
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

impl PartialEq<AbilityNamespace> for AbilityNamespace {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

impl PartialEq<AbilityName> for AbilityName {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invalid_namespace() {
        for s in [
            "https://example.com/",
            "-my-namespace",
            "my-namespace-",
            "my--namespace",
            "not a valid namespace",
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
}
