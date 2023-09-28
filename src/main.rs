use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect(); 

    if args.len() < 2 {
        println!("No command selected");
    } else {
        if let Some(cmd) = mcli::match_command(args) {
            cmd.unwrap().execute().await;
        } else {
            println!("Command not found");
        }
    }
}