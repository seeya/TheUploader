use dotenv::dotenv;
use lazy_static::lazy_static;
use regex::Regex;
use std::io;
use std::path::Path;
use std::time::Duration;
use tdlib::enums::{AuthorizationState, Update};
use tdlib::functions::{self};
use tokio::time::sleep;

async fn handle_login(update: AuthorizationState, client_id: i32) {
    match update {
        AuthorizationState::WaitTdlibParameters => {
            println!("Waiting for TDLib parameters ");

            let api_id = std::env::var("API_ID").expect("WATCH_PATH must be set.");
            let api_id = api_id.parse::<i32>().unwrap();
            let api_hash = std::env::var("API_HASH").expect("UPLOADING_PATH must be set.");

            tokio::spawn(async move {
                let result = functions::set_tdlib_parameters(
                    false,
                    String::new(),
                    String::new(),
                    String::new(),
                    true,
                    true,
                    true,
                    true,
                    api_id,
                    api_hash.to_string(),
                    "en-US".to_owned(),
                    "Desktop".into(),
                    String::new(),
                    "1.0".to_owned(),
                    true,
                    false,
                    client_id,
                )
                .await;

                if result.is_ok() {
                    println!("TDLib parameters set");
                } else {
                    println!("Error setting TDLib parameters {:?}", result);
                }
            });
        }
        AuthorizationState::WaitPhoneNumber => {
            tokio::spawn(async move {
                println!("Enter phone number: ");
                let mut mobile = "".to_string();
                io::stdin().read_line(&mut mobile).unwrap();
                let result =
                    functions::set_authentication_phone_number(mobile.to_owned(), None, client_id)
                        .await;

                if result.is_ok() {
                    println!("Login Code sent");
                } else {
                    println!("Error setting phone number");
                }
            });
        }
        AuthorizationState::WaitCode(_) => {
            let mut code = "".to_string();
            print!("Enter authentication code: ");
            io::stdin().read_line(&mut code).unwrap();

            tokio::spawn(async move {
                let r = functions::check_authentication_code(code, client_id).await;

                if r.is_ok() {
                    println!("Code OK");
                } else {
                    println!("Error Code");
                }
            });
        }
        AuthorizationState::WaitPassword(_) => {
            let mut password = "".to_string();
            print!("Enter password: ");
            io::stdin().read_line(&mut password).unwrap();

            tokio::spawn(async move {
                let r = functions::check_authentication_password(password, client_id).await;

                if r.is_ok() {
                    println!("Password Ok");
                } else {
                    println!("Error Password");
                }
            });
        }
        AuthorizationState::Ready => {
            spawn_watcher(client_id);
        }
        update => {
            println!("Authorized state: {:?}", update);
        }
    }
}

async fn send_file(chat_id: i64, path: String, client_id: i32) {
    println!("Sending File {:#?} to: {:}", path, chat_id);

    let document = tdlib::types::InputMessageDocument {
        document: tdlib::enums::InputFile::Local(tdlib::types::InputFileLocal { path }),
        thumbnail: None,
        disable_content_type_detection: false,
        caption: None,
    };

    let _ = functions::send_message(
        chat_id,
        0,
        0,
        None,
        tdlib::enums::InputMessageContent::InputMessageDocument(document),
        client_id,
    )
    .await;
}

fn is_uploading(uploading_path: String) -> bool {
    let uploading_count = std::fs::read_dir(uploading_path).unwrap().count();

    println!("Total files in uploading folder {:}", uploading_count);
    let max_concurrent_uploads = 1;
    return uploading_count >= max_concurrent_uploads;
}

