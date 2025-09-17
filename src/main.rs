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
        match msg.text() {
            None => {}
            Some(txt) => {
                bot.send_message(msg.chat.id, txt).await?;
            }
        }

        Ok(())
    })
    .await;

    eprintln!("Event loop terminated");
    ExitCode::FAILURE
}
