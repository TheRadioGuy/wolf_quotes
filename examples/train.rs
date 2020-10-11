use markov::Chain;
use std::collections::HashMap;
use tokio::time::delay_for;
use tokio::time::Duration;

#[tokio::main]
pub async fn main() {
    let mut chain: Chain<String> = Chain::load("trained.chain").unwrap_or(Chain::new());

    let n: usize = std::env::args()
        .nth(1)
        .unwrap_or("100".to_string())
        .parse()
        .unwrap(); // по дефолту учим 100 раз.

    let mut last_quote = String::new();

    for i in 0..=n {
        // delay_for(Duration::from_secs(1)).await;
        let resp =
            reqwest::get("https://api.forismatic.com/api/1.0/?method=getQuote&format=json&lang=ru")
                .await
                .unwrap()
                .json::<HashMap<String, String>>()
                .await
                .unwrap();

        let text = &resp["quoteText"];
        if text != &last_quote {
            chain.feed_str(&text);
            dbg!(text);
            println!("Learn for {} time", i);
            last_quote = text.clone();
        }
    }

    chain.save("trained.chain").unwrap();
}
