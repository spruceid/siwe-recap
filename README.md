# SIWE Delegation

Build SIWE messages with capability delegations encoded in the statement and resources.

Use this crate to create a delegation SIWE message, by replacing the resource list with a list of capabilities and the statement with a generated description of those capabilities.

## Example

An example with default actions for the `credential` namespace, and no default actions and multiple targets with actions for the `kepler` namespace.
```rust
let credential: Namespace = "credential".parse().unwrap();
let kepler: Namespace = "kepler".parse().unwrap();

let msg = Builder::new()
    .with_default_actions(&credential, vec!["present".into()])
    .with_actions(
	&kepler,
	"kepler:ens:example.eth://default/kv".to_string(),
	["list", "get", "metadata"].iter().map(|&s| s.into()),
    )
    .with_actions(
	&kepler,
	"kepler:ens:example.eth://default/kv/public".to_string(),
	["list", "get", "metadata", "put", "delete"]
	    .iter()
	    .map(|&s| s.into()),
    )
    .with_actions(
	&kepler,
	"kepler:ens:example.eth://default/kv/dapp-space".to_string(),
	["list", "get", "metadata", "put", "delete"]
	    .iter()
	    .map(|&s| s.into()),
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

By signing this message I am signing in with Ethereum and authorizing the presented URI to perform the following actions on my behalf: (1) credential: present for any. (2) kepler: list, get, metadata for kepler:ens:example.eth://default/kv. (3) kepler: list, get, metadata, put, delete for kepler:ens:example.eth://default/kv/dapp-space, kepler:ens:example.eth://default/kv/public.

URI: did:key:example
Version: 1
Chain ID: 1
Nonce: mynonce1
Issued At: 2022-06-21T12:00:00.000Z
Resources:
- urn:capability:credential:eyJkZWZhdWx0QWN0aW9ucyI6WyJwcmVzZW50Il19
- urn:capability:kepler:eyJ0YXJnZXRlZEFjdGlvbnMiOnsia2VwbGVyOmVuczpleGFtcGxlLmV0aDovL2RlZmF1bHQva3YiOlsibGlzdCIsImdldCIsIm1ldGFkYXRhIl0sImtlcGxlcjplbnM6ZXhhbXBsZS5ldGg6Ly9kZWZhdWx0L2t2L2RhcHAtc3BhY2UiOlsibGlzdCIsImdldCIsIm1ldGFkYXRhIiwicHV0IiwiZGVsZXRlIl0sImtlcGxlcjplbnM6ZXhhbXBsZS5ldGg6Ly9kZWZhdWx0L2t2L3B1YmxpYyI6WyJsaXN0IiwiZ2V0IiwibWV0YWRhdGEiLCJwdXQiLCJkZWxldGUiXX19
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

By signing this message I am signing in with Ethereum.

URI: did:key:example
Version: 1
Chain ID: 1
Nonce: mynonce1
Issued At: 2022-06-21T12:00:00.000Z
```
