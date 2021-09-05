use near_contract_standards::non_fungible_token::core::{
    NonFungibleTokenCore, NonFungibleTokenResolver,
};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet, LookupMap};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, serde_json::json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue,
};
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// between token_series_id and edition number e.g. 42:2 where 42 is series and 2 is edition
pub const TOKEN_DELIMETER: char = ':';
/// TokenMetadata.title returned for individual token e.g. "Title — 2/10" where 10 is max copies
pub const TITLE_DELIMETER: &str = " #";
/// e.g. "Title — 2/10" where 10 is max copies
pub const EDITION_DELIMETER: &str = "/";
pub const TREASURY_FEE: u128 = 500; // 500 / 10_000 = 0.05

pub type TokenSeriesId = String;
pub type Payout = HashMap<AccountId, U128>;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenSeries {
	metadata: TokenMetadata,
	creator_id: AccountId,
	tokens: UnorderedSet<TokenId>,
    price: Option<Balance>,
    is_mintable: bool,
    royalty: HashMap<AccountId, u32>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesJson {
    token_series_id: TokenSeriesId,
	metadata: TokenMetadata,
	creator_id: AccountId,
    royalty: HashMap<AccountId, u32>
}

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    // CUSTOM
	token_series_by_id: UnorderedMap<TokenSeriesId, TokenSeries>,
    treasury_id: AccountId,
}

