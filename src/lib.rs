mod ability;
mod capability;
mod error;
mod translation;

pub use ability::{Ability, AbilityName, AbilityNamespace};
pub use capability::{Capability, ConvertError};
pub use error::Error;
pub use translation::{capabilities_to_statement, extract_capabilities};

use siwe::Message;

/// The prefix for a ReCap uri.
pub const RESOURCE_PREFIX: &str = "urn:recap:";

/// Verifies a ReCap statement.
///
/// Checks that the encoded delegations match the human-readable description in the statement, and
/// that the URI displayed in the statement matches the uri field.
pub fn verify_statement(message: &Message) -> Result<bool, Error> {
    let capabilities = extract_capabilities(message)?;
    let generated_statement = capabilities.map(|c| capabilities_to_statement(&c, &message.uri));
    let verified = match (&message.statement, &generated_statement) {
        (None, None) => true,
        (Some(o), Some(a)) => o.ends_with(a),
        _ => false,
    };
    Ok(verified)
}

#[cfg(test)]
mod test {
    use super::*;
    use siwe::Message;

    const SIWE_WITH_INTERLEAVED_RES: &'static str =
        include_str!("../tests/siwe_with_interleaved_resources.txt");
    const SIWE_WITH_STATEMENT_NO_CAPS: &'static str =
        include_str!("../tests/siwe_with_statement_no_caps.txt");
    const SIWE_WITH_STATEMENT: &'static str = include_str!("../tests/siwe_with_statement.txt");
    const SIWE_NO_CAPS: &'static str = include_str!("../tests/siwe_with_no_caps.txt");
    const SIWE: &'static str = include_str!("../tests/siwe_with_caps.txt");

    #[test]
    fn no_caps_statement_append() {
        let msg = Capability::default()
            .build_message(Message {
                domain: "example.com".parse().unwrap(),
                address: Default::default(),
                statement: Some("Some custom statement.".into()),
                uri: "did:key:example".parse().unwrap(),
                version: siwe::Version::V1,
                chain_id: 1,
                nonce: "mynonce1".into(),
                issued_at: "2022-06-21T12:00:00.000Z".parse().unwrap(),
                expiration_time: None,
                not_before: None,
                request_id: None,
                resources: vec![],
            })
            .expect("failed to build SIWE delegation");

        assert_eq!(
            SIWE_WITH_STATEMENT_NO_CAPS,
            msg.to_string(),
            "generated SIWE message did not match expectation"
        );
    }

    #[test]
    fn build_delegation_statement_append() {
        let msg = Capability::default()
            .with_action_convert("credential:*", "credential/present", [])
            .unwrap()
            .build_message(Message {
                domain: "example.com".parse().unwrap(),
                address: Default::default(),
                statement: Some("Some custom statement.".into()),
                uri: "did:key:example".parse().unwrap(),
                version: siwe::Version::V1,
                chain_id: 1,
                nonce: "mynonce1".into(),
                issued_at: "2022-06-21T12:00:00.000Z".parse().unwrap(),
                expiration_time: None,
                not_before: None,
                request_id: None,
                resources: vec!["http://example.com".parse().unwrap()],
            })
            .expect("failed to build SIWE delegation");

        assert_eq!(
            SIWE_WITH_STATEMENT,
            msg.to_string(),
            "generated SIWE message did not match expectation"
        );
    }

    #[test]
    fn no_caps() {
        let msg = Capability::default()
            .build_message(Message {
                domain: "example.com".parse().unwrap(),
                address: Default::default(),
                statement: None,
                uri: "did:key:example".parse().unwrap(),
                version: siwe::Version::V1,
                chain_id: 1,
                nonce: "mynonce1".into(),
                issued_at: "2022-06-21T12:00:00.000Z".parse().unwrap(),
                expiration_time: None,
                not_before: None,
                request_id: None,
                resources: vec![],
            })
            .expect("failed to build SIWE delegation");

        assert_eq!(
            SIWE_NO_CAPS,
            msg.to_string(),
            "generated SIWE message did not match expectation"
        );
    }

    #[test]
    fn build_delegation() {
        let msg = Capability::default()
            .with_action_convert("urn:credential:type:type1", "credential/present", [])
            .unwrap()
            .with_action_convert("kepler:ens:example.eth://default/kv", "kv/list", [])
            .unwrap()
            .with_action_convert("kepler:ens:example.eth://default/kv", "kv/get", [])
            .unwrap()
            .with_action_convert("kepler:ens:example.eth://default/kv", "kv/metadata", [])
            .unwrap()
            .with_action_convert("kepler:ens:example.eth://default/kv/public", "kv/list", [])
            .unwrap()
            .with_action_convert("kepler:ens:example.eth://default/kv/public", "kv/get", [])
            .unwrap()
            .with_action_convert(
                "kepler:ens:example.eth://default/kv/public",
                "kv/metadata",
                [],
            )
            .unwrap()
            .with_action_convert("kepler:ens:example.eth://default/kv/public", "kv/put", [])
            .unwrap()
            .with_action_convert(
                "kepler:ens:example.eth://default/kv/public",
                "kv/delete",
                [],
            )
            .unwrap()
            .with_action_convert(
                "kepler:ens:example.eth://default/kv/dapp-space",
                "kv/list",
                [],
            )
            .unwrap()
            .with_action_convert(
                "kepler:ens:example.eth://default/kv/dapp-space",
                "kv/get",
                [],
            )
            .unwrap()
            .with_action_convert(
                "kepler:ens:example.eth://default/kv/dapp-space",
                "kv/metadata",
                [],
            )
            .unwrap()
            .with_action_convert(
                "kepler:ens:example.eth://default/kv/dapp-space",
                "kv/put",
                [],
            )
            .unwrap()
            .with_action_convert(
                "kepler:ens:example.eth://default/kv/dapp-space",
                "kv/delete",
                [],
            )
            .unwrap()
            .build_message(Message {
                domain: "example.com".parse().unwrap(),
                address: Default::default(),
                statement: None,
                uri: "did:key:example".parse().unwrap(),
                version: siwe::Version::V1,
                chain_id: 1,
                nonce: "mynonce1".into(),
                issued_at: "2022-06-21T12:00:00.000Z".parse().unwrap(),
                expiration_time: None,
                not_before: None,
                request_id: None,
                resources: vec![],
            })
            .expect("failed to build SIWE delegation");

        assert_eq!(
            SIWE,
            msg.to_string(),
            "generated SIWE message did not match expectation"
        );
    }

    #[test]
    fn verify() {
        let msg: Message = SIWE.parse().unwrap();
        assert!(
            verify_statement(&msg).expect("unable to parse resources as capabilities"),
            "statement did not match capabilities"
        );

        let mut altered_msg_1 = msg.clone();
        altered_msg_1
            .statement
            .iter_mut()
            .for_each(|statement| statement.push_str(" I am the walrus!"));
        assert!(
            !verify_statement(&altered_msg_1).expect("unable to parse resources as capabilities"),
            "altered statement incorrectly matched capabilities"
        );

        let mut altered_msg_2 = msg.clone();
        altered_msg_2.uri = "did:key:altered".parse().unwrap();
        assert!(
            !verify_statement(&altered_msg_2).expect("unable to parse resources as capabilities"),
            "altered uri incorrectly matched capabilities"
        );
    }

    #[test]
    fn verify_interleaved_resources() {
        let msg: Message = SIWE_WITH_INTERLEAVED_RES.parse().unwrap();
        assert!(
            verify_statement(&msg).expect("unable to parse resources as capabilities"),
            "statement did not match capabilities"
        );
    }
}
