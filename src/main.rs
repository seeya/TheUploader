use clap::{arg, Parser};
use dotenv::dotenv;
use std::path::Path;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tdlib::enums::{AuthorizationState, Document, FormattedText, Message, MessageContent, Update};
use tdlib::functions::{self, get_top_chats};
use tdlib::types::{InputFileRemote, MessageDocument};
use tokio::sync::mpsc::{self, Receiver, Sender};

async fn send_file(
    chat_id: i64,
    path: String,
    caption: Option<String>,
    client_id: i32,
) -> Result<tdlib::enums::Message, tdlib::types::Error> {
    println!("Sending File {:#?} to: {:}", path, chat_id);

    let mut formatted_caption: Option<tdlib::types::FormattedText> = None;

    if caption.is_some() {
        formatted_caption = Some(tdlib::types::FormattedText {
            text: caption.unwrap(),
            entities: vec![],
        });
    }

    let document = tdlib::types::InputMessageDocument {
        document: tdlib::enums::InputFile::Local(tdlib::types::InputFileLocal { path }),
        thumbnail: None,
        disable_content_type_detection: false,
        caption: formatted_caption,
    };

    functions::send_message(
        chat_id,
        0,
        0,
        None,
        tdlib::enums::InputMessageContent::InputMessageDocument(document),
        client_id,
    )
    .await
}

fn download_youtube_video(chat_id: i64, link: String, client_id: i32) {
    let video_name = format!("{}.mp4", "test");
    let video_path = format!("./{}", video_name);

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-f")
        .arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/mp4");
    cmd.arg("-o").arg(&video_path);
    cmd.arg(&link);

    let output = cmd.output().expect("Failed to execute command");
    if !output.status.success() {
        println!("Failed to download video: {:?}", output);
        return;
    }

    // tokio::spawn(async move {
    //     send_file(chat_id, video_path, client_id).await;
    // });
}

fn ask_user(string: &str) -> String {
    println!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn handle_update(update: Update, auth_tx: &Sender<AuthorizationState>) {
    match update {
        Update::AuthorizationState(update) => {
            auth_tx.send(update.authorization_state).await.unwrap();
        }
        Update::File(data) => {
            if data.file.remote.id != "" {
                let r = auth_tx.send(AuthorizationState::Closed).await;
                if r.is_err() {
                    println!("Error sending authorization state {:#}", r.err().unwrap());
                } else {
                    println!("File sent!")
                }
            }
        }
        _ => (),
    }
}

async fn handle_authorization_state(
    client_id: i32,
    mut auth_rx: Receiver<AuthorizationState>,
    run_flag: Arc<AtomicBool>,
) -> Receiver<AuthorizationState> {
    while let Some(state) = auth_rx.recv().await {
        match state {
            AuthorizationState::WaitTdlibParameters => {
                let api_id = std::env::var("API_ID").expect("WATCH_PATH must be set.");
                let api_id = api_id.parse::<i32>().unwrap();
                let api_hash = std::env::var("API_HASH").expect("UPLOADING_PATH must be set.");

                let response = functions::set_tdlib_parameters(
                    false,
                    "get_me_db".into(),
                    String::new(),
                    String::new(),
                    true,
                    true,
                    true,
                    false,
                    api_id,
                    api_hash,
                    "en".into(),
                    "Desktop".into(),
                    String::new(),
                    env!("CARGO_PKG_VERSION").into(),
                    false,
                    true,
                    client_id,
                )
                .await;

                if let Err(error) = response {
                    println!("{}", error.message);
                }
            }
            AuthorizationState::WaitPhoneNumber => loop {
                let input = ask_user("Enter your phone number (include the country calling code):");
                let response =
                    functions::set_authentication_phone_number(input, None, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            },
            AuthorizationState::WaitCode(_) => loop {
                let input = ask_user("Enter the verification code:");
                let response = functions::check_authentication_code(input, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            },
            AuthorizationState::Ready => {
                println!("READY!");
                break;
            }
            AuthorizationState::Closed => {
                // Set the flag to false to stop receiving updates from the
                // spawned task
                run_flag.store(false, Ordering::Release);
                break;
            }
            AuthorizationState::WaitPassword(_) => loop {
                let input = ask_user("Enter your 2FA Password:");
                let response = functions::check_authentication_password(input, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            },
            state => {
                println!("Unexpected authorization state: {:?}", state);
            }
        }
    }

    auth_rx
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    hash: String,
    #[arg(short, long)]
    path: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("The Uploader Started!");

    let args = Args::parse();
    let client_id = tdlib::create_client();

    let (auth_tx, auth_rx) = mpsc::channel(5);
    // let (action_tx, mut action_rx) = mpsc::channel(5);

    let run_flag = Arc::new(AtomicBool::new(true));
    let run_flag_clone = run_flag.clone();

    let handle = tokio::spawn(async move {
        while run_flag_clone.load(Ordering::Acquire) {
            if let Some((update, _client_id)) = tdlib::receive() {
                handle_update(update, &auth_tx).await;
            }
        }
    });

    functions::set_log_verbosity_level(0, client_id)
        .await
        .unwrap();

    // tokio::spawn(async move {
    //     println!("Waiting for file to be sent!");
    //     while let Some(state) = action_rx.recv().await {
    //         println!("Received file sent notification! {:}", state);
    //         functions::close(client_id).await.unwrap();
    //     }

    //     println!("Ended!!!")
    // });

    let auth_rx = handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    let r = functions::get_chats(None, 100, client_id).await;

    if r.is_err() {
        println!("Error: {:?}", r);
    } else {
        let send_to = std::env::var("SEND_TO").expect("SEND_TO must be set.");
        let send_to = send_to.parse::<i64>().unwrap();
        let file = send_file(send_to, args.path, Some(args.hash), client_id).await;

        if file.is_err() {
            println!("Failed to send file {:#?}", file.err())
        } else {
            println!("File is uploading");
        }
    }

    println!("ASdfasf");

    handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;
    handle.await.unwrap();
}
