mod deposit;
mod misc;
mod order;
mod trade;
mod withdrawal;

pub use deposit::*;
pub use misc::*;
pub use order::*;
pub use trade::*;
pub use withdrawal::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::util::test_util::*;
    use crate::Credentials;
    use surf::Client as HTTPClient;
    use surf_vcr::VcrMode;

    async fn create_client(cassette: &'static str) -> HTTPClient {
        let mut path_builder = test_resource_path();
        path_builder.push("rest");
        path_builder.push("auth");
        path_builder.push(cassette);
        create_test_recording_client(VcrMode::Replay, path_builder.as_path().to_str().unwrap())
            .await
    }

    #[async_std::test]
    async fn auth_success() {
        let params = GetProfile {};
        let resp = create_client("auth_success.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetProfile::read_response(resp.into()).await;
        assert!(result.is_ok())
    }

    #[async_std::test]
    async fn auth_fail() {
        let empty_credentials = Credentials::new(String::new(), String::new());
        let params = GetProfile {};
        let resp = create_client("auth_fail.yaml")
            .await
            .send(params.to_request(&empty_credentials))
            .await
            .expect("Error while sending request");
        if let Err(Error::RestApi(code, msg)) = GetProfile::read_response(resp.into()).await {
            assert_eq!(code, 2008);
            assert_eq!(msg, String::from("The access key does not exist."));
        } else {
            panic!("Authentication must fail with empty credentials.");
        }
    }
}
