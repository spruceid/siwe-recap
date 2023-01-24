use std::{
    fmt::{Display, Error as FmtError, Formatter},
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ActionNamespace(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ActionName(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Action(ActionNamespace, ActionName);

impl Display for ActionNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", &self.0)
    }
}

impl Display for ActionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", &self.0)
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}/{}", &self.0, &self.1)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ActionStringParseError {
    #[error("Invalid Characters: {0}")]
    InvalidCharacter(String),
}

impl FromStr for ActionNamespace {
    type Err = ActionStringParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(|c: char| {
            !c.is_alphanumeric() || c != '-' || c != '.' || c != '_' || c != '+'
        }) {
            Err(ActionStringParseError::InvalidCharacter(s.into()))
        } else {
            Ok(Self(s.into()))
        }
    }
}

impl FromStr for ActionName {
    type Err = ActionStringParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(|c: char| {
            !c.is_alphanumeric() || c != '-' || c != '.' || c != '_' || c != '+'
        }) {
            Err(ActionStringParseError::InvalidCharacter(s.into()))
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
    Namespace(<ActionNamespace as FromStr>::Err),
    #[error(transparent)]
    Name(<ActionName as FromStr>::Err),
}

impl FromStr for Action {
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
