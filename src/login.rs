use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Response from discord when the login is successful
#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub token: String,
    // Also "user_settings"
}

/// Response from discord when the login requires a 2FA
#[derive(Debug, Deserialize)]
pub struct Login2FA {
    pub user_id: String,
    pub ticket: String,
    pub mfa: bool,
    pub sms: bool,
    pub backup: bool,
    pub totp: bool,
    pub webauthn: Option<String>,
}

/// Represents the state of the login
#[derive(Debug)]
pub enum LoginResult {
    Ok(LoginResponse),
    Requires2FA(Login2FA),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Login failed")]
    UserLoginFailed,

    #[error("The token \"{0}\" is invalid")]
    InvalidTOTP(String),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

//TODO: convert convert errors to `LoginResult::Failed`
pub async fn user_login(
    client: &Client,
    login: &str,
    password: &str,
) -> Result<LoginResult, Error> {
    let body = {
        #[derive(Debug, Serialize)]
        struct UserLogin<'a> {
            login: &'a str,
            password: &'a str,
        }

        UserLogin { login, password }
    };
    let res = client
        .post(crate::endpoint("/auth/login"))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body)?)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(Error::UserLoginFailed);
    }

    let data = res.text().await?;
    match serde_json::from_str(&data) {
        Ok(json) => Ok(LoginResult::Ok(json)),
        Err(_) => Ok(LoginResult::Requires2FA(serde_json::from_str(&data)?)),
    }
}

pub async fn totp_login(
    client: &Client,
    code: &str,
    login_2fa: &Login2FA,
) -> Result<LoginResponse, Error> {
    let body = {
        #[derive(Debug, Serialize)]
        struct TotpLoginData<'a> {
            code: &'a str,
            ticket: &'a str,
        }

        TotpLoginData {
            code,
            ticket: &login_2fa.ticket,
        }
    };
    let res = client
        .post(crate::endpoint("/auth/mfa/totp"))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body)?)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(Error::InvalidTOTP(code.to_string()));
    }

    let token = {
        #[derive(Debug, Deserialize)]
        struct TotpLoginResponse {
            token: String,
        }

        serde_json::from_str::<TotpLoginResponse>(&res.text().await?)?.token
    };

    Ok(LoginResponse {
        user_id: login_2fa.user_id.clone(),
        token,
    })
}
