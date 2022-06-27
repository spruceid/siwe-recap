# SIWE Delegation

Build SIWE messages with capability delegations encoded in the statement and resources.

Use this crate to create a delegation SIWE message, by replacing the resource list with a list of capabilities and the statement with a generated description of those capabilities.

## Example

An example with default actions for the `credential` namespace, and no default actions and multiple targets with actions for the `kepler` namespace.
```rust
let msg: Message = DelegationBuilder::new(Message {
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
.with_capability(
    Capability::new("credential".into(), vec!["present".into()]).unwrap(),
)
.with_capability(
    Capability::new("kepler".into(), vec![])
	.unwrap()
	.with_actions(
	    "kepler:ens:example.eth://default/kv".parse().unwrap(),
	    vec!["list".into(), "get".into(), "metadata".into()],
	)
	.with_actions(
	    "kepler:ens:example.eth://default/kv/dapp-space"
		.parse()
		.unwrap(),
	    vec![
		"list".into(),
		"get".into(),
		"metadata".into(),
		"put".into(),
		"delete".into(),
	    ],
	),
)
.build()?;
```

Which produces this SIWE message:
```
example.com wants you to sign in with your Ethereum account:
0x0000000000000000000000000000000000000000
By signing this message I am signing in with Ethereum and authorizing the presented URI to perform the following actions on my behalf: (1) credential: present for any. (2) kepler: list, get, metadata for kepler:ens:example.eth://default/kv. (3) kepler: list, get, metadata, put, delete for kepler:ens:example.eth://default/kv/dapp-space.
URI: did:key:example
Version: 1
Chain ID: 1
Nonce: mynonce1
Issued At: 2022-06-21T12:00:00.000Z
Resources:
- urn:capability:credential:eyJkZWZhdWx0X2FjdGlvbnMiOlsicHJlc2VudCJdfQ
- urn:capability:kepler:eyJ0YXJnZXRlZF9hY3Rpb25zIjp7ImtlcGxlcjplbnM6ZXhhbXBsZS5ldGg6Ly9kZWZhdWx0L2t2IjpbImxpc3QiLCJnZXQiLCJtZXRhZGF0YSJdLCJrZXBsZXI6ZW5zOmV4YW1wbGUuZXRoOi8vZGVmYXVsdC9rdi9kYXBwLXNwYWNlIjpbImxpc3QiLCJnZXQiLCJtZXRhZGF0YSIsInB1dCIsImRlbGV0ZSJdfX0
```

### Sign-in only

A Message can be built without any capabilities, in which case a statement with only the "sign-in" message is generated:
```rust
let msg: Message = DelegationBuilder::new(Message {
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
.build()?;
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
