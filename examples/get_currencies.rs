//! Fetch the detail of supported currencies from RESTful API.
//!
//! ```bash
//! cargo run --example get_currencies
//! ```

use maicoin_max::v2::rest::{CurrencyInfo, GetCurrencies};

#[async_std::main]
async fn main() -> Result<(), http_types::Error> {
    let client = surf::Client::new();
    let params = GetCurrencies {};
    let resp = client
        .send(params.to_request())
        .await
        .expect("Error while sending request");
    let result = GetCurrencies::read_response(resp.into()).await;
    let currencies: Vec<CurrencyInfo> = result.expect("failed to parse result");

    println!("List of support currencies:");
    for info in currencies {
        println!("- {}", info.id);
    }
    Ok(())
}
