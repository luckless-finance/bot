use crate::errors::{AccountClientError, GenResult};
use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;

const PROTOCOL: &str = "http";
const HOST: &str = "localhost";
const PORT: &str = "3010";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Account {
    name: String,
    id: String,
}

impl Account {
    pub fn new(name: &str, id: &str) -> Self {
        Account {
            name: name.to_string(),
            id: id.to_string(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AccountRequest {
    name: String,
}

impl AccountRequest {
    pub fn new(name: &str) -> Self {
        AccountRequest {
            name: name.to_string(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Asset {
    currency: String,
    symbol: String,
}

impl Asset {
    pub fn new(currency: &str, symbol: &str) -> Self {
        Asset {
            currency: currency.to_string(),
            symbol: symbol.to_string(),
        }
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum OrderStatus {
    COMPLETE,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Order {
    id: String,
    order_status: OrderStatus,
    asset: Asset,
    cash_flow: f64,
    quantity: f64,
    timestamp: String,
}

impl Order {
    pub fn new(
        id: String,
        order_status: OrderStatus,
        asset: Asset,
        cash_flow: f64,
        quantity: f64,
        timestamp: String,
    ) -> Self {
        Order {
            id,
            order_status,
            asset,
            cash_flow,
            quantity,
            timestamp,
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn order_status(&self) -> &OrderStatus {
        &self.order_status
    }
    pub fn asset(&self) -> &Asset {
        &self.asset
    }
    pub fn cash_flow(&self) -> f64 {
        self.cash_flow
    }
    pub fn quantity(&self) -> f64 {
        self.quantity
    }
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrderRequest {
    asset: Asset,
    cash_flow: f64,
    quantity: f64,
    timestamp: String,
}

impl OrderRequest {
    pub fn new(asset: Asset, cash_flow: f64, quantity: f64, timestamp: &str) -> Self {
        OrderRequest {
            asset,
            cash_flow,
            quantity,
            timestamp: timestamp.to_string(),
        }
    }
    pub fn asset(&self) -> &Asset {
        &self.asset
    }
    pub fn cash_flow(&self) -> f64 {
        self.cash_flow
    }
    pub fn quantity(&self) -> f64 {
        self.quantity
    }
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

pub fn create_account(name: &str) -> GenResult<Account> {
    let account = AccountRequest::new(name);
    let url = format!("{}://{}:{}/accounts", PROTOCOL, HOST, PORT);
    let client = reqwest::blocking::Client::new();
    let res = client.post(url).json(&account).send();
    match res {
        Err(e) => match e.status() {
            Some(status) => Err(AccountClientError::new(format!(
                "{} - Failed to create account.",
                status.canonical_reason().unwrap()
            ))),
            None => Err(AccountClientError::new(String::from(
                "Failed to create account. Is Account Server up?",
            ))),
        },
        Ok(response) => match response.status() {
            StatusCode::OK => match response.json::<Account>() {
                Ok(account_response) => Ok(account_response),
                Err(_) => Err(AccountClientError::new(format!(
                    "Failed to serialize Account response."
                ))),
            },
            _ => Err(AccountClientError::new(format!(
                "{} - Failed to create account.",
                response.status().canonical_reason().unwrap()
            ))),
        },
    }
}

pub fn create_order(
    account: Account,
    asset: Asset,
    cash_flow: f64,
    quantity: f64,
    timestamp: &str,
) -> GenResult<Order> {
    let order = OrderRequest::new(asset, cash_flow, quantity, timestamp);
    let url = format!(
        "{}://{}:{}/accounts/{}/orders",
        PROTOCOL,
        HOST,
        PORT,
        account.id()
    );
    let client = reqwest::blocking::Client::new();
    let res = client.post(url).json(&order).send();
    match res {
        Err(e) => match e.status() {
            Some(status) => Err(AccountClientError::new(format!(
                "{} - Failed to create order.",
                status.canonical_reason().unwrap()
            ))),
            None => Err(AccountClientError::new(String::from(
                "Failed to create order. Is Account Server up?",
            ))),
        },
        Ok(response) => match response.status() {
            StatusCode::OK => match response.json::<Order>() {
                Ok(order_response) => Ok(order_response),
                Err(_) => Err(AccountClientError::new(format!(
                    "Failed to serialize Order response."
                ))),
            },
            _ => Err(AccountClientError::new(format!(
                "{} - Failed to create order.",
                response.status().canonical_reason().unwrap()
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use chrono::Duration;

    use crate::errors::GenResult;
    use crate::rest_client::{create_account, create_order, Asset};
    use crate::time_series::TimeSeries1D;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use std::error::Error;

    fn unique_id() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    // #[test]
    // fn create_account_test() -> GenResult<()> {
    //     let account = create_account("new account")?;
    //     assert_eq!(account.name, "new account");
    //
    //     let account = create_account("new account");
    //     assert!(account.is_err());
    //     Ok(())
    // }
    //
    // #[test]
    // fn create_order_test() -> GenResult<()> {
    //     let account = create_account(unique_id().as_str())?;
    //     let asset = Asset::new("USD", "FOO");
    //     let order = create_order(account, asset, 0.0, 0.0, "1234")?;
    //     Ok(())
    // }
}
