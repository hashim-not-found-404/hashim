use my_core::{request_response::*, traits};
use reqwest;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone)]
pub enum Error {
    OtherUnexpectedStatusCode(String),
    SomeInternalErrorOfTheServer,
    Decoding,
    CheckYourWifi,
    ErrorAtSendingRequest(String),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::OtherUnexpectedStatusCode(s) => {
                return String::from("other_unexpected_status_code");
            }
            Self::SomeInternalErrorOfTheServer => {
                return String::from("some_internal_error_of_the_server");
            }
            Self::Decoding => {
                return String::from("decoding");
            }
            Self::CheckYourWifi => {
                return String::from("check_your_wifi");
            }
            Self::ErrorAtSendingRequest(s) => {
                return String::from("error_at_sending_request");
            }
        }
    }
}

pub struct MyClient {
    client: reqwest::Client,
}

impl Default for MyClient {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            // .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        Self { client }
    }
}

impl MyClient {
    async fn send<Input: Serialize, Ok: DeserializeOwned, InvaledInput: DeserializeOwned>(
        &self,
        path: &str,
        message: Input,
    ) -> business_layer::Result<Ok, InvaledInput, Error> {
        let response = self
            .client
            .post(format!("http://{}{}", ADDRESS, path))
            .json(&message)
            .send()
            .await;

        let response = match response {
            Ok(o) => o,
            Err(e) => {
                return Err(Error::ErrorAtSendingRequest(e.to_string()));
            }
        };

        match response.status() {
            reqwest::StatusCode::OK => {
                let response = response
                    .json::<business_layer::Result<Ok, InvaledInput, ()>>()
                    .await;

                match response {
                    Ok(o) => match o {
                        Ok(o) => {
                            return Ok(o);
                        }
                        Err(e) => {
                            return Err(Error::SomeInternalErrorOfTheServer);
                        }
                    },
                    Err(e) => {
                        return Err(Error::Decoding);
                    }
                }
            }
            status_code => {
                return Err(Error::OtherUnexpectedStatusCode(status_code.to_string()));
            }
        }
    }
}

macro_rules! generate_api_backend_methods {
    ($path:ident) => {
        async fn $path(
            &self,
            input: business_layer::Input<$path::Input>,
        ) -> business_layer::Result<$path::Ok, $path::Error, Self::Error> {
            self.send($path::PATH, input).await
        }
    };
}

impl traits::BackendRouts for MyClient {
    type Error = Error;
    generate_api_backend_methods!(sign_up);
    generate_api_backend_methods!(sign_in);
    generate_api_backend_methods!(get_all_user_roles);
    generate_api_backend_methods!(create_company);
    generate_api_backend_methods!(create_company_branch);
}
