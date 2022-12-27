use async_trait::async_trait;

const ADDR: &str = "http://localhost:7955/";

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
        reqwest::Client::new().put(format!("{ADDR}{}/{}", self.name, self.id)).send().await.expect("Failed to start the server");
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
        reqwest::Client::new().put(format!("{ADDR}{}/{}", self.name, self.id)).send().await.expect("Failed to start the server");
    }
}

pub fn match_command(args: Vec<String>) -> Option<Result<Box<dyn Command>, String>> {
    match args[1].as_str() {
        "start" => Some(Start::build_from_args(args)),
        "stop" => Some(Stop::build_from_args(args)),
        _ => None,
    }
}
