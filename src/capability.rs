use crate::RESOURCE_PREFIX;
use cid::Cid;
use std::collections::BTreeMap;

use crate::ability::{Ability, AbilityName, AbilityNamespace};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};

use iri_string::types::UriString;
use siwe::Message;

pub type NotaBene<T> = Vec<BTreeMap<String, T>>;
pub type Attenuations<NB> = BTreeMap<UriString, BTreeMap<Ability, NotaBene<NB>>>;

/// Representation of a set of delegated Capabilities.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capability<NB = Value>
where
    NB: for<'d> Deserialize<'d> + Serialize,
{
    /// The actions that are allowed for the given target within this namespace.
    #[serde(rename = "att")]
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    attenuations: BTreeMap<UriString, BTreeMap<Ability, NotaBene<NB>>>,

    /// Cids of parent delegations which these capabilities are attenuated from
    #[serde(rename = "prf")]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    proof: Vec<Cid>,
}

#[derive(thiserror::Error, Debug)]
pub enum ConvertError<T, A> {
    #[error("Invalid Target: {0}")]
    InvalidTarget(T),
    #[error("Invalid Action: {0}")]
    InvalidAction(A),
}

impl<NB> Capability<NB>
where
    NB: for<'d> Deserialize<'d> + Serialize,
{
    /// Create a new empty Capability.
    pub fn new() -> Self {
        Self {
            attenuations: Default::default(),
            proof: Default::default(),
        }
    }

    /// Check if a particular action is allowed for the specified target, or is allowed globally.
    pub fn can<T, A>(
        &self,
        target: T,
        action: A,
    ) -> Result<Option<impl Iterator<Item = &BTreeMap<String, NB>>>, ConvertError<T::Error, A::Error>>
    where
        T: TryInto<UriString>,
        A: TryInto<Ability>,
    {
        Ok(self.can_do(
            &target.try_into().map_err(ConvertError::InvalidTarget)?,
            &action.try_into().map_err(ConvertError::InvalidAction)?,
        ))
    }

    /// Check if a particular action is allowed for the specified target, or is allowed globally, without type conversion.
    pub fn can_do<'l>(
        &'l self,
        target: &UriString,
        action: &Ability,
    ) -> Option<impl Iterator<Item = &'l BTreeMap<String, NB>>> {
        self.attenuations
            .get(target)
            .and_then(|m| m.get(action))
            .map(|v| v.iter())
    }

    /// Merge this Capabilities set with another
    pub fn merge(mut self, other: Self) -> Self {
        for proof in &other.proof {
            if self.proof.contains(proof) {
                continue;
            }
            self.proof.push(*proof);
        }

        for (uri, abs) in other.attenuations.into_iter() {
            let res_entry = self.attenuations.entry(uri).or_default();
            for (ab, nbs) in abs.into_iter() {
                res_entry.entry(ab).or_default().extend(nbs);
            }
        }
        self
    }

    /// Add an allowed action for the given target, with a set of note-benes
    pub fn with_action(
        mut self,
        target: UriString,
        action: Ability,
        nb: impl IntoIterator<Item = BTreeMap<String, NB>>,
    ) -> Self {
        self.attenuations
            .entry(target)
            .or_default()
            .entry(action)
            .or_default()
            .extend(nb);
        self
    }

    /// Add an allowed action for the given target, with a set of note-benes.
    ///
    /// This method automatically converts the provided args into the correct types for convenience.
    pub fn with_action_convert<T, A>(
        self,
        target: T,
        action: A,
        nb: impl IntoIterator<Item = BTreeMap<String, NB>>,
    ) -> Result<Self, ConvertError<T::Error, A::Error>>
    where
        T: TryInto<UriString>,
        A: TryInto<Ability>,
    {
        Ok(self.with_action(
            target.try_into().map_err(ConvertError::InvalidTarget)?,
            action.try_into().map_err(ConvertError::InvalidAction)?,
            nb,
        ))
    }

    /// Add a set of allowed action for the given target, with associated note-benes
    pub fn with_actions(
        mut self,
        target: UriString,
        abilities: impl IntoIterator<Item = (Ability, impl IntoIterator<Item = BTreeMap<String, NB>>)>,
    ) -> Self {
        let entry = self.attenuations.entry(target).or_default();
        for (ability, nbs) in abilities {
            let ab_entry = entry.entry(ability).or_default();
            ab_entry.extend(nbs);
        }
        self
    }

    /// Add a set of allowed action for the given target, with associated note-benes.
    ///
    /// This method automatically converts the provided args into the correct types for convenience.
    pub fn with_actions_convert<T, A, N>(
        self,
        target: T,
        abilities: impl IntoIterator<Item = (A, N)>,
    ) -> Result<Self, ConvertError<T::Error, A::Error>>
    where
        T: TryInto<UriString>,
        A: TryInto<Ability>,
        N: IntoIterator<Item = BTreeMap<String, NB>>,
    {
        Ok(self.with_actions(
            target.try_into().map_err(ConvertError::InvalidTarget)?,
            abilities
                .into_iter()
                .map(|(a, n)| Ok((a.try_into()?, n)))
                .collect::<Result<Vec<(Ability, N)>, A::Error>>()
                .map_err(ConvertError::InvalidAction)?,
        ))
    }

    /// Read the set of abilities granted in this capabilities set
    pub fn abilities(&self) -> &BTreeMap<UriString, BTreeMap<Ability, NotaBene<NB>>> {
        &self.attenuations
    }

    /// Read the set of abilities granted for a given target in this capabilities set
    pub fn abilities_for<T>(
        &self,
        target: T,
    ) -> Result<Option<&BTreeMap<Ability, NotaBene<NB>>>, T::Error>
    where
        T: TryInto<UriString>,
    {
        Ok(self.attenuations.get(&target.try_into()?))
    }

    /// Read the set of proofs which support the granted capabilities
    pub fn proof(&self) -> &[Cid] {
        &self.proof
    }

    /// Add a supporting proof CID
    pub fn with_proof(mut self, proof: &Cid) -> Self {
        if self.proof.contains(proof) {
            return self;
        }
        self.proof.push(*proof);
        self
    }

    /// Add a set of supporting proofs
    pub fn with_proofs<'l>(mut self, proofs: impl IntoIterator<Item = &'l Cid>) -> Self {
        for proof in proofs {
            if self.proof.contains(proof) {
                continue;
            }
            self.proof.push(*proof);
        }
        self
    }

    fn to_line_groups(
        &self,
    ) -> impl Iterator<Item = (&UriString, &AbilityNamespace, Vec<&AbilityName>)> {
        self.attenuations.iter().flat_map(|(resource, abilities)| {
            // group abilities by namespace
            abilities
                .iter()
                .fold(
                    BTreeMap::<&AbilityNamespace, Vec<&AbilityName>>::new(),
                    |mut map, (ability, _)| {
                        map.entry(ability.namespace())
                            .or_default()
                            .push(ability.name());
                        map
                    },
                )
                .into_iter()
                .map(move |(namespace, names)| (resource, namespace, names))
        })
    }

    fn to_statement_lines(&self) -> impl Iterator<Item = String> + '_ {
        self.to_line_groups().map(|(resource, namespace, names)| {
            format!(
                "\"{}\": {} for \"{}\".",
                namespace,
                names
                    .iter()
                    .map(|an| format!("\"{an}\""))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource
            )
        })
    }

    pub fn into_inner(self) -> (Attenuations<NB>, Vec<Cid>) {
        (self.attenuations, self.proof)
    }

    /// Apply this capabilities set to a SIWE message by writing to it's statement and resource list
    pub fn build_message(&self, mut message: Message) -> Result<Message, EncodingError> {
        if self.attenuations.is_empty() {
            return Ok(message);
        }
        let statement = self.to_statement();
        let encoded: UriString = self.try_into()?;
        message.resources.push(encoded);
        let m = message.statement.unwrap_or_default();
        message.statement = Some(if m.is_empty() {
            statement
        } else {
            format!("{m} {statement}")
        });
        Ok(message)
    }

    /// Generate a ReCap statement from capabilities and URI (delegee).
    pub fn to_statement(&self) -> String {
        [
            "I further authorize the stated URI to perform the following actions on my behalf:"
                .to_string(),
            self.to_statement_lines()
                .enumerate()
                .map(|(n, line)| format!(" ({}) {line}", n + 1))
                .collect(),
        ]
        .concat()
    }

    fn extract(message: &Message) -> Result<Option<Self>, DecodingError> {
        message
            .resources
            .iter()
            .last()
            .filter(|u| u.as_str().starts_with(RESOURCE_PREFIX))
            .map(Self::try_from)
            .transpose()
    }

    /// Extract the encoded capabilities from a SIWE message and ensures the correctness of the statement.
    pub fn extract_and_verify(message: &Message) -> Result<Option<Self>, VerificationError> {
        if let Some(c) = Self::extract(message)? {
            let expected = c.to_statement();
            match &message.statement {
                Some(s) if s.ends_with(&expected) => Ok(Some(c)),
                _ => Err(VerificationError::IncorrectStatement(expected)),
            }
        } else {
            // no caps
            Ok(None)
        }
    }

    fn encode(&self) -> Result<String, EncodingError> {
        serde_json::to_vec(self)
            .map_err(EncodingError::Ser)
            .map(|bytes| base64::encode_config(bytes, base64::URL_SAFE_NO_PAD))
    }

    fn decode(encoded: &str) -> Result<Self, DecodingError> {
        base64::decode_config(encoded, base64::URL_SAFE_NO_PAD)
            .map_err(DecodingError::Base64Decode)
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(DecodingError::De))
    }
}

