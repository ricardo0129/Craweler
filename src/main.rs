use reqwest;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::{thread, time};
use tokio;

pub fn base_url(url: &String) -> String {
    if let Some(i) = url.find(".com") {
        if let Some(sub) = url.get(0..(i + 4)) {
            return sub.to_string();
        } else {
            return url.to_string();
        }
    } else {
        return url.to_string();
    }
}

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5)) // Set maximum redirects to 5
        .build()
        .unwrap();

    let mut url_queue: VecDeque<String> = VecDeque::new();
    let mut visited_url: HashMap<String, i32> = HashMap::new();

    let visited: i32 = 0;
    let start = "https://dmoz-odp.org/";
    const WAIT: u64 = 50;

    let res = client.get(start).send().await.unwrap();
    println!("{}", res.text().await.unwrap());
    url_queue.push_back(start.to_string());

    while url_queue.len() > 0 && visited < 10 {
        let url = url_queue.pop_front().unwrap();
        println!("Parsing {}", url);
        let response = client.get(&url).send().await;

        match response {
            Ok(res) => {
                let body = res.text().await.unwrap();
                let document = Html::parse_document(&body);
                let selector = Selector::parse("a").unwrap();
                let div_selector = Selector::parse("div").unwrap();

                for element in document.select(&div_selector) {
                    let text: Vec<&str> = element
                        .text()
                        .collect::<Vec<_>>()
                        .iter()
                        .map(|s| s.trim())
                        .filter(|&s| !s.is_empty())
                        .collect();
                    if text.len() > 0 {
                        println!("Text: {} URL: {}", text[0], &url);
                    }
                }
                for element in document.select(&selector) {
                    if let Some(url) = element.value().attr("href") {
                        let base: String = base_url(&url.to_string());
                        if url.contains("http") {
                            if !visited_url.contains_key(&base) {
                                visited_url.insert(base.clone(), 1);
                            } else {
                                let val = visited_url.get(&base).unwrap();
                                visited_url.insert(base.clone(), val + 1);
                            }
                            if *visited_url.get(&base).unwrap() <= 5 {
                                url_queue.push_back(url.to_string());
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error occurred during the request: {}", err);
                continue;
            }
        }
        thread::sleep(time::Duration::from_millis(WAIT));
    }
}
