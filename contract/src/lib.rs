use std::collections::HashMap;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::core::{
	NonFungibleTokenCore, NonFungibleTokenResolver
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{U64, ValidAccountId};
use near_sdk::{
    env, near_bindgen, Balance, CryptoHash, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

pub type TokenType = String;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
	// CUSTOM
	token_types: LookupMap<TokenType, TokenMetadata>,
	type_owners: LookupMap<TokenType, AccountId>,
	tokens_by_type: LookupMap<TokenType, UnorderedSet<TokenId>>,
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
    TokensByTypeInner { token_type_hash: CryptoHash },
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Sonar by Satori".to_string(),
                symbol: "SONAR".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
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
			token_types: LookupMap::new(StorageKey::TokenTypes),
			type_owners: LookupMap::new(StorageKey::TypeOwners),
			tokens_by_type: LookupMap::new(StorageKey::TokensByType),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

	// CUSTOM
    
    #[payable]
    pub fn set_type(
        &mut self,
		token_type: String,
        token_metadata: TokenMetadata,
    ) {
		let initial_storage_usage = env::storage_usage();
        let owner_id = env::predecessor_account_id();

        assert!(self.type_owners.insert(&token_type, &owner_id).is_none(), "type exists");
		self.token_types.insert(&token_type, &token_metadata);
		self.tokens_by_type.insert(&token_type, &UnorderedSet::new(
			StorageKey::TokensByTypeInner {
				token_type_hash: hash_account_id(&token_type),
			}
			.try_to_vec()
			.unwrap(),
		));

        refund_deposit(env::storage_usage() - initial_storage_usage);
    }

	#[payable]
	pub fn nft_mint_type(
		&mut self,
		token_type: TokenType,
		receiver_id: ValidAccountId,
	) -> Token {
		let initial_storage_usage = env::storage_usage();

		let type_owner = self.type_owners.get(&token_type).expect("no token type owner");
		assert_eq!(env::predecessor_account_id(), type_owner, "you are not the type owner");

		let mut tokens_by_type = self.tokens_by_type.get(&token_type).expect("no token type");
		let num_tokens = tokens_by_type.len();
		let type_metadata = self.token_types.get(&token_type).expect("no token type");
		let max_copies = type_metadata.copies.unwrap_or(u64::MAX);
		assert_ne!(num_tokens, max_copies, "type supply maxed");

		let token_id = format!("{}:{}", &token_type, num_tokens + 1);
		tokens_by_type.insert(&token_id);
		self.tokens_by_type.insert(&token_type, &tokens_by_type);
		let token_metadata = type_as_metadata(token_type);
		let token = self.tokens.mint(token_id, receiver_id, Some(token_metadata));

        refund_deposit(env::storage_usage() - initial_storage_usage);
		token
	}

	// CUSTOM VIEWS

	pub fn get_type(self, token_type: TokenType) -> (AccountId, TokenMetadata) {
		let type_owner = self.type_owners.get(&token_type).expect("no token type owner");
		let token_type = self.token_types.get(&token_type).expect("no token type");
		(type_owner, token_type)
	}

	pub fn nft_supply_for_type(
        &self,
        token_type: String,
    ) -> U64 {
        let tokens_by_type = self.tokens_by_type.get(&token_type);
        if let Some(tokens_by_type) = tokens_by_type {
            U64(tokens_by_type.len())
        } else {
            U64(0)
        }
    }

	pub fn nft_tokens_by_type(
		&self,
        token_type: String,
		from_index: Option<U128>,
		limit: Option<u64>
	) -> Vec<Token> {

        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
		let tokens = self.tokens_by_type.get(&token_type).expect("no token type");
        assert!(
            (tokens.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        
		tokens.iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }

	pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
		let owner_id = self.tokens.owner_by_id.get(&token_id)?;
        let mut metadata = self.tokens.token_metadata_by_id.as_ref().and_then(|by_id| by_id.get(&token_id));
        let approved_account_ids = self.tokens
            .approvals_by_id
			.as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));

		// CUSTOM (switch metadata for the token_type metadata)
		metadata = self.token_types.get(&metadata.unwrap().title.unwrap());

        Some(Token { token_id, owner_id, metadata, approved_account_ids })
	}

	// core here because not using macro below

	#[payable]
	pub fn nft_transfer(
		&mut self,
		receiver_id: ValidAccountId,
		token_id: TokenId,
		approval_id: Option<u64>,
		memo: Option<String>,
	) {
		self.tokens.nft_transfer(receiver_id, token_id, approval_id, memo)
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
		self.tokens.nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
	}

}

// near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

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

pub fn type_as_metadata(token_type: String) -> TokenMetadata {
	TokenMetadata {
		title: Some(token_type),
		description: None, // free-form description
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
	}
}

pub fn hash_account_id(account_id: &str) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}