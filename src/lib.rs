mod builder;
mod capability;
mod error;
mod namespace;
mod set;

pub use builder::{extract_capabilities, verify_statement, Builder};
pub use capability::Capability;
pub use error::Error;
pub use namespace::Namespace;
pub use set::Set;

pub const RESOURCE_PREFIX: &str = "urn:capability:";

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
        let msg = Builder::new()
            .build(Message {
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
        let credential: Namespace = "credential".parse().unwrap();

        let msg = Builder::new()
            .with_default_actions(&credential, ["present"])
            .build(Message {
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
        let msg = Builder::new()
            .build(Message {
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
        let credential: Namespace = "credential".parse().unwrap();
        let kepler: Namespace = "kepler".parse().unwrap();

        let msg = Builder::new()
            .with_default_actions(&credential, ["present"])
            .with_actions(&credential, "type:type1", ["present"])
            .with_actions(
                &kepler,
                "kepler:ens:example.eth://default/kv",
                ["list", "get", "metadata"],
            )
            .with_actions(
                &kepler,
                "kepler:ens:example.eth://default/kv/public",
                ["list", "get", "metadata", "put", "delete"],
            )
            .with_actions(
                &kepler,
                "kepler:ens:example.eth://default/kv/dapp-space",
                ["list", "get", "metadata", "put", "delete"],
            )
            .build(Message {
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
