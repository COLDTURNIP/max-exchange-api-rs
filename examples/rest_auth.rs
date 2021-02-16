//! Get personal remaining balance of given currency.
//!
//! ```bash
//! cargo run --example rest_auth <currency> <api_key> <api_secret>
//! ```

use maicoin_max::v2::rest::{GetAccountOfCurrency, RespAccountCurrencyInfo};
use maicoin_max::Credentials;

#[async_std::main]
async fn main() -> Result<(), http_types::Error> {
    let mut args = std::env::args().collect::<Vec<String>>().into_iter();
    args.next();
    let currency = args.next().unwrap();
    let api_key = args.next().unwrap();
    let api_secret = args.next().unwrap();

    let credentials = Credentials::new(api_key, api_secret);

    let client = surf::Client::new();
    let params = GetAccountOfCurrency {
        path_currency: currency.clone(),
    };
    let resp = client
        .send(params.to_request(&credentials))
        .await
        .expect("Error while sending request");
    let result = GetAccountOfCurrency::read_response(resp.into()).await;
    let info: RespAccountCurrencyInfo = result.expect("failed to parse result");

    println!("My {} balance is {}", currency, info.balance);
    Ok(())
}
