use reqwest;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tokio::task::spawn_blocking;
use tokio::{spawn, task};

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
struct Crawler {
    visited: HashMap<String, i32>,
    queue: VecDeque<String>,
    val: i32,
}

impl Crawler {
    pub fn new() -> Self {
        return Self {
            visited: HashMap::new(),
            queue: VecDeque::new(),
            val: 0,
        };
    }

    pub fn add_visited(&mut self, url: String) {
        let prev = self.get_visited(&url);
        self.visited.insert(url, prev + 1);
    }

    pub fn get_visited(&self, url: &String) -> i32 {
        let prev: i32;
        if !self.visited.contains_key(url) {
            prev = 0;
        } else {
            prev = *self.visited.get(url).unwrap();
        }
        prev
    }

    pub fn next_url(&mut self) -> String {
        let url = self.queue.pop_front().unwrap();
        url
    }

    pub fn add_url(&mut self, url: String) {
        self.queue.push_back(url);
    }

    pub fn queue_size(&self) -> i32 {
        return self.queue.len() as i32;
    }
}

#[tokio::main]
async fn main() {
    let mut crawler: Crawler = Crawler::new();
    let start = "https://dmoz-odp.org/";
    crawler.add_url(start.to_string());
    let shared_data = Arc::new(Mutex::new(crawler));
    let mut handles = Vec::new();
    let num_threads: i32 = 5;

    for i in 0..num_threads {
        let shared_data_clone = Arc::clone(&shared_data);
        let thread_id: i32 = i;
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5)) // Set maximum redirects to 5
            .build()
            .unwrap();
        println!("starting thread {}", i);
        let handle = spawn(async move {
            loop {
                let mut url: String = "".to_string();
                //println!("{}", thread_id);
                {
                    println!("thread {} is waitinf for lock", thread_id);
                    let mut ds = shared_data_clone.lock().unwrap();
                    println!("thread {} has lock", thread_id);
                    if ds.queue_size() != 0 {
                        url = ds.next_url();
                    }
                    //std::mem::drop(ds);
                }
                println!("thread {} released lock", thread_id);
                if url.is_empty() {
                    println!("thread {} is sleeping", thread_id);
                    thread::sleep(time::Duration::from_millis(2000));
                    continue;
                }
                println!("thread {} is querieng url: {}", thread_id, url);
                let response = client.get(&url).send().await;
                println!("DONE");

                match response {
                    Ok(res) => {
                        let body = res.text().await.unwrap();
                        print!("body {}", body);
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
                        //let mut ds = shared_data_clone.lock().unwrap();
                        for element in document.select(&selector) {
                            if let Some(url) = element.value().attr("href") {
                                let base: String = base_url(&url.to_string());
                                println!("new url {}", url);
                                if url.contains("http") {
                                    let mut ds = shared_data_clone.lock().unwrap();
                                    ds.add_url(url.to_string());
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error occurred during the request: {}", err);
                        continue;
                    }
                }
            }
        });
        handles.push(handle);
    }
    let mut results = Vec::with_capacity(handles.len());

    for handle in handles {
        results.push(handle.await.unwrap());
    }
    /* Wait for all tasks to complete
    let shared_data_clone = Arc::clone(&shared_data);
    let data = shared_data_clone.lock().unwrap();
    println!("Final: {}", data.val);
    */
}
