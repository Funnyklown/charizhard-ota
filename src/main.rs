use tide::Request;
use tide::prelude::*;



#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/").get(|_| async { Ok("Welcome to Charizhard OTA ! Check /latest to get latest firmware") });
    app.at("/latest").get(latest_firmware);
    app.at("/firmware/").post(new_firmware);
    app.at("/firmware/").get(specific_firmware);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}


async fn latest_firmware(mut req: Request<()>) -> tide::Result{
    
    unimplemented!()
}

async fn specific_firmware(mut req: Request<()>) -> tide::Result{
    unimplemented!()
}

async fn new_firmware(mut req: Request<()>) -> tide::Result{
    unimplemented!()
}