fn main() -> Result<(), Box<dyn std::error::Error>>{

    let client = redis::Client::open("redis://127.0.0.1/")?;
    let _con = client.get_connection()?;

    /* do something here */

    Ok(())
}
