//! Minreq http Client

use async_trait::async_trait;
use cashu::nuts::{
    BlindedMessage, CurrencyUnit, Id, KeySet, KeysResponse, KeysetResponse, MeltBolt11Request,
    MeltBolt11Response, MeltQuoteBolt11Request, MeltQuoteBolt11Response, MintBolt11Request,
    MintBolt11Response, MintInfo, MintQuoteBolt11Request, MintQuoteBolt11Response, PreMintSecrets,
    Proof, SwapRequest, SwapResponse,
};
#[cfg(feature = "nut07")]
use cashu::nuts::{CheckStateRequest, CheckStateResponse};
#[cfg(feature = "nut07")]
use cashu::secret::Secret;
use cashu::{Amount, Bolt11Invoice};
use serde_json::Value;
use tracing::{error, warn};
use url::Url;

use super::join_url;
use crate::client::{Client, Error};

#[derive(Debug, Clone)]
pub struct HttpClient {}

#[async_trait(?Send)]
impl Client for HttpClient {
    /// Get Mint Keys [NUT-01]
    async fn get_mint_keys(&self, mint_url: Url) -> Result<Vec<KeySet>, Error> {
        let url = join_url(mint_url, &["v1", "keys"])?;
        let keys = minreq::get(url).send()?.json::<Value>()?;

        let keys: KeysResponse = serde_json::from_str(&keys.to_string())?;
        Ok(keys.keysets)
    }

    /// Get Keyset Keys [NUT-01]
    async fn get_mint_keyset(&self, mint_url: Url, keyset_id: Id) -> Result<KeySet, Error> {
        let url = join_url(mint_url, &["v1", "keys", &keyset_id.to_string()])?;
        let keys = minreq::get(url).send()?.json::<KeysResponse>()?;

        // let keys: KeysResponse = serde_json::from_value(keys)?; //
        // serde_json::from_str(&keys.to_string())?;
        Ok(keys.keysets[0].clone())
    }

    /// Get Keysets [NUT-02]
    async fn get_mint_keysets(&self, mint_url: Url) -> Result<KeysetResponse, Error> {
        let url = join_url(mint_url, &["v1", "keysets"])?;
        let res = minreq::get(url).send()?.json::<Value>()?;

        let response: Result<KeysetResponse, serde_json::Error> =
            serde_json::from_value(res.clone());

        match response {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::from_json(&res.to_string())?),
        }
    }

    /// Mint Quote [NUT-04]
    async fn post_mint_quote(
        &self,
        mint_url: Url,
        amount: Amount,
        unit: CurrencyUnit,
    ) -> Result<MintQuoteBolt11Response, Error> {
        let url = join_url(mint_url, &["v1", "mint", "quote", "bolt11"])?;

        let request = MintQuoteBolt11Request { amount, unit };

        let res = minreq::post(url).with_json(&request)?.send()?;

        let response: Result<MintQuoteBolt11Response, serde_json::Error> =
            serde_json::from_value(res.json()?);

        match response {
            Ok(res) => Ok(res),
            Err(_) => {
                warn!("Bolt11 Mint Quote Unexpected response: {:?}", res);
                Err(Error::from_json(&res.status_code.to_string())?)
            }
        }
    }

    /// Mint Tokens [NUT-04]
    async fn post_mint(
        &self,
        mint_url: Url,
        quote: &str,
        premint_secrets: PreMintSecrets,
    ) -> Result<MintBolt11Response, Error> {
        let url = join_url(mint_url, &["v1", "mint", "bolt11"])?;

        let request = MintBolt11Request {
            quote: quote.to_string(),
            outputs: premint_secrets.blinded_messages(),
        };

        let res = minreq::post(url)
            .with_json(&request)?
            .send()?
            .json::<Value>()?;

        let response: Result<MintBolt11Response, serde_json::Error> =
            serde_json::from_value(res.clone());

        match response {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::from_json(&res.to_string())?),
        }
    }

    /// Melt Quote [NUT-05]
    async fn post_melt_quote(
        &self,
        mint_url: Url,
        unit: CurrencyUnit,
        request: Bolt11Invoice,
    ) -> Result<MeltQuoteBolt11Response, Error> {
        let url = join_url(mint_url, &["v1", "melt", "quote", "bolt11"])?;

        let request = MeltQuoteBolt11Request { request, unit };

        let value = minreq::post(url)
            .with_json(&request)?
            .send()?
            .json::<Value>()?;

        let response: Result<MeltQuoteBolt11Response, serde_json::Error> =
            serde_json::from_value(value.clone());

        match response {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::from_json(&value.to_string())?),
        }
    }

    /// Melt [NUT-05]
    /// [Nut-08] Lightning fee return if outputs defined
    async fn post_melt(
        &self,
        mint_url: Url,
        quote: String,
        inputs: Vec<Proof>,
        outputs: Option<Vec<BlindedMessage>>,
    ) -> Result<MeltBolt11Response, Error> {
        let url = join_url(mint_url, &["v1", "melt", "bolt11"])?;

        let request = MeltBolt11Request {
            quote,
            inputs,
            outputs,
        };

        let value = minreq::post(url)
            .with_json(&request)?
            .send()?
            .json::<Value>()?;

        let response: Result<MeltBolt11Response, serde_json::Error> =
            serde_json::from_value(value.clone());

        match response {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::from_json(&value.to_string())?),
        }
    }

    /// Split Token [NUT-06]
    async fn post_swap(
        &self,
        mint_url: Url,
        split_request: SwapRequest,
    ) -> Result<SwapResponse, Error> {
        // TODO: Add to endpoint
        let url = join_url(mint_url, &["v1", "swap"])?;

        let res = minreq::post(url).with_json(&split_request)?.send()?;

        let response: Result<SwapResponse, serde_json::Error> =
            serde_json::from_value(res.json::<Value>()?.clone());

        if let Err(err) = &response {
            error!("{}", err)
        }

        Ok(response?)
    }

    /// Spendable check [NUT-07]
    #[cfg(feature = "nut07")]
    async fn post_check_state(
        &self,
        mint_url: Url,
        secrets: Vec<Secret>,
    ) -> Result<CheckStateResponse, Error> {
        let url = join_url(mint_url, &["v1", "checkstate"])?;
        let request = CheckStateRequest { secrets };

        let res = minreq::post(url)
            .with_json(&request)?
            .send()?
            .json::<Value>()?;

        let response: Result<CheckStateResponse, serde_json::Error> =
            serde_json::from_value(res.clone());

        match response {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::from_json(&res.to_string())?),
        }
    }

    /// Get Mint Info [NUT-09]
    async fn get_mint_info(&self, mint_url: Url) -> Result<MintInfo, Error> {
        let url = join_url(mint_url, &["v1", "info"])?;

        let res = minreq::get(url).send()?.json::<Value>()?;

        let response: Result<MintInfo, serde_json::Error> = serde_json::from_value(res.clone());

        match response {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::from_json(&res.to_string())?),
        }
    }
}
