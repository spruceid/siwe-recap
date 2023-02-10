use crate::{capabilities_to_statement, translation::ToResource, Error};
use cid::Cid;
use std::collections::{BTreeMap, BTreeSet};

use crate::ability::{Ability, AbilityName, AbilityNamespace};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};

use iri_string::types::UriString;
use siwe::Message;

pub type NoteBenes<T> = Vec<BTreeMap<String, T>>;
pub type Attenuations<NB> = BTreeMap<UriString, BTreeMap<Ability, NoteBenes<NB>>>;

/// Representation of a set of delegated Capabilities.
#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Capability<NB = Value>
where
    NB: for<'d> Deserialize<'d> + Serialize,
{
    /// The actions that are allowed for the given target within this namespace.
    #[serde(rename = "att")]
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    attenuations: BTreeMap<UriString, BTreeMap<Ability, NoteBenes<NB>>>,

    /// Cids of parent delegations which these capabilities are attenuated from
    #[serde(rename = "prf")]
    #[serde_as(as = "BTreeSet<DisplayFromStr>")]
    proof: BTreeSet<Cid>,
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

    pub(crate) fn encode(&self) -> Result<String, Error> {
        serde_json::to_vec(self)
            .map_err(Error::Ser)
            .map(|bytes| base64::encode_config(bytes, base64::URL_SAFE_NO_PAD))
    }

    pub(crate) fn decode(encoded: &str) -> Result<Self, Error> {
        base64::decode_config(encoded, base64::URL_SAFE_NO_PAD)
            .map_err(Error::Base64Decode)
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(Error::De))
    }

    /// Merge this Capabilities set with another
    pub fn merge(mut self, other: Self) -> Self {
        self.proof.extend(other.proof);
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
    pub fn abilities(&self) -> &BTreeMap<UriString, BTreeMap<Ability, NoteBenes<NB>>> {
        &self.attenuations
    }

    /// Read the set of abilities granted for a given target in this capabilities set
    pub fn abilities_for<T>(
        &self,
        target: T,
    ) -> Result<Option<&BTreeMap<Ability, NoteBenes<NB>>>, T::Error>
    where
        T: TryInto<UriString>,
    {
        Ok(self.attenuations.get(&target.try_into()?))
    }

    /// Read the set of proofs which support the granted capabilities
    pub fn proof(&self) -> &BTreeSet<Cid> {
        &self.proof
    }

    /// Add a supporting proof CID
    pub fn with_proof(mut self, proof: &Cid) -> Self {
        self.proof.insert(*proof);
        self
    }

    /// Add a set of supporting proofs
    pub fn with_proofs<'l>(mut self, proofs: impl IntoIterator<Item = &'l Cid>) -> Self {
        self.proof.extend(proofs);
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

    pub(crate) fn to_statement_lines(&self) -> impl Iterator<Item = String> + '_ {
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

    pub fn into_inner(self) -> (Attenuations<NB>, BTreeSet<Cid>) {
        (self.attenuations, self.proof)
    }

    /// Apply this capabilities set to a SIWE message by writing to it's statement and resource list
    pub fn build_message(&self, mut message: Message) -> Result<Message, Error> {
        if self.attenuations.is_empty() {
            return Ok(message);
        }
        let statement = capabilities_to_statement(self, &message.uri);
        let encoded = self.to_resource()?;
        message.resources.push(encoded);
        let m = message.statement.unwrap_or_default();
        message.statement = Some(if m.is_empty() {
            statement
        } else {
            format!("{m} {statement}")
        });
        Ok(message)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deser() {
        let ser = r#"{"att":{"http://example.com/public/photos/":{"crud/delete":[]},"mailto:username@example.com":{"msg/receive":[{"max_count":5,"templates":["newsletter","marketing"]}],"msg/send":[{"to":"someone@email.com"},{"to":"joe@email.com"}]}},"prf":["bafysameboaza4mnsng7t3djdbilbrnliv6ikxh45zsph7kpettjfbp4ad2g2uu2znujlf2afphw25d4y35pq"]}"#;
        let cap: Capability = serde_json::from_str(ser).unwrap();
        let reser = serde_json::to_string(&cap).unwrap();
        assert_eq!(ser, reser);
    }
}
