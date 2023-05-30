use reqwest;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::VecDeque;
use std::{thread, time};
use tokio;

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5)) // Set maximum redirects to 5
        .build()
        .unwrap();

    let mut url_queue: VecDeque<String> = VecDeque::new();

    let start = "https://dmoz-odp.org/";
    url_queue.push_back(start.to_string());

    while url_queue.len() > 0 {
        let url = url_queue.pop_front().unwrap();
        println!("Parsing {}", url);
        let response = reqwest::get(url).await.unwrap();

        if response.status().is_success() {
            let body = response.text().await.unwrap();
            //println!("{}", body);
            let document = Html::parse_document(&body);
            let selector = Selector::parse("a").unwrap();

            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join("");
                //println!("Container Text: {}", text);
                if let Some(url) = element.value().attr("href") {
                    if url.contains("http") {
                        println!("URL: {}", url);
                        url_queue.push_back(url.to_string());
                    }
                }
            }
        } else {
            println!("BAD");
        }
        thread::sleep(time::Duration::from_millis(1000));
    }
}
