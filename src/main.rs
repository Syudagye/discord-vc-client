use dialoguer::{Input, Password};
use login::LoginResult;

use crate::discord::{guilds, login};

mod discord;

async fn login(client: &reqwest::Client) -> Result<login::LoginResponse, anyhow::Error> {
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

    Ok(match login {
        LoginResult::Ok(data) => data,
        LoginResult::Requires2FA(login_2fa) => {
            if !login_2fa.totp {
                println!("2fa needed but unavailable. (what ?)");
                std::process::exit(1);
            }

            loop {
                let code: String = Input::new()
                    .with_prompt("Enter 2FA TOTP Code")
                    .interact_text()?;

                let res = login::totp_login(&client, &code, &login_2fa).await;
                match res {
                    Ok(login) => break login,
                    Err(e) => {
                        eprintln!("Error occured: {}", e);
                        eprintln!("Try again");
                    }
                }
            }
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    let client = reqwest::Client::new();

    let login = match (
        std::env::var("DISCORD_TOKEN"),
        std::env::var("DISCORD_USERID"),
    ) {
        (Ok(token), Ok(user_id)) => login::LoginResponse { token, user_id },
        _ => login(&client).await?,
    };

    println!("{:?}", login);

    let guilds = guilds::list_guilds(&client, &login.token).await?;

    println!("You are in {} guilds", guilds.len());

    for guild in guilds {
        let channels = guilds::list_vc_channels(&client, &login.token, &guild).await?;
        println!("VC Channels for {}:", guild.name);
        println!("{channels:?}");
    }

    Ok(())
}
