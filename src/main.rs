use octocrab::Octocrab;
use std::env::var_os;
use std::process::ExitCode;
use teloxide::prelude::Requester;
use teloxide::types::Message;
use teloxide::{Bot, repl};

#[tokio::main]
async fn main() -> ExitCode {
    let envvar = "MERGECONFLICT_TGBOT_TGTOKEN";

    let tg_token = match var_os(envvar) {
        None => {
            eprintln!("Env var {} missing", envvar);
            return ExitCode::FAILURE;
        }
        Some(oss_token) => {
            if oss_token.is_empty() {
                eprintln!("Env var {} is empty", envvar);
                return ExitCode::FAILURE;
            } else {
                match String::from_utf8(oss_token.into_encoded_bytes()) {
                    Err(err) => {
                        eprintln!("Env var {} is invalid UTF-8: {}", envvar, err);
                        return ExitCode::FAILURE;
                    }
                    Ok(s_token) => s_token,
                }
            }
        }
    };

    let tg_bot = Bot::new(tg_token);

    repl(tg_bot, |bot: Bot, msg: Message| async move {
        let txt = msg.text().unwrap_or("");
        let ghpat = "github_pat_";

        if txt.starts_with(ghpat) {
            match Octocrab::builder().personal_token(txt).build() {
                Err(_) => {
                    let _ = bot.send_message(msg.chat.id, "Internal error").await;
                }
                Ok(github) => match github.current().user().await {
                    Err(_) => {
                        let _ = bot.send_message(msg.chat.id, "Invalid token").await;
                    }
                    Ok(user) => {
                        let _ = bot.send_message(msg.chat.id, format!("Connected to GitHub as {}", user.login)).await;
                    }
                }
            }
        } else {
            let _ = bot.send_message(msg.chat.id, format!(
                "Getting started:\n\n1. Navigate to https://github.com/settings/personal-access-tokens/new\n2. Set expiration: not greater than 366 days\n3. Read-only access to public repositories is sufficient\n4. Generate the token and paste it here ({}...)\n5. I will automatically notify you about merge conflicts in your PRs",
                ghpat
            )).await;
        }

        Ok(())
    })
    .await;

    eprintln!("Event loop terminated");
    ExitCode::FAILURE
}
