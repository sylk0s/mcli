use async_trait::async_trait;
use futures::StreamExt;

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
        reqwest::Client::new().put(format!("{ADDR}/{}/{}", self.name, self.id)).send().await.expect("Failed to start the server");
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
        reqwest::Client::new().put(format!("{ADDR}/{}/{}", self.name, self.id)).send().await.expect("Failed to start the server");
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
        reqwest::Client::new().post(format!("{ADDR}/{}/{}", self.name, self.id))
            .body(body)
            .send().await.expect("Failed to send command to the server");
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

        println!("Body: {:?}", res);
    }
}

pub fn match_command(args: Vec<String>) -> Option<Result<Box<dyn Command>, String>> {
    match args[1].as_str() {
        "start" => Some(Start::build_from_args(args)),
        "stop" => Some(Stop::build_from_args(args)),
        "exec" => Some(Exec::build_from_args(args)),
        "output" => Some(Output::build_from_args(args)),
        "status" => Some(Status::build_from_args(args)),
        _ => None,
    }
}
