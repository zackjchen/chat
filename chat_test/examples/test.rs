#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:6688/api/signin")
        .header("Content-Type", "application/json")
        .body(r#"{"email": "zack.j.chen@hkjc.org.hk","password": "Jiajia520"}"#)
        .send()
        .await?;
    println!("resp: {:?}", resp);
    assert_eq!(resp.status(), 200);
    println!("resp: {:?}", resp.text().await?);
    Ok(())
}