fn spawn_watcher(client_id: i32) {
    tokio::spawn(async move {
        let watch_path = std::env::var("WATCH_PATH").expect("WATCH_PATH must be set.");
        let uploading_path = std::env::var("UPLOADING_PATH").expect("UPLOADING_PATH must be set.");
        let send_to = std::env::var("SEND_TO").expect("SEND_TO must be set.");
        let send_to = send_to.parse::<i64>().unwrap();

        loop {
            println!("Checking for new files...");

            // Check if uploading_path contains more than 1 file

            for entry in std::fs::read_dir(watch_path.clone()).unwrap() {
                if is_uploading(uploading_path.clone()) {
                    continue;
                }

                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    println!("{:?} is a dir", path);
                } else {
                    if path.to_str().unwrap().contains(".DS_Store") {
                        continue;
                    }

                    let file_name = path
                        .as_path()
                        .display()
                        .to_string()
                        .split("/")
                        .last()
                        .unwrap()
                        .to_string();

                    let upload_path = format!("{}/{}", uploading_path, file_name);
                    let _ = std::fs::rename(path.to_str().unwrap(), upload_path.clone());

                    send_file(send_to, upload_path, client_id).await;
                }
            }

            sleep(Duration::from_secs(10)).await;
        }
    });
}

fn start_magnet(chat_id: i64, magnet: String, client_id: i32) {
    println!("Starting magnet: {:}", magnet);
    let torrent_api = std::env::var("TORRENT_API").expect("TORRENT_API must be set.");

    tokio::spawn(async move {
        // Send the magnet to an api over post
        let client = reqwest::Client::new();
        let res = client.post(torrent_api).body(magnet).send().await;

        let mut response = "I see a 🧲! Will start downloading now!";
        if res.is_err() {
            response = "Failed to add magnet!";
            println!("Failed to add magnet {:#?}", res);
        }
        let _ = functions::send_message(
            chat_id,
            0,
            0,
            None,
            tdlib::enums::InputMessageContent::InputMessageText(tdlib::types::InputMessageText {
                text: tdlib::types::FormattedText {
                    text: response.to_string(),
                    entities: vec![],
                },
                disable_web_page_preview: true,
                clear_draft: false,
            }),
            client_id,
        )
        .await;
    });
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("The Uploader Started!");

    // Create an instance of the TDLib client
    println!("Creating TDLib client...");
    let client_id = tdlib::create_client();

    tokio::spawn(async move {
        println!("Setting log verbosity level...");
        let _ = functions::set_log_verbosity_level(0, client_id).await;
    });

    loop {
        if let Some((update, client_id)) = tdlib::receive() {
            // println!("Received update: {:?}", update);

            match update {
                Update::AuthorizationState(update) => {
                    handle_login(update.authorization_state, client_id).await;
                }
                Update::NewMessage(data) => match data.message.content {
                    tdlib::enums::MessageContent::MessageText(message_text) => {
                        let message = message_text.text.text;
                        println!("New message {:#?}", message);

                        lazy_static! {
                            static ref RE: Regex =
                                Regex::new(r"magnet:\?xt=urn:[a-z0-9]+:[a-zA-Z0-9&=.%-]{32,}")
                                    .unwrap();
                        }

                        for cap in RE.captures_iter(&message) {
                            println!("Magnet Found\n{}", &cap[0]);
                            start_magnet(data.message.chat_id, cap[0].to_string(), client_id);
                        }
                    }
                    tdlib::enums::MessageContent::MessageDocument(_) => {
                        // println!("New document message {:#?}", message_document);
                    }
                    _ => {}
                },
                Update::MessageSendSucceeded(data) => match data.message.content {
                    tdlib::enums::MessageContent::MessageDocument(message_document) => {
                        let path = message_document.document.document.local.path;
                        if Path::new(&path).exists() {
                            std::fs::remove_file(path).unwrap();
                        }
                    }
                    _ => {}
                },
                Update::File(data) => {
                    let uploaded_size = data.file.remote.uploaded_size;
                    let file_name = data.file.local.path.clone();
                    if data.file.expected_size == uploaded_size {
                        if Path::new(&file_name).exists() {
                            std::fs::remove_file(file_name.clone()).unwrap();
                        }

                        println!("{:} uploaded 100%", file_name.split("/").last().unwrap());
                    } else {
                        let percentage: f64 =
                            uploaded_size as f64 / data.file.expected_size as f64 * 100.0;

                        println!(
                            "{:} uploaded {:.2}%",
                            file_name.split("/").last().unwrap(),
                            percentage
                        );
                    }
                }
                update => {
                    // println!("UNHANDLED Update: {:#?}", update);
                }
            }
        }
    }
}
