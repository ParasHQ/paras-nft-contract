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

### Nft create series
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_create_series '{"token_series_id":"1", "creator_id":"alice.test.near","token_metadata":{"title":"Naruto Shippuden ch.2: Menolong sasuke","media":"bafybeidzcan4nzcz7sczs4yzyxly4galgygnbjewipj6haco4kffoqpkiy", "reference":"bafybeicg4ss7qh5odijfn2eogizuxkrdh3zlv4eftcmgnljwu7dm64uwji"},"price":"1000000000000000000000000"}' --depositYocto 11790000000000000000000
```

### NFT create series with royalty
```
env NEAR_ENV=lddocal near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_create_series '{"token_series_id":"1","creator_id":"alice.test.near","token_metadata":{"title":"Naruto Shippuden ch.2: Menolong sasuke","media":"bafybeidzcan4nzcz7sczs4yzyxly4galgygnbjewipj6haco4kffoqpkiy", "reference":"bafybeicg4ss7qh5odijfn2eogizuxkrdh3zlv4eftcmgnljwu7dm64uwji"},"price":"1000000000000000000000000", "royalty":{"alice.test.near": 1000}}' --depositYocto 11790000000000000000000
```

### NFT batch mint
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic-migration.test.near comic-migration.test.near nft_mint_batch '{"token_series_id":"1","receiver_ids":["comic.test.near", "comic1.test.near", "comic2.test.near", "comic3.test.near", "comic4.test.near", "comic5.test.near", "comic6.test.near", "comic7.test.near", "comic8.test.near", "comic9.test.near", "comic222.test.near", "comic21.test.near", "comic22.test.near", "comic23.test.near", "comic24.test.near"],"issued_ats":["1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920","1630133919920"]}' --gas 300000000000000
```

### NFT transfer with payout
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_transfer_payout '{"token_id":"10:1","receiver_id":"comic1.test.near","approval_id":"0","balance":"1000000000000000000000000", "max_len_payout": 10}' --depositYocto 1
```

### Nft buy
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_buy '{"token_series_id":"1","receiver_id":"comic.test.near"}' --depositYocto 1018320000000000000000000
```

### Nft mint series(Author only)
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_mint '{"token_series_id":"1","receiver_id":"comic.test.near"}' --depositYocto 18320000000000000000000
```

### Nft transfer
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_transfer '{"token_id":"1:1","receiver_id":"comic1.test.near"}' --depositYocto 1
```

### Nft set series mintable (Author only )
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_set_series_non_mintable '{"token_series_id":"1"}' --depositYocto 1
```

### Nft set series price (Author only)
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_set_series_price '{"token_series_id":"1", "price": "2000000000000000000000000"}' --depositYocto 1
```

### Nft burn
```
env NEAR_ENV=local near call --keyPath ~/.near/localnet/validator_key.json --accountId comic.test.near comic.test.near nft_burn '{"token_id":"1:2"}' --depositYocto 1
```
