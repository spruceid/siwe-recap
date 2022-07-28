# Capgrok - capabilities for humans to read.

Use this crate to build wallet-signable messages with capability delegations. The generated message contains two representations of the capabilities: an unambiguous machine-readable representation, and a human-readable description. Of the two representations, the latter is deterministically generated from the former.

## Message formats

We currently support the following message formats:
* [EIP-4361](https://eips.ethereum.org/EIPS/eip-4361): Sign-In With Ethereum (SIWE)

## SIWE Examples

An example with:
- the capability to `present` any credential
- the capability to `present` credentials of type `type1` (technically redundant)
- the capability to `list`, `get` and retrieve `metadata` from the kepler location `kepler:ens:example.eth://default/kv`
- the capability to `list`, `get`, retrieve `metadata`, `put` and `delete` from the kepler locations `kepler:ens:example.eth://default/kv/public` and `kepler:ens:example.eth://default/kv/dapp-space`
```rust
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
    })?;
```

Which produces this SIWE message:
```
example.com wants you to sign in with your Ethereum account:
0x0000000000000000000000000000000000000000

I further authorize did:key:example to perform the following actions on my behalf: (1) credential: present for any. (2) credential: present for type:type1. (3) kepler: list, get, metadata for kepler:ens:example.eth://default/kv. (4) kepler: list, get, metadata, put, delete for kepler:ens:example.eth://default/kv/dapp-space, kepler:ens:example.eth://default/kv/public.

URI: did:key:example
Version: 1
Chain ID: 1
Nonce: mynonce1
Issued At: 2022-06-21T12:00:00.000Z
Resources:
- urn:capability:credential:eyJkZWZhdWx0QWN0aW9ucyI6WyJwcmVzZW50Il0sInRhcmdldGVkQWN0aW9ucyI6eyJ0eXBlOnR5cGUxIjpbInByZXNlbnQiXX19
- urn:capability:kepler:eyJ0YXJnZXRlZEFjdGlvbnMiOnsia2VwbGVyOmVuczpleGFtcGxlLmV0aDovL2RlZmF1bHQva3YiOlsibGlzdCIsImdldCIsIm1ldGFkYXRhIl0sImtlcGxlcjplbnM6ZXhhbXBsZS5ldGg6Ly9kZWZhdWx0L2t2L2RhcHAtc3BhY2UiOlsibGlzdCIsImdldCIsIm1ldGFkYXRhIiwicHV0IiwiZGVsZXRlIl0sImtlcGxlcjplbnM6ZXhhbXBsZS5ldGg6Ly9kZWZhdWx0L2t2L3B1YmxpYyI6WyJsaXN0IiwiZ2V0IiwibWV0YWRhdGEiLCJwdXQiLCJkZWxldGUiXX1
```

### Sign-in only

A Message can be built without any capabilities, in which case a statement with only the "sign-in" message is generated:
```rust
let msg: Message = DelegationBuilder::new()
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
    }))?;
```

Which produces this SIWE message:
```
'example.com wants you to sign in with your Ethereum account:
0x0000000000000000000000000000000000000000


URI: did:key:example
Version: 1
Chain ID: 1
Nonce: mynonce1
Issued At: 2022-06-21T12:00:00.000Z
```
