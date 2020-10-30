// Сделано мною в свободное время для vk.com/real_quotes
// Хочу выразить спасибо Владу, Алине, Наде, Елдосу, MkFair
// Матвей, не скамь бабулек!
// - So how does the story of Wolf Brothers end?

#[macro_use]
extern crate vkapi;

use markov::Chain;
use rand::Rng;
use vkapi::longpoll::EventType;

#[tokio::main]
async fn main() {
    let access_token =
        std::env::var("ACCESS_TOKEN").expect("Вы не указали токен в переменной ACCESS_TOKEN");
    let service_token = std::env::var("SERVICE_ACCESS_TOKEN")
        .expect("Вы не указали сервисный токен в переменной SERVICE_ACCESS_TOKEN");

    let action = std::env::args().nth(1).expect("Вы не указали действие.");
    if action == "train" {
        let mut chain: Chain<String> = Chain::load("trained.chain").unwrap_or(Chain::new());
        let vk_api = vkapi::VK::new("5.103", "ru", service_token);

        let mut last_offset = 0;
        for i in 0..=200 {
            vk_api.request("wall.get", &mut param!{"owner_id" => "-29534144", "offset" => &last_offset.to_string(), "count" => "100"}).await.unwrap()["response"]["items"].members().for_each(|post|{
                let text = post["text"].as_str().unwrap().trim();
                // dbg!(&text);
                chain.feed_str(text);
            });
            println!("Learn {} time, offset is {}", i, last_offset);

            last_offset += 100;
        }

        chain.save("trained.chain").unwrap();
    } else if action == "bot" {
        let vk_api = vkapi::VK::new("5.103", "ru", access_token);
        println!("Reading trained database..");
        let chain: Chain<String> = Chain::load("trained.chain")
            .expect("Для начала натренируйте сеть - .\\wolf_quotes train");
        println!("Successfully read database!");
        let ch = vk_api.start_longpoll(198225294, 25);

        for event in ch {
            match event.0 {
                EventType::NewMessage => {
                    let mut rng = rand::thread_rng();
                    let data = &event.1["message"];
                    println!("JSON: {}", data.pretty(4));
                    let text = data["text"].as_str().unwrap();
                    let peer_id: String = data["peer_id"].as_i32().unwrap().to_string();
                    let random_id: String = rng.gen::<i32>().to_string();
                    dbg!(&text);
                    dbg!(&peer_id);
                    dbg!(&random_id);

                    let mut final_quote = String::new();

                    if text == "/цитата" {
                        final_quote = generate_quote(&chain, None);
                    } else if text.starts_with("/цитата") {
                        let text = text.replace("/цитата ", ""); // получаем текст, который нужно продолжить
                        final_quote = generate_quote(&chain, Some(text));
                    }

                    dbg!(&final_quote);

                    if !final_quote.is_empty(){
                        vk_api
                        .request("messages.send", &mut param!{"random_id" => &random_id, "peer_id" => &peer_id, "message" => &final_quote})
                        .await
                        .unwrap();
                    }
                }
                _ => {} // do nothing. really. nothing.
            }
        }
    } else {
    }
}

pub fn generate_quote(chain: &Chain<String>, start_token: Option<String>) -> String {
     match start_token {
        Some(text) => {
            let mut quote = chain.generate_str_from_token(&text);
            if quote.is_empty() {
                quote = String::from("Я не смог сгенерировать цитату :(");
            }
            println!("Продолжаем текст с {}", text);
            quote
        }
        None => {
            let mut quote = chain.generate_str();
            if quote.is_empty() || quote.contains("http") {
                quote = loop {
                    // циклично генерируем цитату, пока не сгенерируем ее нормально(не пустую)
                    let new_quote = chain.generate_str();
                    if new_quote.is_empty() || quote.contains("http") {
                        continue;
                    } else {
                        break new_quote;
                    }
                };
            }
            quote
        }
    }
}
