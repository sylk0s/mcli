use clap::{Parser, Subcommand};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

const ADDR: &str = "http://localhost:7955";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Starts a server
    Start {
        /// The name of the server to start
        #[arg(short, long)]
        name: String,

        /// The id of the server to start
        #[arg(short, long)]
        id: String,
    },
    /// Stops a server
    Stop {
        /// The name of the server to stop
        #[arg(short, long)]
        name: String,

        /// The id of the server to stop
        #[arg(short, long)]
        id: String,
    },
    /// Executes a command on a server
    Exec {
        /// The name of the server to execute on
        #[arg(short, long)]
        name: String,

        /// The id of the server to execute on
        #[arg(short, long)]
        id: String,

        /// The command to execute
        #[arg(short, long)]
        cmd: Vec<String>,
    },
    /// Gets output from a server
    Output {
        /// The name of the server to get output from
        #[arg(short, long)]
        name: String,

        /// The id of the server to get output from
        #[arg(short, long)]
        id: String,
    },
    /// Gets status from a server
    Status {
        /// The name of the server to get status from
        #[arg(short, long)]
        name: String,

        /// The id of the server to get status from
        #[arg(short, long)]
        id: String,
    },
    /// Creates a new server
    New {
        /// The name of the server to create
        #[arg(short, long)]
        name: String,

        /// The id of the server to create
        #[arg(short, long)]
        id: String,

        /// The path to the server files
        #[arg(short, long)]
        path: Option<String>,

        /// The port to run the server on
        #[arg(short, long)]
        port: Option<u16>,

        /// The version of the server to run
        #[arg(short, long)]
        version: Option<String>,

        /// The type of server to run
        #[arg(short, long)]
        server_type: Option<String>,
    },
    /// Lists servers
    List {
        /// The name of the server to list
        #[arg(short, long)]
        name: String,
    },
    /// Gets a cleaned output from a server
    CleanOutput {
        /// The name of the server to get output from
        #[arg(short, long)]
        name: String,

        /// The id of the server to get output from
        #[arg(short, long)]
        id: String,
    },
}

impl Commands {
    pub async fn execute(&self) {
        match self {
            Commands::Start { name, id } => {
                println!("Starting {} {}", name, id);
                println!(
                    "{:?}",
                    reqwest::Client::new()
                        .put(format!("{ADDR}/{}/{}", name, id))
                        .send()
                        .await
                        .unwrap()
                );
            }
            Commands::Stop { name, id } => {
                println!("Stopping {} {}", name, id);
                println!(
                    "{:?}",
                    reqwest::Client::new()
                        .put(format!("{ADDR}/{}/{}", name, id))
                        .send()
                        .await
                        .unwrap()
                );
            }
            Commands::Exec { name, id, cmd } => {
                let body = format!(
                    "{{\"args\":[{}]}}",
                    cmd.iter()
                        .fold(String::new(), |a, b| format!("{a} \"{b}\","))
                        .trim()
                        .trim_end_matches(",")
                );
                println!("exec {body}");
                println!(
                    "{:?}",
                    reqwest::Client::new()
                        .post(format!("{ADDR}/{}/{}", name, id))
                        .body(body)
                        .send()
                        .await
                );
            }
            Commands::Output { name, id } => {
                let mut stream = reqwest::get(format!("{ADDR}/{}/{}", name, id))
                    .await
                    .unwrap()
                    .bytes_stream();

                while let Some(msg) = stream.next().await {
                    println!("{:?}", msg);
                }
            }
            Commands::Status { name, id } => {
                let res = reqwest::get(format!("{ADDR}/{}/{}", name, id))
                    .await
                    .unwrap();
                if res.status().is_success() {
                    let res1 = res.text().await.unwrap();
                    let thing: StatusResponse = serde_json::from_str(&res1).unwrap();

                    // Format status into a reply
                    let s = if let Some(players) = thing.sample.as_ref() {
                        players
                            .iter()
                            .fold(String::from(":\n"), |a, b| format!("{a}\t{}\n", b.name))
                    } else {
                        String::from("\n")
                    };
                    let out = format!(
                        "{} [{}] [{}/{}]{}",
                        id, thing.version, thing.online_players, thing.max_players, s
                    );
                    println!("{}", out);
                } else {
                    println!("{:?}", res.text().await.unwrap());
                }

                //println!("Body: {:?}", thing);
            }
            Commands::New {
                name,
                id,
                path,
                port,
                version,
                server_type,
            } => {
                let new = New {
                    name: name.clone(),
                    id: id.clone(),
                    path: path.clone(),
                    port: *port,
                    version: version.clone(),
                    server_type: server_type.clone(),
                };
                println!("Sending...");
                println!(
                    "{:?}",
                    reqwest::Client::new()
                        .post(format!("{ADDR}/{}", name))
                        .body(serde_json::to_string(&new).unwrap())
                        .send()
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap()
                );
            }
            Commands::List { name } => {
                let res = reqwest::get(format!("{ADDR}/{}", name)).await.unwrap();
                let res_str: ListResponse =
                    serde_json::from_str(&res.text().await.unwrap()).unwrap();
                let output_str = res_str
                    .servers
                    .iter()
                    .fold(String::from("Servers:\n"), |a, b| format!("{a}{b}\n"));

                println!("{}", output_str);
            }
            Commands::CleanOutput { name, id } => {
                let mut stream = reqwest::get(format!("{ADDR}/{}/{}", name, id))
                    .await
                    .unwrap()
                    .bytes_stream();

                while let Some(msg) = stream.next().await {
                    println!("{}", std::str::from_utf8(&msg.unwrap()).unwrap());
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct StatusResponse {
    version: String,
    max_players: u16,
    online_players: u16,
    sample: Option<Vec<Player>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
    id: String,
}

#[derive(Deserialize)]
struct ListResponse {
    servers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct New {
    name: String,
    id: String,
    path: Option<String>,
    port: Option<u16>,
    version: Option<String>,
    server_type: Option<String>,
}