use async_std::fs;
use async_std::fs::OpenOptions;
use async_std::io;
use async_std::stream::StreamExt;
use regex::Regex;
use tide::log::info;
use tide::prelude::json;
use tide::Request;
use tide::Response;
use tide::StatusCode;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/")
        .get(|_| async { Ok("Welcome to Charizhard OTA ! Check /latest/ to get latest firmware") });
    app.at("/latest/").get(latest_firmware);
    app.at("/firmware/:file").put(new_firmware);
    app.at("/firmware/:file").get(specific_firmware);
    app.at("/firmware/:file").delete(delete_firmware);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn latest_firmware(mut req: Request<()>) -> tide::Result {
    let path = fs::read_dir("./bin/").await;
    let mut firmware_files = Vec::new();

    match path {
        Ok(mut dir) => {
            while let Some(entry) = dir.next().await {
                let entry = entry?;
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                // Validate firmware naming pattern using a regex
                let regex = Regex::new(r"^(?P<name>.+)\.V(?P<major>\d+)\.(?P<minor>\d+)\.bin$")?;
                if let Some(captures) = regex.captures(&file_name_str) {
                    let name = captures.name("name").unwrap().as_str().to_string();
                    let major: u32 = captures.name("major").unwrap().as_str().parse()?;
                    let minor: u32 = captures.name("minor").unwrap().as_str().parse()?;

                    // Push parsed details into a vector
                    firmware_files.push((name, major, minor, file_name_str.to_string()));
                }
            }

            firmware_files.sort_by(|a, b| {
                a.1.cmp(&b.1) // Compare major versions
                    .then_with(|| a.2.cmp(&b.2)) // Then compare minor versions
            });

            if let Some(latest) = firmware_files.last() {
                let mut path: String = "./bin/".to_owned();
                path.push_str(&latest.3);

                let body = tide::Body::from_file(path).await?; // Create a response body from the file
                let mut header: String = "attachment; filename=".to_owned();
                header.push_str(&latest.3);
                return Ok(Response::builder(StatusCode::Ok)
                    .header("Content-Disposition", header)
                    .body(body)
                    .content_type("application/octet-stream") // Set content type for binary download
                    .build());
            }
        }
        Err(_) => {
            return Ok(Response::builder(StatusCode::InternalServerError)
                .body("Failed to read directory".to_string())
                .build());
        }
    };

    Ok(Response::builder(StatusCode::NotFound)
        .body("No firmware found".to_string())
        .build())
}

async fn specific_firmware(mut req: Request<()>) -> tide::Result {
    let file = req.param("file")?;
    let mut path: String = "./bin/".to_owned();
    path.push_str(file);

    if let Ok(body) = tide::Body::from_file(path).await {
        let mut header: String = "attachment; filename=".to_owned();
        header.push_str(file);
        Ok(Response::builder(StatusCode::Ok)
            .header("Content-Disposition", header)
            .body(body)
            .content_type("application/octet-stream") // Set content type for binary download
            .build())
    } else {
        Ok(Response::builder(StatusCode::NotFound)
        .body("Version not found, please use this expression : http://127.0.0.1/firmware/charizhard.Vx.x.bin".to_string())
        .build())
    }
}

async fn new_firmware(mut req: Request<()>) -> tide::Result {
    let path = req.param("file")?;
    let mut fs_path: String = "./bin/".to_owned();
    fs_path.push_str(path);

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&fs_path)
        .await?;

    let bytes_written = io::copy(req, file).await?;

    info!("file written", {
        bytes: bytes_written,
        path: fs_path,
    });

    Ok(json!({ "bytes": bytes_written }).into())
}

async fn delete_firmware(mut req: Request<()>) -> tide::Result {
    let path = req.param("file")?;
    let mut fs_path: String = "./bin/".to_owned();
    fs_path.push_str(path);

    let file = fs::remove_file(fs_path).await;

    match file {
        Ok(_) => return Ok(Response::new(StatusCode::Ok)),
        Err(e) => {
            return Ok(Response::builder(StatusCode::NotFound)
                .body(e.to_string())
                .build());
        }
    }
}
