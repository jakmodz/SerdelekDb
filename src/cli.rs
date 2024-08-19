use crate::db::Db;
use std::io;
use std::string::ToString;

pub struct Cli
{
    database: Db
}
const DATABASE_PATH: &str = "database.msgpack";
impl Cli
{
    pub async fn new()->Self
    {
        Self
        {
            database: Db::open_database(DATABASE_PATH).await.expect("couldn't load database")
        }
    }
    pub async fn run(&mut self)
    {
        loop
        {
            eprint!("your query: ");
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            let buffer = buffer.trim();
            if  buffer == "close"
            {
                match self.database.close_database(&DATABASE_PATH).await
                {
                    Ok(())=>
                        {
                            println!("The database was saved to file at path {DATABASE_PATH}");
                            return;
                        }
                    Err(error)=>
                        {
                            println!("{error}")
                        }
                }
            }
            match self.database.execute_query(buffer.to_string()).await
            {
                Ok(result)=>println!("{}",result),
                Err(error)=>println!("{}",error)
            }
        }
    }
}