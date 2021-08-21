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
$ env NEAR_ENV=local near --keyPath ~/.near/localnet/validator_key.json deploy --accountId comic.test.near
```

### Nft deploy
```
$ env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near new_default_meta '{"owner_id":"comic.test.near"}'
```

### Nft create type
```
$ env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_create_type '{"token_type":"naruto-2","token_metadata":{"title":"Naruto Shippuden ch.2: Menolong sasuke","media":"bafybeiax25bdn5go7b6xoc7kocfw3kbzng7ijnpemdo6aiogumzx53s6ga", "reference":"bafybeihvoofv5rkclwipij3rlozllrpyuc4wpcux6jve6o6qdbm4v7gepi"},"price":"1000000000000000000000000"}' --depositYocto 6560000000000000000000
```

### Nft buy
```
$ env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_buy '{"token_type":"naruto-2","receiver_id":"comic.test.near"}' --depositYocto 1000000000000000000000000
```

### Nft mint
```
$ env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_mint_type '{"token_type":"naruto-2","receiver_id":"comic.test.near"}' --depositYocto 1000000000000000000000000
```

### Nft transfer
```
$ env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_transfer '{"token_id":"naruto-1:1","receiver_id":"comic1.test.near"}' --depositYocto 1
```
