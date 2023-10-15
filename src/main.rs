use dialoguer::{Input, Password};
use login::LoginResult;

pub const API_ENDPOINT: &str = "https://discord.com/api/v10";

/// Creates url for a given endpoint
pub fn endpoint(endpoint: &str) -> String {
    format!("{}{}", API_ENDPOINT, endpoint)
}

mod login;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    let client = reqwest::Client::new();

    let login = loop {
        let login: String = Input::new().with_prompt("login").interact_text()?;
        let password = Password::new().with_prompt("password").interact()?;
        println!("Authenticating {login}...");

        match login::user_login(&client, &login, &password).await {
            Ok(login) => break login,
            Err(_) => {
                println!("Login Failed, try again");
            }
        };
    };

    let login = match login {
        LoginResult::Ok(data) => data,
        LoginResult::Requires2FA(login_2fa) => {
            if !login_2fa.totp {
                println!("2fa needed but unavailable. (what ?)");
                std::process::exit(1);
            }

            loop {
                let code: String = Input::new().with_prompt("Enter 2F1 TOTP Code").interact_text()?;

                let res =
                    login::totp_login(&client, &code, &login_2fa).await;
                match res {
                    Ok(login) => break login,
                    Err(e) => {
                        eprintln!("Error occured: {}", e);
                        eprintln!("Try again");
                    }
                }
            }
        }
    };

    println!("{:?}", login);

    Ok(())
}
