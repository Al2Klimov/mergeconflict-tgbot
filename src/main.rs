use octocrab::Octocrab;
use std::env::var_os;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, Message};
use teloxide::{Bot, repl};
use tokio::spawn;
use tokio::time::sleep;

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

    let dbname = "mergeconflict-tgbot.sqlite";

    let db = match sqlite::open(dbname) {
        Err(err) => {
            eprintln!("Failed to open database {}: {}", dbname, err);
            return ExitCode::FAILURE;
        }
        Ok(conn) => conn,
    };

    match db.execute(
        "
CREATE TABLE IF NOT EXISTS chat (
    id INTEGER PRIMARY KEY,
    ghpat TEXT NOT NULL,
    last_scan INTEGER NOT NULL DEFAULT -1
);
",
    ) {
        Err(err) => {
            eprintln!("Failed to initialize database: {}", err);
            return ExitCode::FAILURE;
        }
        Ok(_) => {}
    }

    let db = Arc::new(Mutex::new(db));
    let tg_bot = Bot::new(tg_token);

    spawn({
        let db = Arc::clone(&db);
        let tg_bot = tg_bot.clone();
        async move {
            loop {
                let mut due = vec![];

                match db.lock().unwrap().prepare("SELECT id, ghpat FROM chat WHERE last_scan < unixepoch() - 2 ORDER BY last_scan LIMIT 69") {
                    Err(err) => {
                        eprintln!("sqlite error: {}", err);
                    }
                    Ok(query) => {
                        for row in query {
                            match row {
                                Err(err) => {
                                    eprintln!("sqlite error: {}", err);
                                    due.clear();
                                }
                                Ok(row) => {
                                    due.push((row.read::<i64,_>("id"), row.read::<&str,_>("ghpat").to_owned()));
                                }
                            }
                        }
                    }
                }

                for (id, ghpat) in due {
                    match Octocrab::builder().personal_token(ghpat).build() {
                        Err(err) => {
                            eprintln!("octocrab error: {}", err);
                        }
                        Ok(github) => match github.current().user().await {
                            Err(err) => {
                                eprintln!("octocrab error: {}", err);

                                match tg_bot
                                    .send_message(
                                        ChatId(id),
                                        "Your token expired, please provide a new one",
                                    )
                                    .await
                                {
                                    Err(err) => {
                                        eprintln!("teloxide error: {}", err);

                                        match db.lock().unwrap().prepare("UPDATE chat SET last_scan = unixepoch() WHERE id = ?").and_then(|mut stmt| stmt.bind((1, id)).and_then(|_| stmt.next())) {
                                            Err(err) => {
                                                eprintln!("sqlite error: {}", err);
                                            }
                                            Ok(_) => {}
                                        }
                                    }
                                    Ok(_) => match db
                                        .lock()
                                        .unwrap()
                                        .prepare("DELETE FROM chat WHERE id = ?")
                                        .and_then(|mut stmt| {
                                            stmt.bind((1, id)).and_then(|_| stmt.next())
                                        }) {
                                        Err(err) => {
                                            eprintln!("sqlite error: {}", err);
                                        }
                                        Ok(_) => {}
                                    },
                                }
                            }
                            Ok(user) => {
                                // TODO
                                eprintln!("Scanned {}", user.login);

                                match db
                                    .lock()
                                    .unwrap()
                                    .prepare("UPDATE chat SET last_scan = unixepoch() WHERE id = ?")
                                    .and_then(|mut stmt| {
                                        stmt.bind((1, id)).and_then(|_| stmt.next())
                                    }) {
                                    Err(err) => {
                                        eprintln!("sqlite error: {}", err);
                                    }
                                    Ok(_) => {}
                                }
                            }
                        },
                    }
                }

                sleep(Duration::from_secs(1)).await;
            }
        }
    });

    repl(tg_bot, move |bot: Bot, msg: Message| {
        let db = Arc::clone(&db);
        async move {
            let txt = msg.text().unwrap_or("");
            let ghpat = "github_pat_";

            if txt.starts_with(ghpat) {
                let mut int_err = false;

                match Octocrab::builder().personal_token(txt).build() {
                    Err(err) => {
                        eprintln!("octocrab error: {}", err);
                        int_err = true;
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
                            match db
                                .lock()
                                .unwrap()
                                .prepare("REPLACE INTO chat(id, ghpat) VALUES (?, ?)")
                                .and_then(|mut stmt| {
                                    stmt.bind((1, msg.chat.id.0))
                                        .and_then(|_| stmt.bind((2, txt)))
                                        .and_then(|_| stmt.next())
                                }) {
                                Err(err) => {
                                    eprintln!("sqlite error: {}", err);
                                    int_err = true;
                                }
                                Ok(_) => {}
                            }

                            if !int_err {
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
                        }
                    },
                }

                if int_err {
                    match bot.send_message(msg.chat.id, "Internal error").await {
                        Err(err) => {
                            eprintln!("teloxide error: {}", err);
                        }
                        Ok(_) => {}
                    }
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
        }
    })
    .await;

    eprintln!("Event loop terminated");
    ExitCode::FAILURE
}
