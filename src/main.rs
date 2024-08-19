mod table;
mod enum_utils;
mod select;
mod db;
mod query;
mod cli;
use cli::*;
#[tokio::main]
async fn main()
{
    let _ =Cli::new().await.run().await;
}
