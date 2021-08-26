# NFT Series Implementation

## Instructions

`yarn && yarn test:deploy`

#### Pre-reqs

Rust, cargo, near-cli, etc...
Everything should work if you have NEAR development env for Rust contracts set up.

[Tests](test/api.test.js)
[Contract](contract/src/lib.rs)

## Example Call

### Deploy
```
env NEAR_ENV=local near --keyPath ~/.near/localnet/validator_key.json deploy --accountId comic.test.near
```

### Nft init
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near new_default_meta '{"owner_id":"comic.test.near"}'
```

### Nft create type
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_create_type '{"author_id":"alice.test.near","token_metadata":{"title":"Naruto Shippuden ch.2: Menolong sasuke","media":"bafybeidzcan4nzcz7sczs4yzyxly4galgygnbjewipj6haco4kffoqpkiy", "reference":"bafybeicg4ss7qh5odijfn2eogizuxkrdh3zlv4eftcmgnljwu7dm64uwji"},"price":"1000000000000000000000000"}' --depositYocto 11790000000000000000000
```

### Nft buy
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_buy '{"token_type":"1","receiver_id":"comic.test.near"}' --depositYocto 1018320000000000000000000
```

### Nft mint type (Author only)
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_mint_type '{"token_type":"1","receiver_id":"comic.test.near"}' --depositYocto 18320000000000000000000a
```

### Nft transfer
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_transfer '{"token_id":"1:1","receiver_id":"comic1.test.near"}' --depositYocto 1
```

### Nft set type mintable (Author only )
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_set_type_mintable '{"token_type":"1", "is_mintable":false}' --depositYocto 1
```

### Nft set type price (Author only)
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_set_type_price '{"token_type":"1", "price": "2000000000000000000000000"}' --depositYocto 1
```

### Nft burn
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_burn '{"token_id":"1:2"}' --depositYocto 1
```