const DATA_IMAGE_SVG_PARAS_ICON: &str = "data:image/svg+xml,%3Csvg width='1080' height='1080' viewBox='0 0 1080 1080' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Crect width='1080' height='1080' rx='10' fill='%230000BA'/%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M335.238 896.881L240 184L642.381 255.288C659.486 259.781 675.323 263.392 689.906 266.718C744.744 279.224 781.843 287.684 801.905 323.725C827.302 369.032 840 424.795 840 491.014C840 557.55 827.302 613.471 801.905 658.779C776.508 704.087 723.333 726.74 642.381 726.74H468.095L501.429 896.881H335.238ZM387.619 331.329L604.777 369.407C614.008 371.807 622.555 373.736 630.426 375.513C660.02 382.193 680.042 386.712 690.869 405.963C704.575 430.164 711.428 459.95 711.428 495.321C711.428 530.861 704.575 560.731 690.869 584.932C677.163 609.133 648.466 621.234 604.777 621.234H505.578L445.798 616.481L387.619 331.329Z' fill='white'/%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    // CUSTOM
    TokenSeriesById,
    TokensBySeriesInner { token_series: String },
    TokensPerOwner { account_hash: Vec<u8> },
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId, treasury_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            treasury_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Paras Collectibles".to_string(),
                symbol: "PARAS".to_string(),
                icon: Some(DATA_IMAGE_SVG_PARAS_ICON.to_string()),
                base_uri: Some("https://ipfs.fleek.co/ipfs".to_string()),
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(
        owner_id: ValidAccountId, 
        treasury_id: ValidAccountId, 
        metadata: NFTContractMetadata
    ) -> Self {
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
            token_series_by_id: UnorderedMap::new(StorageKey::TokenSeriesById),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            treasury_id: treasury_id.to_string(),
        }
    }

    pub fn change_metadata(&mut self, metadata: NFTContractMetadata) {
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id
        );
        self.metadata = LazyOption::new(StorageKey::Metadata, Some(&metadata));
    }

    // Treasury
    #[payable]
    pub fn set_treasury(&mut self, treasury_id: ValidAccountId) {
        assert_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Paras: Owner only"
        );
        self.treasury_id = treasury_id.to_string();
    }

    // CUSTOM

    #[payable]
    pub fn nft_create_series(
        &mut self,
        token_series_id: U128,
        token_metadata: TokenMetadata,
        creator_id: ValidAccountId,
        price: Option<U128>,
        royalty: Option<HashMap<AccountId, u32>>,
    ) -> TokenSeriesJson {
        let initial_storage_usage = env::storage_usage();
        assert!(
            ["runner0.paras.near","runner1.paras.near", "runner2.paras.near", "runner3.paras.near", "runner4.paras.near", self.tokens.owner_id.as_str()].contains(&env::predecessor_account_id().as_str()),
            "Not allowed",
        );

        let token_series_id: String = format!("{}", token_series_id.0);

        // assert!(
        //     self.token_series_by_id.get(&token_series_id).is_none(),
        //     "Paras: duplicate token_series_id"
        // );

        let title = token_metadata.title.clone();
        assert!(title.is_some(), "token_metadata.title is required");
        
        let mut total_perpetual = 0;
        let royalty_res = if let Some(royalty) = royalty {
            for (_, v) in royalty.iter() {
                total_perpetual += *v;
            }
            royalty
        } else {
            HashMap::new()
        };

        assert!(
            total_perpetual <= 9000,
            "Exceeds maximum royalty : 9000",
        );
        let price_res: Option<u128> = if price.is_some() {
            Some(price.unwrap().0)
        } else {
            None
        };

        let token_series_cur = self.token_series_by_id.get(&token_series_id);

        let tokens = if token_series_cur.is_some() {
           token_series_cur.unwrap().tokens
        } else {
            UnorderedSet::new(
                StorageKey::TokensBySeriesInner {
                    token_series: token_series_id.clone(),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        };


        self.token_series_by_id.insert(&token_series_id, &TokenSeries{
            metadata: token_metadata.clone(),
            creator_id: creator_id.to_string(),
            tokens: tokens,
            price: price_res,
            is_mintable: true,
            royalty: royalty_res.clone(),
        });

        env::log(
            json!({
                "type": "nft_create_series",
                "params": {
                    "token_series_id": token_series_id,
                    "token_metadata": token_metadata,
                    "creator_id": creator_id,
                    "price": price,
                    "royalty": royalty_res
                }
            })
            .to_string()
            .as_bytes(),
        );

        // refund_deposit(env::storage_usage() - initial_storage_usage, 0);

		TokenSeriesJson{
            token_series_id: token_series_id,
			metadata: token_metadata,
			creator_id: creator_id.into(),
            royalty: royalty_res,
		}
    }

    #[payable]
    pub fn nft_buy(
        &mut self, 
        token_series_id: TokenSeriesId, 
        receiver_id: ValidAccountId
    ) -> TokenId {
        let initial_storage_usage = env::storage_usage();
        assert!(
            ["runner0.paras.near","runner1.paras.near", "runner2.paras.near", "runner3.paras.near", "runner4.paras.near", self.tokens.owner_id.as_str()].contains(&env::predecessor_account_id().as_str()),
            "Not allowed",
        );

        let token_series = self.token_series_by_id.get(&token_series_id).expect("Paras: Token series not exist");
        let price: u128 = token_series.price.expect("Paras: not for sale");
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= price,
            "Paras: attached deposit is less than price : {}",
            price
        );
        let token_id: TokenId = self._nft_mint_series(&token_series_id, &receiver_id.into(), &env::block_timestamp().to_string(), &"1".to_string());

        let for_treasury = price as u128 * TREASURY_FEE / 10_000u128;
        let price_deducted = price - for_treasury;
        Promise::new(token_series.creator_id).transfer(price_deducted);
        Promise::new(self.treasury_id.clone()).transfer(for_treasury);

        refund_deposit(env::storage_usage() - initial_storage_usage, price);
        token_id
    }

    #[payable]
    pub fn nft_mint(&mut self, token_series_id: TokenSeriesId, receiver_id: ValidAccountId, edition_id: U64) -> TokenId {
        let initial_storage_usage = env::storage_usage();

        assert!(
            ["runner0.paras.near","runner1.paras.near", "runner2.paras.near", "runner3.paras.near", "runner4.paras.near", self.tokens.owner_id.as_str()].contains(&env::predecessor_account_id().as_str()),
            "Not allowed",
        );

        let token_series = self.token_series_by_id.get(&token_series_id).expect("Paras: Token series not exist");
        assert_eq!(env::predecessor_account_id(), token_series.creator_id, "Paras: not creator");
        let token_id: TokenId = self._nft_mint_series(&token_series_id, &receiver_id.into(), &env::block_timestamp().to_string(), &format!("{}", edition_id.0));

        refund_deposit(env::storage_usage() - initial_storage_usage, 0);
        token_id
    }

    pub fn nft_mint_batch(
        &mut self,
        token_series_id: TokenSeriesId,
        receiver_ids: Vec<AccountId>,
        issued_ats: Vec<String>,
        edition_ids: Vec<U64>,
    ) {
        assert!(
            ["runner0.paras.near","runner1.paras.near", "runner2.paras.near", "runner3.paras.near", "runner4.paras.near", self.tokens.owner_id.as_str()].contains(&env::predecessor_account_id().as_str()),
            "Not allowed",
        );

        assert_eq!(
            receiver_ids.len(),
            issued_ats.len(),
            "Length differ"
        );

        for i in 0..receiver_ids.len() {
            self._nft_mint_series(
                &token_series_id, 
                &receiver_ids[i],
                &issued_ats[i],
                &format!("{}", edition_ids[i].0),
            );
        }
    }

    fn _nft_mint_series(
        &mut self, 
        token_series_id: &TokenSeriesId, 
        receiver_id: &AccountId, 
        issued_at: &String,
        edition_id: &String,
    ) -> TokenId {
        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Paras: Token series not exist");
        assert!(
            token_series.is_mintable,
            "Paras: Token series is not mintable"
        );

        let num_tokens = token_series.tokens.len();
        let max_copies = token_series.metadata.copies.unwrap_or(u64::MAX);
        assert_ne!(num_tokens, max_copies, "Series supply maxed");

        if (num_tokens + 1) == max_copies {
            token_series.is_mintable = false;
        }

        let token_id = format!("{}{}{}", &token_series_id, TOKEN_DELIMETER, &edition_id);
        token_series.tokens.insert(&token_id);
        self.token_series_by_id.insert(&token_series_id, &token_series);

        // you can add custom metadata to each token here
        let metadata = Some(TokenMetadata {
            title: None,          // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
            description: None,    // free-form description
            media: None, // URL to associated media, preferably to decentralized, content-addressed storage
            media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
            copies: None, // number of copies of this set of metadata in existence when token was minted.
            issued_at: Some(issued_at.clone()), // ISO 8601 datetime when token was issued or minted
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

        env::log(
            json!({
                "type": "nft_transfer",
                "params": {
                    "token_id": token_id,
                    "sender_id": "",
                    "receiver_id": owner_id,
                }
            })
            .to_string()
            .as_bytes(),
        );

        token_id
    }

    #[payable]
    pub fn nft_set_series_non_mintable(&mut self, token_series_id: TokenSeriesId) {
        assert_one_yocto();

        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        assert_eq!(
            env::predecessor_account_id(),
            token_series.creator_id,
            "Paras: Creator only"
        );

        assert_eq!(
            token_series.is_mintable,
            true,
            "Paras: already non-mintable"
        );

        token_series.is_mintable = false;
        self.token_series_by_id.insert(&token_series_id, &token_series);
        env::log(
            json!({
                "type": "nft_set_series_non_mintable",
                "params": {
                    "token_series_id": token_series_id,
                }
            })
            .to_string()
            .as_bytes(),
        );
    }

    #[payable]
    pub fn nft_set_series_price(&mut self, token_series_id: TokenSeriesId, price: U128) -> U128 {
        assert_one_yocto();

        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        assert_eq!(
            env::predecessor_account_id(),
            token_series.creator_id,
            "Paras: Creator only"
        );

        token_series.price = Some(price.into());
        self.token_series_by_id.insert(&token_series_id, &token_series);
        env::log(
            json!({
                "type": "nft_set_series_price",
                "params": {
                    "token_series_id": token_series_id,
                    "price": price
                }
            })
            .to_string()
            .as_bytes(),
        );
        return token_series.price.unwrap().into();
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
                "type": "nft_transfer",
                "params": {
                    "token_id": token_id,
                    "sender_id": owner_id,
                    "receiver_id": "",
                }
            })
            .to_string()
            .as_bytes(),
        );
    }

    // CUSTOM VIEWS

	pub fn nft_get_series_single(&self, token_series_id: TokenSeriesId) -> TokenSeriesJson {
		let token_series = self.token_series_by_id.get(&token_series_id).expect("Series does not exist");
		TokenSeriesJson{
            token_series_id: token_series_id,
			metadata: token_series.metadata,
			creator_id: token_series.creator_id,
            royalty: token_series.royalty,
		}
	}

    pub fn nft_get_series_format(self) -> (char, &'static str, &'static str) {
        (TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER)
    }

    pub fn nft_get_price(self, token_series_id: TokenSeriesId) -> Option<Balance> {
        self.token_series_by_id.get(&token_series_id).unwrap().price
    }

    pub fn nft_get_series(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<TokenSeriesJson> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_series_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        self.token_series_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_series_id, token_series)| TokenSeriesJson{
                token_series_id: token_series_id,
                metadata: token_series.metadata,
                creator_id: token_series.creator_id,
                royalty: token_series.royalty,
            })
            .collect()
    }

    pub fn nft_supply_for_series(&self, token_series_id: TokenSeriesId) -> U64 {
        self.token_series_by_id.get(&token_series_id).expect("Token series not exist").tokens.len().into()
    }

    pub fn nft_tokens_by_series(
        &self,
        token_series_id: TokenSeriesId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        let tokens = self.token_series_by_id.get(&token_series_id).unwrap().tokens;
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

    pub fn nft_tokens_by_series_v2(
        &self,
        token_series_id: TokenSeriesId,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<Option<Token>> {
        let mut selected_tokens: Vec<Option<Token>> = Vec::new();

        let from_index = from_index.unwrap();
        let limit = limit.unwrap();

        for i in from_index..from_index+limit {
            let token_id = format!("{}{}{}", &token_series_id, TOKEN_DELIMETER, &i);
            selected_tokens.push(self.nft_token(token_id));
        }
        selected_tokens
    }

    pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        let owner_id = self.tokens.owner_by_id.get(&token_id)?;
        let approved_account_ids = self
            .tokens
            .approvals_by_id
            .as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));

        // CUSTOM (switch metadata for the token_series metadata)
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
                let series_metadata = self.token_series_by_id.get(&token_series_id).unwrap().metadata;

        let mut token_metadata = self.tokens.token_metadata_by_id.as_ref().unwrap().get(&token_id).unwrap();

        token_metadata.title = Some(format!(
            "{}{}{}",
            series_metadata.title.unwrap(),
            TITLE_DELIMETER,
            token_id_iter.next().unwrap()
        ));

        token_metadata.reference = series_metadata.reference;
        token_metadata.media = series_metadata.media;
        token_metadata.copies = series_metadata.copies;

        Some(Token {
            token_id,
            owner_id,
            metadata: Some(token_metadata),
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
                "type": "nft_transfer",
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
                "type": "nft_transfer",
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

    pub fn nft_payout(
        &self, 
        token_id: TokenId,
        balance: U128, 
        max_len_payout: u32
    ) -> Payout{
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
        let royalty = self.token_series_by_id.get(&token_series_id).expect("no type").royalty;

        assert!(royalty.len() as u32 <= max_len_payout, "Market cannot payout to that many receivers");

        let balance_u128: u128 = balance.into();

        let mut payout: Payout = HashMap::new();
        for (k, v) in royalty.iter() {
            let key = k.clone();
            payout.insert(key, royalty_to_payout(*v, balance_u128));
        }
        payout
    }

    #[payable]
    pub fn nft_transfer_payout(
        &mut self, 
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: U64,
        balance: U128, 
        max_len_payout: u32
    ) -> Payout {
        assert_one_yocto();

        // Transfer
        let previous_token = self.nft_token(token_id.clone()).expect("no token");
        self.tokens.nft_transfer(receiver_id.clone(), token_id.clone(), Some(approval_id.into()), None);

        // Payout calculation
        let owner_id = previous_token.owner_id;
        let mut total_perpetual = 0;
        let payout = {
            let balance_u128: u128 = balance.into();
            let mut payout: Payout = HashMap::new();

            let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
            let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
            let royalty = self.token_series_by_id.get(&token_series_id).expect("no type").royalty;

            assert!(royalty.len() as u32 <= max_len_payout, "Market cannot payout to that many receivers");
            for (k, v) in royalty.iter() {
                let key = k.clone();
                if key != owner_id {
                    payout.insert(key, royalty_to_payout(*v, balance_u128));
                    total_perpetual += *v;
                }
            }

            assert!(
                total_perpetual <= 10000,
                "Total payout overflow"
            );

            payout.insert(owner_id.clone(), royalty_to_payout(10000 - total_perpetual, balance_u128));
            payout
        }; 
        env::log(
            json!({
                "type": "nft_transfer",
                "params": {
                    "token_id": token_id,
                    "sender_id": owner_id,
                    "receiver_id": receiver_id,
                }
            })
            .to_string()
            .as_bytes(),
        );
        payout
    }

    pub fn get_owner(&self) -> AccountId {
        self.tokens.owner_id.clone()
    }

    pub fn nft_migrate(&self, receiver_id: ValidAccountId) -> AccountId {
        receiver_id.into()
    }
}

fn royalty_to_payout(a: u32, b: Balance) -> U128 {
    U128(a as u128 * b / 10_000u128)
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env};
    
    const STORAGE_FOR_CREATE_SERIES: Balance = 8540000000000000000000;
    const STORAGE_FOR_MINT: Balance = 11280000000000000000000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn setup_contract() -> (VMContextBuilder, Contract) {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        let contract = Contract::new_default_meta(accounts(0), accounts(4));
        (context, contract)
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new(
            accounts(1),
            accounts(4),
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Triple Triad".to_string(),
                symbol: "TRIAD".to_string(),
                icon: Some(DATA_IMAGE_SVG_PARAS_ICON.to_string()),
                base_uri: Some("https://ipfs.fleek.co/ipfs/".to_string()),
                reference: None,
                reference_hash: None,
            }
        );
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.get_owner(), accounts(1).to_string());
        assert_eq!(contract.nft_metadata().base_uri.unwrap(), "https://ipfs.fleek.co/ipfs/".to_string());
        assert_eq!(contract.nft_metadata().icon.unwrap(), DATA_IMAGE_SVG_PARAS_ICON.to_string());
    }

    fn create_series(
        contract: &mut Contract, 
        token_series_id: U128, 
        royalty: &HashMap<AccountId, u32>,
        price: Option<U128>,
    ) {
        contract.nft_create_series(
            token_series_id,
            TokenMetadata {
                title: Some("Tsundere land".to_string()),
                description: None,
                media: Some(
                    "bafybeidzcan4nzcz7sczs4yzyxly4galgygnbjewipj6haco4kffoqpkiy".to_string()
                ),
                media_hash: None,
                copies: Some(10),
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: Some(
                    "bafybeicg4ss7qh5odijfn2eogizuxkrdh3zlv4eftcmgnljwu7dm64uwji".to_string()
                ),
                reference_hash: None,
            },
            accounts(1),
            price,
            Some(royalty.clone()),
        );
    }

    #[test]
    fn test_create_series() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);
        create_series(&mut contract, U128::from(1), &royalty, Some(U128::from(1 * 10u128.pow(24))));

        let nft_series_return = contract.nft_get_series_single("1".to_string());
        assert_eq!(
            nft_series_return.creator_id,
            accounts(1).to_string()
        );

        assert_eq!(
            nft_series_return.token_series_id,
            "1",
        );

        assert_eq!(
            nft_series_return.royalty,
            royalty,
        );

        assert_eq!(
            nft_series_return.metadata.copies.unwrap(),
            10
        );

        assert_eq!(
            nft_series_return.metadata.title.unwrap(),
            "Tsundere land".to_string()
        );

        assert_eq!(
            nft_series_return.metadata.reference.unwrap(),
            "bafybeicg4ss7qh5odijfn2eogizuxkrdh3zlv4eftcmgnljwu7dm64uwji".to_string()
        );
    }

    #[test]
    #[should_panic(expected = "Paras: duplicate token_series_id")]
    fn test_invalid_duplicate_token_series() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);

        create_series(&mut contract, U128::from(1), &royalty, Some(U128::from(1 * 10u128.pow(24))));
        create_series(&mut contract, U128::from(1), &royalty, Some(U128::from(1 * 10u128.pow(24))));
    }

    #[test]
    fn test_buy() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);

        create_series(&mut contract, U128::from(1), &royalty, Some(U128::from(1 * 10u128.pow(24))));

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(1 * 10u128.pow(24) + STORAGE_FOR_MINT)
            .build()
        );

        let token_id = contract.nft_buy("1".to_string(), accounts(2));

        let token_from_nft_token = contract.nft_token(token_id);
        assert_eq!(
            token_from_nft_token.unwrap().owner_id,
            accounts(2).to_string()
        )
    }

    #[test]
    fn test_mint() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);

        create_series(&mut contract, U128::from(1), &royalty, None);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(STORAGE_FOR_MINT)
            .build()
        );

        let token_id = contract.nft_mint("1".to_string(), accounts(2), U64::from(1));

        let token_from_nft_token = contract.nft_token(token_id);
        assert_eq!(
            token_from_nft_token.unwrap().owner_id,
            accounts(2).to_string()
        )
    }

    #[test]
    #[should_panic(expected = "Paras: Token series is not mintable")]
    fn test_invalid_mint_non_mintable() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);

        create_series(&mut contract, U128::from(1), &royalty, None);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(1)
            .build()
        );
        contract.nft_set_series_non_mintable("1".to_string());

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(STORAGE_FOR_MINT)
            .build()
        );

        contract.nft_mint("1".to_string(), accounts(2), U64::from(1));
    }

    #[test]
    #[should_panic( expected = "Paras: not for sale" )]
    fn test_invalid_buy_price_null() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);

        create_series(&mut contract, U128::from(1), &royalty, None);

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(1 * 10u128.pow(24) + STORAGE_FOR_MINT)
            .build()
        );

        let token_id = contract.nft_buy("1".to_string(), accounts(2));

        let token_from_nft_token = contract.nft_token(token_id);
        assert_eq!(
            token_from_nft_token.unwrap().owner_id,
            accounts(2).to_string()
        )
    }

    #[test]
    fn test_nft_burn() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);

        create_series(&mut contract, U128::from(1), &royalty, None);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(STORAGE_FOR_MINT)
            .build()
        );

        let token_id = contract.nft_mint("1".to_string(), accounts(2), U64::from(1));

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(1)
            .build()
        );

        contract.nft_burn(token_id.clone());
        let token = contract.nft_token(token_id);
        assert!(token.is_none());
    }

    #[test]
    fn test_nft_transfer() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);

        create_series(&mut contract, U128::from(1), &royalty, None);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(STORAGE_FOR_MINT)
            .build()
        );

        let token_id = contract.nft_mint("1".to_string(), accounts(2), U64::from(1));

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(1)
            .build()
        );

        contract.nft_transfer(accounts(3), token_id.clone(), None, None);

        let token = contract.nft_token(token_id).unwrap();
        assert_eq!(
            token.owner_id,
            accounts(3).to_string()
        )
    }

    #[test]
    fn test_nft_transfer_payout() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(STORAGE_FOR_CREATE_SERIES)
            .build()
        );

        let mut royalty: HashMap<AccountId, u32> = HashMap::new();
        royalty.insert(accounts(1).to_string(), 1000);

        create_series(&mut contract, U128::from(1), &royalty, None);

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(STORAGE_FOR_MINT)
            .build()
        );

        let token_id = contract.nft_mint("1".to_string(), accounts(2), U64::from(1));

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(1)
            .build()
        );

        let payout = contract.nft_transfer_payout(
            accounts(3), 
            token_id.clone(), 
            U64::from(0), 
            U128::from(1 * 10u128.pow(24)),
            10
        );

        let mut payout_calc: HashMap<AccountId, U128> = HashMap::new();
        payout_calc.insert(
            accounts(1).to_string(), 
            U128::from((1000 * (1 * 10u128.pow(24)))/10_000)
        );
        payout_calc.insert(
            accounts(2).to_string(), 
            U128::from((9000 * (1 * 10u128.pow(24))) / 10_000)
        );

        assert_eq!(payout, payout_calc);

        let token = contract.nft_token(token_id).unwrap();
        assert_eq!(
            token.owner_id,
            accounts(3).to_string()
        )
    }

    #[test]
    fn test_change_metadata() {
        let (mut context, mut contract) = setup_contract();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .build()
        );
        contract.change_metadata(
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Paras-1".to_string(),
                symbol: "PARAS-1".to_string(),
                icon: Some(DATA_IMAGE_SVG_PARAS_ICON.to_string()),
                base_uri: Some("https://ipfs.fleek.co/ipfs/random".to_string()),
                reference: None,
                reference_hash: None,
            },
        );

        let metadata_cur = contract.nft_metadata();

        assert_eq!(metadata_cur.name, "Paras-1".to_string());
        assert_eq!(metadata_cur.symbol, "PARAS-1".to_string());
        assert_eq!(metadata_cur.base_uri, Some("https://ipfs.fleek.co/ipfs/random".to_string()));
    }
}