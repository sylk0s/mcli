use async_trait::async_trait;
use futures::StreamExt;
use serde::{Serialize,Deserialize};

const ADDR: &str = "http://localhost:7955";

#[async_trait]
pub trait Command {
    async fn execute(&self);
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> where Self: Sized;
}
struct Start {
    name: String,
    id: String,
}

#[async_trait]
impl Command for Start {
    // Maybe someday implement some way to determine which arg is missing?
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> {
        if args.len() < 3 {
            return Err::<Box<dyn Command>, String>(String::from("Too few args"));
        }
        Ok(Box::new(Start { name: args[1].clone(), id: args[2].clone() }))
    }

    async fn execute(&self) {
        println!("{:?}", reqwest::Client::new().put(format!("{ADDR}/{}/{}", self.name, self.id)).send().await.unwrap());
    }

}

struct Stop {
    name: String,
    id: String,
}

#[async_trait]
impl Command for Stop {
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> {
        if args.len() < 3 {
            return Err::<Box<dyn Command>, String>(String::from("Too few args"));
        }
        Ok(Box::new(Stop { name: args[1].clone(), id: args[2].clone() }))
    }

    async fn execute(&self) {
        println!("{:?}", reqwest::Client::new().put(format!("{ADDR}/{}/{}", self.name, self.id)).send().await.unwrap());
    }
}

struct Exec {
    name: String,
    id: String,
    cmd: Vec<String>,
}

#[async_trait]
impl Command for Exec {
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> {
        if args.len() < 4 {
            return Err::<Box<dyn Command>, String>(String::from("Too few args"));
        }
        Ok(Box::new(Exec { name: args[1].clone(), id: args[2].clone(), cmd: args[3..].to_vec() }))
    }

    async fn execute(&self) {
        let body = format!("{{\"args\":[{}]}}",self.cmd.iter().fold(String::new(), |a, b| format!("{a} \"{b}\",") ).trim().trim_end_matches(","));
        println!("exec {body}");
        println!("{:?}", reqwest::Client::new().post(format!("{ADDR}/{}/{}", self.name, self.id))
            .body(body)
            .send().await);
    }
}

struct Output {
    name: String,
    id: String,
}

#[async_trait]
impl Command for Output {
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> {
        if args.len() < 3 {
            return Err::<Box<dyn Command>, String>(String::from("Too few args"));
        }
        Ok(Box::new(Output { name: args[1].clone(), id: args[2].clone() }))
    }

    async fn execute(&self) {
        let mut stream = reqwest::get(format!("{ADDR}/{}/{}", self.name, self.id)).await.unwrap().bytes_stream();

        while let Some(msg) = stream.next().await {
            println!("{:?}", msg);
        }
    }
}

struct Status {
    name: String,
    id: String,
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

#[async_trait]
impl Command for Status {
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> {
        if args.len() < 3 {
            return Err::<Box<dyn Command>, String>(String::from("Too few args"));
        }
        Ok(Box::new(Status { name: args[1].clone(), id: args[2].clone() }))
    }

    async fn execute(&self) {
        let res = reqwest::get(format!("{ADDR}/{}/{}", self.name, self.id)).await.unwrap();
        if res.status().is_success() {
            println!("aaa");
            let res1 = res.text().await.unwrap();
            let thing: StatusResponse = serde_json::from_str(&res1).unwrap();

            // Format status into a reply
            if let Some(players) = thing.sample.as_ref() {
                let mut s = String::new();
                s = players.iter().fold(String::from(":\n"), |a, b| format!("{a}\t{}\n", b.name));
                let out = format!("{} [{}] [{}/{}]{}", self.id, thing.version, thing.online_players, thing.max_players, s);
                println!("{}", out);
            }
        } else {
            println!("{:?}", res.text().await.unwrap());
        }

        //println!("Body: {:?}", thing);
    }
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

#[async_trait]
impl Command for New {
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> {
        if args.len() < 3 {
            return Err::<Box<dyn Command>, String>(String::from("Too few args"));
        }

        let port = if let Some(string) = find_flag("p", &args) {
            Some(string.parse::<u16>().unwrap()) 
        } else {
            None
        };

        // REDO this so that we can do flags
        Ok(Box::new(New { 
            name: args[1].clone(),
            id: args[2].clone(),
            path: find_flag("d", &args),
            port,
            version: find_flag("v", &args),
            server_type: find_flag("t", &args),
        }))
    }

    async fn execute(&self) {
        println!("Sending...");
        println!("{:?}", reqwest::Client::new().post(format!("{ADDR}/{}", self.name))
            .body(serde_json::to_string(self).unwrap())
            .send().await.unwrap().text().await.unwrap());
    }
}

fn find_flag(flag: &str, args: &Vec<String>) -> Option<String> {
    if let Some(pos) = args.iter().position(|a| *a == format!("-{flag}")) {
        Some(args[pos+1].clone())
    } else {
        None
    }
}

struct List {
    name: String,
}

#[derive(Deserialize)]
struct ListResponse {
    servers: Vec<String>,
}

#[async_trait]
impl Command for List {
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> {
        if args.len() < 2 {
            return Err::<Box<dyn Command>, String>(String::from("Too few args"));
        }

        Ok(Box::new(List { name: args[1].clone() }))
    }

    async fn execute(&self) {
        let res = reqwest::get(format!("{ADDR}/{}", self.name)).await.unwrap();
        let res_str: ListResponse = serde_json::from_str(&res.text().await.unwrap()).unwrap();
        let output_str = res_str.servers.iter().fold(String::from("Servers:\n"), |a, b| { format!("{a}{b}\n") });
        
        println!("{}", output_str);
    }
}

struct CleanOutput {
    name: String,
    id: String,
}

#[async_trait]
impl Command for CleanOutput {
    fn build_from_args(args: Vec<String>) -> Result<Box<dyn Command>, String> {
        if args.len() < 3 {
            return Err::<Box<dyn Command>, String>(String::from("Too few args"));
        }
        Ok(Box::new(CleanOutput { name: args[1].clone(), id: args[2].clone() }))
    }

    async fn execute(&self) {
        let mut stream = reqwest::get(format!("{ADDR}/{}/{}", self.name, self.id)).await.unwrap().bytes_stream();

        while let Some(msg) = stream.next().await {
            println!("{}", std::str::from_utf8(&msg.unwrap()).unwrap());
        }
    }
}

pub fn match_command(args: Vec<String>) -> Option<Result<Box<dyn Command>, String>> {
    match args[1].as_str() {
        "start" => Some(Start::build_from_args(args)),
        "stop" => Some(Stop::build_from_args(args)),
        "exec" => Some(Exec::build_from_args(args)),
        "fullout" => Some(Output::build_from_args(args)),
        "status" => Some(Status::build_from_args(args)),
        "new" => Some(New::build_from_args(args)),
        "list" => Some(List::build_from_args(args)),
        "out" => Some(CleanOutput::build_from_args(args)),
        _ => None,
    }
}
