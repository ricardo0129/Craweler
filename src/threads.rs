use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::task;

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
}

#[tokio::main]
async fn main() {
    // Create shared data using Arc and Mutex
    let shared_data = Arc::new(Mutex::new(Crawler::new()));

    // Spawn multiple tasks that access and modify the shared data
    for _ in 0..5 {
        let shared_data_clone = Arc::clone(&shared_data);

        task::spawn(async move {
            // Acquire the lock on the shared data
            let mut data = shared_data_clone.lock().unwrap();

            // Modify the shared data
            data.val += 1;

            // Print the shared data
            println!("Shared data: {}", data.val);
        });
    }

    // Wait for all tasks to complete
    task::yield_now().await;
    let shared_data_clone = Arc::clone(&shared_data);
    let data = shared_data_clone.lock().unwrap();
    println!("Final: {}", data.val);
}
