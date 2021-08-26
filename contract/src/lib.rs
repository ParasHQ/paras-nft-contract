use near_contract_standards::non_fungible_token::core::{
    NonFungibleTokenCore, NonFungibleTokenResolver,
};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, serde_json::json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue,
};
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// between token_type_id and edition number e.g. 42:2 where 42 is type and 2 is edition
pub const TOKEN_DELIMETER: char = ':';
/// TokenMetadata.title returned for individual token e.g. "Title — 2/10" where 10 is max copies
pub const TITLE_DELIMETER: &str = " #";
/// e.g. "Title — 2/10" where 10 is max copies
pub const EDITION_DELIMETER: &str = "/";

pub type TokenTypeId = String;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenType {
	metadata: TokenMetadata,
	author_id: AccountId,
	tokens: UnorderedSet<TokenId>,
    price: Balance,
    is_mintable: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenTypeJson {
	metadata: TokenMetadata,
	author_id: AccountId,
}

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    // CUSTOM
	token_type_by_id: UnorderedMap<TokenTypeId, TokenType>,
}

const DATA_IMAGE_SVG_COMIC_ICON: &str = "data:image/svg+xml,%3Csvg width='80' height='80' viewBox='0 0 80 80' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M10 79C5.02944 79 1 74.9706 1 70V10C1 5.02944 5.02944 1 10 1H71C75.4183 1 79 4.58172 79 9V70C79 74.9706 74.9706 79 70 79H10Z' fill='%2318162B' stroke='%23C6FF00' stroke-width='2'/%3E%3Cpath d='M71 0L77 3L80 9H71V0Z' fill='%23C6FF00'/%3E%3Cpath d='M9 80L3.5 76.5L0 71H9V80Z' fill='%23C6FF00'/%3E%3Cpath d='M40.7745 64C35.0045 64 30.433 62.3846 27.0598 59.1538C23.6866 55.8782 22 51.2115 22 45.1538V33.8462C22 27.7885 23.6866 23.1442 27.0598 19.9135C30.433 16.6378 35.0045 15 40.7745 15C46.5 15 50.9162 16.5929 54.0231 19.7788C57.1744 22.9199 58.75 27.25 58.75 32.7692V33.1731H50.0951V32.5C50.0951 29.7179 49.3184 27.4295 47.7649 25.6346C46.2559 23.8397 43.9257 22.9423 40.7745 22.9423C37.6676 22.9423 35.2264 23.9071 33.4511 25.8365C31.6757 27.766 30.788 30.391 30.788 33.7115V45.2885C30.788 48.5641 31.6757 51.1891 33.4511 53.1635C35.2264 55.0929 37.6676 56.0577 40.7745 56.0577C43.9257 56.0577 46.2559 55.1603 47.7649 53.3654C49.3184 51.5256 50.0951 49.2372 50.0951 46.5V45.2885H58.75V46.2308C58.75 51.75 57.1744 56.1026 54.0231 59.2885C50.9162 62.4295 46.5 64 40.7745 64Z' fill='%23C6FF00'/%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    // CUSTOM
    TokenTypeById,
    TokensByTypeInner { token_type: String },
    TokensPerOwner { account_hash: Vec<u8> },
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
                icon: Some(DATA_IMAGE_SVG_COMIC_ICON.to_string()),
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
            token_type_by_id: UnorderedMap::new(StorageKey::TokenTypeById),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    // CUSTOM

    #[payable]
    pub fn nft_create_type(
        &mut self,
        token_metadata: TokenMetadata,
        author_id: ValidAccountId,
        price: U128,
    ) {
        let initial_storage_usage = env::storage_usage();
        let owner_id = env::predecessor_account_id();
        assert_eq!(
            owner_id, self.tokens.owner_id,
            "Paras: Only owner can set type"
        );

        let token_type: String = format!(
            "{}", (self.token_type_by_id.len() + 1));

        assert!(
            self.token_type_by_id.get(&token_type).is_none(),
            "Paras: duplicate token_type"
        );

        let title = token_metadata.title.clone();
        assert!(title.is_some(), "token_metadata.title is required");

        self.token_type_by_id.insert(&token_type, &TokenType{
            metadata: token_metadata.clone(),
            author_id: author_id.to_string(),
            tokens: UnorderedSet::new(
                StorageKey::TokensByTypeInner {
                    token_type: token_type.clone(),
                }
                .try_to_vec()
                .unwrap(),
            ),
            price: price.into(),
            is_mintable: true,
        });

        env::log(
            json!({
                "type": "create_type",
                "params": {
                    "token_type": token_type,
                    "token_metadata": token_metadata,
                    "author_id": author_id,
                    "price": price
                }
            })
            .to_string()
            .as_bytes(),
        );

        refund_deposit(env::storage_usage() - initial_storage_usage, 0);
    }

    #[payable]
    pub fn nft_buy(&mut self, token_type: TokenTypeId, receiver_id: ValidAccountId) -> Token {
        let initial_storage_usage = env::storage_usage();

        let token_type_res = self.token_type_by_id.get(&token_type).expect("Token type not exist");
        let price: u128 = token_type_res.price;
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= price,
            "Paras: attached deposit is less than price : {}",
            price
        );
        let token: Token = self._nft_mint_type(token_type, receiver_id);
        Promise::new(token_type_res.author_id).transfer(price);

        refund_deposit(env::storage_usage() - initial_storage_usage, price);
        token
    }

    #[payable]
    pub fn nft_mint_type(&mut self, token_type: TokenTypeId, receiver_id: ValidAccountId) -> Token {
        let initial_storage_usage = env::storage_usage();

        let token_type_res = self.token_type_by_id.get(&token_type).expect("Token type not exist");
        assert_eq!(env::predecessor_account_id(), token_type_res.author_id, "not type owner");
        let token: Token = self._nft_mint_type(token_type, receiver_id);

        refund_deposit(env::storage_usage() - initial_storage_usage, 0);
        token
    }

    fn _nft_mint_type(&mut self, token_type: TokenTypeId, receiver_id: ValidAccountId) -> Token {
        let mut token_type_res = self.token_type_by_id.get(&token_type).expect("Token type not exist");
        assert!(
            token_type_res.is_mintable,
            "Paras: Token type is not mintable"
        );

        let num_tokens = token_type_res.tokens.len();
        let max_copies = token_type_res.metadata.copies.unwrap_or(u64::MAX);
        assert_ne!(num_tokens, max_copies, "Type supply maxed");

        let token_id = format!("{}{}{}", &token_type, TOKEN_DELIMETER, token_type_res.tokens.len() + 1);
        token_type_res.tokens.insert(&token_id);
        self.token_type_by_id.insert(&token_type, &token_type_res);

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
        // This allows lazy minting

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

        let token_res = self.nft_token(token_id.clone()).unwrap();

        env::log(
            json!({
                "type": "mint",
                "params": {
                    "token_id": token_id,
                    "metadata": token_res.metadata,
                    "owner_id": token_res.owner_id
                }
            })
            .to_string()
            .as_bytes(),
        );

        Token {
            token_id,
            owner_id: token_res.owner_id,
            metadata: token_res.metadata,
            approved_account_ids: token_res.approved_account_ids
        }
    }

    #[payable]
    pub fn nft_set_type_mintable(&mut self, token_type: TokenTypeId, is_mintable: bool) {
        assert_one_yocto();

        let mut token_type_res = self.token_type_by_id.get(&token_type).expect("Token type not exist");
        assert_eq!(
            env::predecessor_account_id(),
            token_type_res.author_id,
            "Paras: Author only"
        );
        token_type_res.is_mintable = is_mintable;
        self.token_type_by_id.insert(&token_type, &token_type_res);
    }

    #[payable]
    pub fn nft_set_type_price(&mut self, token_type: TokenTypeId, price: U128) -> U128 {
        assert_one_yocto();

        let mut token_type_res = self.token_type_by_id.get(&token_type).expect("Token type not exist");
        assert_eq!(
            env::predecessor_account_id(),
            token_type_res.author_id,
            "Paras: Author only"
        );

        token_type_res.price = price.into();
        self.token_type_by_id.insert(&token_type, &token_type_res);
        env::log(
            json!({
                "type": "set_type_price",
                "params": {
                    "token_type": token_type,
                    "price": price
                }
            })
            .to_string()
            .as_bytes(),
        );
        return token_type_res.price.into();
    }

    #[payable]
    pub fn nft_burn(&mut self, token_id: TokenId) {
        assert_one_yocto();

        let owner_id = self.tokens.owner_by_id.get(&token_id).unwrap();
        assert_eq!(
            owner_id,
            env::predecessor_account_id(),
            "Token owner only"
        );

        if let Some(next_approval_id_by_id) = &mut self.tokens.next_approval_id_by_id {
            next_approval_id_by_id.remove(&token_id);
        }

        if let Some(approvals_by_id) = &mut self.tokens.approvals_by_id {
            approvals_by_id.remove(&token_id);
        }

        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap();
            token_ids.remove(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }

        if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
            token_metadata_by_id.remove(&token_id);
        }

        self.tokens.owner_by_id.remove(&token_id);
        env::log(
            json!({
                "type": "burn",
                "params": {
                    "token_id": token_id,
                    "owner_id": owner_id,
                }
            })
            .to_string()
            .as_bytes(),
        );
    }

    // CUSTOM VIEWS

	pub fn nft_get_type(&self, token_type: TokenTypeId) -> TokenTypeJson {
		let token_type = self.token_type_by_id.get(&token_type).expect("no type");
		TokenTypeJson{
			metadata: token_type.metadata,
			author_id: token_type.author_id,
		}
	}

    pub fn nft_get_type_format(self) -> (char, &'static str, &'static str) {
        (TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER)
    }

    pub fn nft_get_price(self, token_type: TokenTypeId) -> Balance {
        self.token_type_by_id.get(&token_type).unwrap().price
    }

    pub fn nft_get_types(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<TokenTypeJson> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_type_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        self.token_type_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(_, token_type)| TokenTypeJson{
                metadata: token_type.metadata,
                author_id: token_type.author_id,
            })
            .collect()
    }

    pub fn nft_supply_for_type(&self, token_type: TokenTypeId) -> U64 {
        self.token_type_by_id.get(&token_type).expect("Token type not exist").tokens.len().into()
    }

    pub fn nft_tokens_by_type(
        &self,
        token_type: TokenTypeId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        let tokens = self.token_type_by_id.get(&token_type).unwrap().tokens;
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
        let mut metadata = self.token_type_by_id.get(&token_type).unwrap().metadata;
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

fn refund_deposit(storage_used: u64, extra_spend: Balance) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit() - extra_spend;

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}
