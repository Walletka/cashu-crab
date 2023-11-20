//! Client to connet to mint

use async_trait::async_trait;
#[cfg(feature = "mint")]
use cashu::nuts::nut00;
#[cfg(feature = "nut07")]
use cashu::nuts::CheckSpendableResponse;
#[cfg(feature = "nut09")]
use cashu::nuts::MintInfo;
use cashu::nuts::{
    BlindedMessage, BlindedMessages, CheckFeesResponse, Keys, KeysetResponse, MeltResponse,
    PostMintResponse, Proof, RequestMintResponse, SplitRequest, SplitResponse,
};
use cashu::{utils, Amount};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[cfg(feature = "gloo")]
pub mod gloo_client;
#[cfg(not(target_arch = "wasm32"))]
pub mod minreq_client;

pub use cashu::Bolt11Invoice;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invoice not paid")]
    InvoiceNotPaid,
    #[error("Wallet not responding")]
    LightingWalletNotResponding(Option<String>),
    /// Parse Url Error
    #[error("`{0}`")]
    UrlParse(#[from] url::ParseError),
    /// Serde Json error
    #[error("`{0}`")]
    SerdeJson(#[from] serde_json::Error),
    /// Cashu Url Error
    #[error("`{0}`")]
    CashuUrl(#[from] cashu::url::Error),
    ///  Min req error
    #[cfg(not(target_arch = "wasm32"))]
    #[error("`{0}`")]
    MinReq(#[from] minreq::Error),
    #[cfg(feature = "gloo")]
    #[error("`{0}`")]
    Gloo(String),
    /// Custom Error
    #[error("`{0}`")]
    Custom(String),
}

impl Error {
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let mint_res: MintErrorResponse = serde_json::from_str(json)?;

        let err = mint_res
            .error
            .as_deref()
            .or(mint_res.detail.as_deref())
            .unwrap_or_default();

        let mint_error = match err {
            error if error.starts_with("Lightning invoice not paid yet.") => Error::InvoiceNotPaid,
            error if error.starts_with("Lightning wallet not responding") => {
                let mint = utils::extract_url_from_error(error);
                Error::LightingWalletNotResponding(mint)
            }
            error => Error::Custom(error.to_owned()),
        };
        Ok(mint_error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MintErrorResponse {
    code: u32,
    error: Option<String>,
    detail: Option<String>,
}

#[async_trait(?Send)]
pub trait Client {
    async fn get_mint_keys(&self, mint_url: Url) -> Result<Keys, Error>;

    async fn get_mint_keysets(&self, mint_url: Url) -> Result<KeysetResponse, Error>;

    async fn get_request_mint(
        &self,
        mint_url: Url,
        amount: Amount,
    ) -> Result<RequestMintResponse, Error>;

    // TODO: Hash should have a type
    async fn post_mint(
        &self,
        mint_url: Url,
        blinded_messages: BlindedMessages,
        hash: &str,
    ) -> Result<PostMintResponse, Error>;

    async fn post_check_fees(
        &self,
        mint_url: Url,
        invoice: Bolt11Invoice,
    ) -> Result<CheckFeesResponse, Error>;

    async fn post_melt(
        &self,
        mint_url: Url,
        proofs: Vec<Proof>,
        invoice: Bolt11Invoice,
        outputs: Option<Vec<BlindedMessage>>,
    ) -> Result<MeltResponse, Error>;

    // REVIEW: Should be consistent aboue passing in the Request struct or the
    // compnatants and making it within the function. Here the struct is passed
    // in but in check spendable and melt the compants are passed in
    async fn post_split(
        &self,
        mint_url: Url,
        split_request: SplitRequest,
    ) -> Result<SplitResponse, Error>;

    #[cfg(feature = "nut07")]
    async fn post_check_spendable(
        &self,
        mint_url: Url,
        proofs: Vec<nut00::mint::Proof>,
    ) -> Result<CheckSpendableResponse, Error>;

    #[cfg(feature = "nut09")]
    async fn get_mint_info(&self, mint_url: Url) -> Result<MintInfo, Error>;
}

#[cfg(any(not(target_arch = "wasm32"), feature = "gloo"))]
fn join_url(url: Url, path: &str) -> Result<Url, Error> {
    let mut url = url;
    if !url.path().ends_with('/') {
        url.path_segments_mut()
            .map_err(|_| Error::Custom("Url Path Segmants".to_string()))?
            .push(path);
    } else {
        url.path_segments_mut()
            .map_err(|_| Error::Custom("Url Path Segmants".to_string()))?
            .pop()
            .push(path);
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_error() {
        let err = r#"{"code":0,"error":"Lightning invoice not paid yet."}"#;

        let error = Error::from_json(err).unwrap();

        match error {
            Error::InvoiceNotPaid => {}
            _ => panic!("Wrong error"),
        }

        let err = r#"{"code": 0, "error": "Lightning wallet not responding: Failed to connect to https://legend.lnbits.com due to: All connection attempts failed"}"#;
        let error = Error::from_json(err).unwrap();
        match error {
            Error::LightingWalletNotResponding(mint) => {
                assert_eq!(mint, Some("https://legend.lnbits.com".to_string()));
            }
            _ => panic!("Wrong error"),
        }
    }
}
