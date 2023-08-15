use serde::Serialize;
use std::env;

const BASE_URL: &'static str =
    "https://www.salford.gov.uk/bins-and-recycling/bin-collection-days/your-bin-collections/?UPRN=";

#[derive(Serialize)]
struct Output {
    date: chrono::NaiveDate,
    black: bool,
    blue: bool,
    brown: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let uprn = match env::var("UPRN") {
        Ok(uprn) => uprn,
        Err(_err) => {
            eprintln!("Error: No UPRN environment variable set");
            std::process::exit(2);
        }
    };

    let webhook_url = match env::var("WEBHOOK_URL") {
        Ok(webhook_url) => webhook_url,
        Err(_err) => {
            eprintln!("Error: No WEBHOOK_URL environment variable set");
            std::process::exit(2);
        }
    };

    let operator_email = match env::var("OPERATOR_EMAIL") {
        Ok(operator_email) => operator_email,
        Err(_err) => {
            eprintln!("Error: Not OPERATOR_EMAIL environment variable set");
            std::process::exit(2);
        }
    };

    let mut url = String::with_capacity(BASE_URL.len() + uprn.len());
    url.insert_str(0, BASE_URL);
    url.insert_str(BASE_URL.len(), &uprn);

    let client = match reqwest::Client::builder()
        .user_agent(format!("binday-bot/{} (reqwest/0.11.10; +{}", env!("CARGO_PKG_VERSION"), operator_email))
        .gzip(true)
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Error: Unable to build HTTP client {:?}", err);
            std::process::exit(1);
        }
    };

    let html = match fetch_page(&client, url).await {
        Ok(html) => html,
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    };
    let (what, when) = match extract_waste(&html) {
        Ok((what, when)) => (what, when),
        Err(_) => {
            std::process::exit(1);
        }
    };
    let bins = extract_bins(&what);
    let collection_date = match extract_date(&when) {
        Ok(collection_date) => collection_date,
        Err(err) => {
            eprintln!("Error parsing date: {:?}", err);
            std::process::exit(1);
        }
    };
    let output = Output {
        black: bins.black,
        blue: bins.blue,
        brown: bins.brown,
        date: collection_date,
    };

    match post_to_webhook(&client, &webhook_url, &output).await {
        Ok(_) => {
            println!(
                "{}",
                serde_json::to_string(&output).expect("Output should serialise")
            );
        }
        Err(err) => {
            eprintln!("Error POSTing data to webhook: {:?}", err);
            std::process::exit(1);
        }
    }
}

async fn fetch_page(
    client: &reqwest::Client,
    url: String,
) -> Result<String, impl std::error::Error> {
    client
        .get(url)
        .header(
            "accept",
            reqwest::header::HeaderValue::from_static("text/html"),
        )
        .send()
        .await?
        .text()
        .await
}

fn extract_waste(html: &str) -> Result<(String, String), ()> {
    use scraper::{selector::Selector, Html};
    let document = Html::parse_document(html);
    let waste = document
        .select(&Selector::parse(".waste").map_err(|_| ())?)
        .next()
        .ok_or(())?;
    let next_waste_collection = waste
        .select(&Selector::parse(".wastecollection").map_err(|_| ())?)
        .next()
        .ok_or(())?
        .text()
        .collect::<String>();
    let next_waste_date = waste
        .select(&Selector::parse(".wastedate").map_err(|_| ())?)
        .next()
        .ok_or(())?
        .text()
        .collect::<String>();
    Ok((next_waste_collection, next_waste_date))
}

struct Bins {
    black: bool,
    blue: bool,
    brown: bool,
}

fn extract_bins(input: &str) -> Bins {
    Bins {
        black: input.contains("black bin"),
        blue: input.contains("blue bin"),
        brown: input.contains("brown bin"),
    }
}

fn extract_date(input: &str) -> Result<chrono::naive::NaiveDate, impl std::error::Error> {
    chrono::naive::NaiveDate::parse_from_str(input, "%A %d %B %Y")
}

async fn post_to_webhook(
    client: &reqwest::Client,
    url: &str,
    output: &Output,
) -> Result<(), impl std::error::Error> {
    client
        .post(url)
        .header(
            "content-type",
            reqwest::header::HeaderValue::from_static("application/json; charset=utf-8"),
        )
        .json(output)
        .send()
        .await
        .map(|_| ())
}
