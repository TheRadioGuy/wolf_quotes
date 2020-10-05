#[macro_use]
extern crate vkapi;

use rand::Rng;
use vkapi::longpoll::EventType;
use markov::Chain;

#[tokio::main]
async fn main() {
    let access_token = std::env::var("ACCESS_TOKEN").expect("Вы не указали токен в переменной ACCESS_TOKEN");
    let service_token = std::env::var("SERVICE_ACCESS_TOKEN").expect("Вы не указали сервисный токен в переменной SERVICE_ACCESS_TOKEN");

    let action = std::env::args().nth(1).expect("Вы не указали действие.");
    if action == "train" {
        let mut chain: Chain<String> = Chain::load("trained.chain").unwrap_or(Chain::new());
        let vk_api = vkapi::VK::new(
            "5.103",
            "ru",
            service_token
        );

        let mut last_offset = 0;
        for i in 0..=500 {
            vk_api.request("wall.get", &mut param!{"owner_id" => "-51687183", "offset" => &last_offset.to_string(), "count" => "100"}).await.unwrap()["response"]["items"].members().for_each(|post|{
                let text = post["text"].as_str().unwrap().trim();
                // dbg!(&text);
                chain.feed_str(text);
            });
            println!("Learn {} time, offset is {}", i, last_offset);

            last_offset += 100;
        }

        chain.save("trained.chain").unwrap();


    } else if action == "bot" {
        let vk_api = vkapi::VK::new(
            "5.103",
            "ru",
            access_token
        );
        let chain: Chain<String> = Chain::load("trained.chain").expect("Для начала натренируйте сеть - .\\wolf_quotes train");
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
                    if text == "/цитата" { 
                        let mut quote = chain.generate_str();
                        if quote.is_empty(){
                            quote = loop { // циклично генерируем цитату, пока не сгенерируем ее нормально(не пустую)
                                let new_quote = chain.generate_str();
                                if new_quote.is_empty(){
                                    continue;
                                } else {
                                    break new_quote;
                                }
                            };
                        }

                        quote = quote.replace("добиться", "долбиться").replace("добил", "долбил"); // Вобще можно убрать, но я ору с этого

                        vk_api
                            .request("messages.send", &mut param!{"random_id" => &random_id, "peer_id" => &peer_id, "message" => &quote})
                            .await
                            .unwrap();
                    }
                }
                _ => {}
            }
        }
    } else {

    }
    
}