impl<NB> Default for Capability<NB>
where
    NB: for<'d> Deserialize<'d> + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<NB> TryFrom<&UriString> for Capability<NB>
where
    NB: for<'d> Deserialize<'d> + Serialize,
{
    type Error = DecodingError;
    fn try_from(uri: &UriString) -> Result<Self, Self::Error> {
        uri.as_str()
            .strip_prefix(RESOURCE_PREFIX)
            .ok_or_else(|| DecodingError::InvalidResourcePrefix(uri.to_string()))
            .and_then(Capability::decode)
    }
}

impl<NB> TryFrom<&Capability<NB>> for UriString
where
    NB: for<'d> Deserialize<'d> + Serialize,
{
    type Error = EncodingError;
    fn try_from(cap: &Capability<NB>) -> Result<Self, Self::Error> {
        cap.encode()
            .map(|encoded| format!("{RESOURCE_PREFIX}{encoded}"))
            .and_then(|s| s.parse().map_err(EncodingError::UriParse))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DecodingError {
    #[error(
        "invalid resource prefix (expected prefix: {}, found: {0})",
        RESOURCE_PREFIX
    )]
    InvalidResourcePrefix(String),
    #[error("failed to decode base64 capability resource: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("failed to deserialize capability from json: {0}")]
    De(#[from] serde_json::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum EncodingError {
    #[error("unable to parse capability as a URI: {0}")]
    UriParse(#[from] iri_string::validate::Error),
    #[error("failed to serialize capability to json: {0}")]
    Ser(#[from] serde_json::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum VerificationError {
    #[error("error decoding capabilities: {0}")]
    Decoding(#[from] DecodingError),
    #[error("incorrect statement in siwe message, expected to end with: {0}")]
    IncorrectStatement(String),
}

#[cfg(test)]
mod test {
    use super::*;

    const JSON_CAP: &str = include_str!("../tests/serialized_cap.json");

    #[test]
    fn deser() {
        let cap: Capability = serde_json::from_str(JSON_CAP).unwrap();
        let reser = serde_json::to_string(&cap).unwrap();
        assert_eq!(JSON_CAP.trim(), reser);
    }
}
