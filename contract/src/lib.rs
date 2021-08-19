use near_contract_standards::non_fungible_token::core::{
    NonFungibleTokenCore, NonFungibleTokenResolver,
};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, serde_json::json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue,
};
use std::collections::HashMap;

/// between token_type_id and edition number e.g. 42:2 where 42 is type and 2 is edition
pub const TOKEN_DELIMETER: char = ':';
/// TokenMetadata.title returned for individual token e.g. "Title — 2/10" where 10 is max copies
pub const TITLE_DELIMETER: &str = " #";
/// e.g. "Title — 2/10" where 10 is max copies
pub const EDITION_DELIMETER: &str = "/";

pub const TYPE_OWNERSHIP_DELIMTER: &str = "::";

pub type TokenType = String;
pub type TypeOwnerId = String;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    // CUSTOM
    token_types: UnorderedMap<TokenType, TokenMetadata>,
    type_owners: LookupMap<TokenType, AccountId>,
    tokens_by_type: LookupMap<TokenType, UnorderedSet<TokenId>>,
    type_price: LookupMap<TokenType, u128>,
    type_balances: LookupMap<TypeOwnerId, Balance>,
    type_is_mintable: LookupMap<TokenType, bool>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    // CUSTOM
    TokenTypes,
    TypeOwners,
    TokensByType,
    TokensByTypeInner { token_type: String },
    TypePrice,
    TokensPerOwner { account_hash: Vec<u8> },
    TypeBalances,
    TypeIsMintable,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Comic by Paras".to_string(),
                symbol: "COMIC".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: Some("https://ipfs.fleek.co/ipfs".to_string()),
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: ValidAccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            token_types: UnorderedMap::new(StorageKey::TokenTypes),
            type_owners: LookupMap::new(StorageKey::TypeOwners),
            tokens_by_type: LookupMap::new(StorageKey::TokensByType),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            type_price: LookupMap::new(StorageKey::TypePrice),
            type_balances: LookupMap::new(StorageKey::TypeBalances),
            type_is_mintable: LookupMap::new(StorageKey::TypeIsMintable),
        }
    }

    // CUSTOM

    #[payable]
    pub fn nft_create_type(
        &mut self,
        token_type: TokenType,
        token_metadata: TokenMetadata,
        price: U128,
    ) {
        let initial_storage_usage = env::storage_usage();
        let owner_id = env::predecessor_account_id();
        assert_eq!(
            owner_id, self.tokens.owner_id,
            "Paras: Only owner can set type"
        );

        assert!(
            self.token_types.get(&token_type).is_none(),
            "Paras: duplicate token_type"
        );

        let title = token_metadata.title.clone();
        assert!(title.is_some(), "token_metadata.title is required");

        self.token_types.insert(&token_type, &token_metadata);
        self.type_owners.insert(&token_type, &owner_id);
        self.tokens_by_type.insert(
            &token_type,
            &UnorderedSet::new(
                StorageKey::TokensByTypeInner {
                    token_type: token_type.clone(),
                }
                .try_to_vec()
                .unwrap(),
            ),
        );
        self.type_price.insert(&token_type, &price.into());
        self.type_is_mintable.insert(&token_type, &true);

        env::log(
            json!({
                "type": "create_type",
                "params": {
                    "token_type": token_type,
                    "token_metadata": token_metadata,
                    "price": price
                }
            })
            .to_string()
            .as_bytes(),
        );

        refund_deposit(env::storage_usage() - initial_storage_usage);
    }

    #[payable]
    pub fn nft_buy(&mut self, token_type: TokenType, receiver_id: ValidAccountId) -> Token {
        let price: u128 = self.type_price.get(&token_type).unwrap();
        assert!(
            env::attached_deposit() >= price,
            "Paras: attached deposit is less than price : {}",
            price
        );
        self._nft_mint_type(token_type, receiver_id)
    }

    #[payable]
    pub fn nft_mint_type(&mut self, token_type: TokenType, receiver_id: ValidAccountId) -> Token {
        let type_owner = self.type_owners.get(&token_type).expect("no type owner");
        assert_eq!(env::predecessor_account_id(), type_owner, "not type owner");
        self._nft_mint_type(token_type, receiver_id)
    }

    fn _nft_mint_type(&mut self, token_type: TokenType, receiver_id: ValidAccountId) -> Token {
        assert_eq!(
            self.type_is_mintable.get(&token_type).unwrap(),
            true,
            "Paras: Token type is not mintable"
        );
        let initial_storage_usage = env::storage_usage();

        let mut tokens_by_type = self.tokens_by_type.get(&token_type).unwrap();
        let num_tokens = tokens_by_type.len();

        let token_id = format!("{}{}{}", &token_type, TOKEN_DELIMETER, num_tokens + 1);
        tokens_by_type.insert(&token_id);
        self.tokens_by_type.insert(&token_type, &tokens_by_type);

        // you can add custom metadata to each token here
        let metadata = Some(TokenMetadata {
            title: None,          // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
            description: None,    // free-form description
            media: None, // URL to associated media, preferably to decentralized, content-addressed storage
            media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
            copies: None, // number of copies of this set of metadata in existence when token was minted.
            issued_at: None, // ISO 8601 datetime when token was issued or minted
            expires_at: None, // ISO 8601 datetime when token expires
            starts_at: None, // ISO 8601 datetime when token starts being valid
            updated_at: None, // ISO 8601 datetime when token was last updated
            extra: None, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
            reference: None, // URL to an off-chain JSON file with more info.
            reference_hash: None, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
        });
        //let token = self.tokens.mint(token_id, receiver_id, metadata);
        // From : https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/non_fungible_token/core/core_impl.rs#L359
        let owner_id: AccountId = receiver_id.into();
        self.tokens.owner_by_id.insert(&token_id, &owner_id);

        self.tokens
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, &metadata.as_ref().unwrap()));

        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(&owner_id.as_bytes()),
                })
            });
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }

        let approved_account_ids = if self.tokens.approvals_by_id.is_some() {
            Some(HashMap::new())
        } else {
            None
        };

        let type_owner_id = format!("{}{}{}", token_type, TYPE_OWNERSHIP_DELIMTER, owner_id);
        let owned_supply = self.type_balances.get(&type_owner_id);
        if owned_supply.is_some() {
            self.type_balances
                .insert(&type_owner_id, &(owned_supply.unwrap() + 1));
        } else {
            self.type_balances.insert(&type_owner_id, &1);
        }

        refund_deposit(env::storage_usage() - initial_storage_usage);

        let token_res = self.nft_token(token_id.clone()).unwrap();

        env::log(
            json!({
                "type": "mint",
                "params": {
                    "token_id": token_id,
                    "metadata": token_res.metadata,
                    "owner_id": owner_id
                }
            })
            .to_string()
            .as_bytes(),
        );

        Token {
            token_id,
            owner_id,
            metadata: token_res.metadata,
            approved_account_ids,
        }
    }

    #[payable]
    pub fn nft_set_type_mintable(&mut self, token_type: TokenType, is_mintable: bool) {
        assert_one_yocto();

        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Paras: Owner only"
        );
        self.type_is_mintable.insert(&token_type, &is_mintable);
    }

    // CUSTOM VIEWS

    pub fn nft_get_type_info(
        self,
        token_type: TokenType,
    ) -> (TokenType, AccountId, TokenMetadata, bool) {
        (
            token_type.clone(),
            self.type_owners.get(&token_type).unwrap(),
            self.token_types.get(&token_type).unwrap(),
            self.type_is_mintable.get(&token_type).unwrap(),
        )
    }

    pub fn nft_get_type_format(self) -> (char, &'static str, &'static str) {
        (TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER)
    }

    pub fn nft_get_type(self, token_type: TokenType) -> TokenMetadata {
        self.token_types.get(&token_type).unwrap()
    }

    pub fn nft_get_price(self, token_type: TokenType) -> u128 {
        self.type_price.get(&token_type).unwrap()
    }

    pub fn nft_get_types(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<TokenMetadata> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_types.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        self.token_types
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(_, token_metadata)| token_metadata)
            .collect()
    }

    pub fn nft_supply_for_type(&self, token_type: TokenType) -> U64 {
        let tokens_by_type = self.tokens_by_type.get(&token_type);
        if let Some(tokens_by_type) = tokens_by_type {
            U64(tokens_by_type.len())
        } else {
            U64(0)
        }
    }

    pub fn nft_tokens_by_type(
        &self,
        token_type: TokenType,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        let tokens = self.tokens_by_type.get(&token_type).unwrap();
        assert!(
            (tokens.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        tokens
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        let owner_id = self.tokens.owner_by_id.get(&token_id)?;
        let approved_account_ids = self
            .tokens
            .approvals_by_id
            .as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));

        // CUSTOM (switch metadata for the token_type metadata)
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_type = token_id_iter.next().unwrap().parse().unwrap();
        let mut metadata = self.token_types.get(&token_type).unwrap();
        metadata.title = Some(format!(
            "{}{}{}",
            metadata.title.unwrap(),
            TITLE_DELIMETER,
            token_id_iter.next().unwrap()
        ));
        Some(Token {
            token_id,
            owner_id,
            metadata: Some(metadata),
            approved_account_ids,
        })
    }

    // CUSTOM core standard repeated here because no macro below

    #[payable]
    pub fn nft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        let receiver_id_str = receiver_id.to_string();
        let sender_id = env::predecessor_account_id();
        self.tokens
            .nft_transfer(receiver_id, token_id.clone(), approval_id, memo);
        self.transfer_type_balance_calculation(&sender_id, token_id.clone(), &receiver_id_str);
        env::log(
            json!({
                "type": "transfer",
                "params": {
                    "token_id": token_id,
                    "sender_id": sender_id,
                    "receiver_id": receiver_id_str
                }
            })
            .to_string()
            .as_bytes(),
        );
    }

    #[payable]
    pub fn nft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        let receiver_id_str = receiver_id.to_string();
        let sender_id = env::predecessor_account_id();
        let nft_transfer_call_ret =
            self.tokens
                .nft_transfer_call(receiver_id, token_id.clone(), approval_id, memo, msg);
        self.transfer_type_balance_calculation(&sender_id, token_id.clone(), &receiver_id_str);
        env::log(
            json!({
                "type": "transfer",
                "params": {
                    "token_id": token_id,
                    "sender_id": sender_id,
                    "receiver_id": receiver_id_str
                }
            })
            .to_string()
            .as_bytes(),
        );
        nft_transfer_call_ret
    }

    fn transfer_type_balance_calculation(
        &mut self,
        sender_id: &AccountId,
        token_id: TokenId,
        receiver_id: &AccountId,
    ) {
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_type: TokenType = token_id_iter.next().unwrap().parse().unwrap();

        let type_sender_id = format!(
            "{}{}{}",
            token_type,
            TYPE_OWNERSHIP_DELIMTER,
            sender_id.clone()
        );
        let type_receiver_id = format!(
            "{}{}{}",
            token_type,
            TYPE_OWNERSHIP_DELIMTER,
            receiver_id.clone()
        );

        let supply_receiver = self.type_balances.get(&type_receiver_id);
        let supply_sender = self.type_balances.get(&type_sender_id);

        if supply_receiver.is_some() {
            self.type_balances
                .insert(&type_receiver_id, &(supply_receiver.unwrap() + 1));
        } else {
            self.type_balances.insert(&type_receiver_id, &1);
        }
        self.type_balances
            .insert(&type_sender_id, &(supply_sender.unwrap() - 1));
    }

    // CUSTOM enumeration standard modified here because no macro below

    pub fn nft_total_supply(&self) -> U128 {
        (self.tokens.owner_by_id.len() as u128).into()
    }

    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        // Get starting index, whether or not it was explicitly given.
        // Defaults to 0 based on the spec:
        // https://nomicon.io/Standards/NonFungibleToken/Enumeration.html#interface
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.tokens.owner_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.tokens
            .owner_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_id, _)| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn nft_supply_for_owner(self, account_id: ValidAccountId) -> U128 {
        let tokens_per_owner = self.tokens.tokens_per_owner.expect(
            "Could not find tokens_per_owner when calling a method on the enumeration standard.",
        );
        tokens_per_owner
            .get(account_id.as_ref())
            .map(|account_tokens| U128::from(account_tokens.len() as u128))
            .unwrap_or(U128(0))
    }

    pub fn nft_type_balance(self, account_id: ValidAccountId, token_type: TokenType) -> U128 {
        let type_owner_id = format!("{}{}{}", token_type, TYPE_OWNERSHIP_DELIMTER, account_id);
        let bal = self.type_balances.get(&type_owner_id);
        if bal.is_some() {
            bal.unwrap().into()
        } else {
            U128(0)
        }
    }

    pub fn nft_tokens_for_owner(
        &self,
        account_id: ValidAccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let tokens_per_owner = self.tokens.tokens_per_owner.as_ref().expect(
            "Could not find tokens_per_owner when calling a method on the enumeration standard.",
        );
        let token_set = if let Some(token_set) = tokens_per_owner.get(account_id.as_ref()) {
            token_set
        } else {
            return vec![];
        };
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            token_set.len() as u128 > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        token_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }
}

// near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
// near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        self.tokens.nft_resolve_transfer(
            previous_owner_id,
            receiver_id,
            token_id,
            approved_account_ids,
        )
    }
}

/// from https://github.com/near/near-sdk-rs/blob/e4abb739ff953b06d718037aa1b8ab768db17348/near-contract-standards/src/non_fungible_token/utils.rs#L29

pub fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    // log!("refund_deposit amount {}", refund);
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}
