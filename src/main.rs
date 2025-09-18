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
                Err(err) => {
                    eprintln!("octocrab error: {}", err);

                    match bot.send_message(msg.chat.id, "Internal error").await {
                        Err(err) => {
                            eprintln!("teloxide error: {}", err);
                        }
                        Ok(_) => {}
                    }
                }
                Ok(github) => match github.current().user().await {
                    Err(err) => {
                        eprintln!("octocrab error: {}", err);

                        match bot.send_message(msg.chat.id, "Invalid token").await {
                            Err(err) => {
                                eprintln!("teloxide error: {}", err);
                            }
                            Ok(_) => {}
                        }
                    }
                    Ok(user) => {
                        match bot
                            .send_message(
                                msg.chat.id,
                                format!("Connected to GitHub as {}", user.login),
                            )
                            .await
                        {
                            Err(err) => {
                                eprintln!("teloxide error: {}", err);
                            }
                            Ok(_) => {}
                        }
                    }
                },
            }
        } else {
            match bot
                .send_message(
                    msg.chat.id,
                    format!(
                        "Getting started:

1. Navigate to https://github.com/settings/personal-access-tokens/new
2. Set expiration: not greater than 366 days
3. Read-only access to public repositories is sufficient
4. Generate the token and paste it here ({}...)
5. I will automatically notify you about merge conflicts in your PRs",
                        ghpat
                    ),
                )
                .await
            {
                Err(err) => {
                    eprintln!("teloxide error: {}", err);
                }
                Ok(_) => {}
            }
        }

        Ok(())
    })
    .await;

    eprintln!("Event loop terminated");
    ExitCode::FAILURE
}
