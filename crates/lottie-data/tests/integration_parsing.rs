use std::fs::File;
use std::io::BufReader;
use lottie_data::model::LottieJson;

#[test]
fn test_parse_heart_eyes() {
    let file = File::open("tests/heart_eyes.json").expect("Failed to open heart_eyes.json");
    let reader = BufReader::new(file);
    let res: Result<LottieJson, _> = serde_json::from_reader(reader);
    match res {
        Ok(_) => println!("Successfully parsed heart_eyes.json"),
        Err(e) => panic!("Failed to parse heart_eyes.json: {}", e),
    }
}

#[test]
fn test_parse_mobilo_a() {
    let file = File::open("tests/mobilo_a.json").expect("Failed to open mobilo_a.json");
    let reader = BufReader::new(file);
    let res: Result<LottieJson, _> = serde_json::from_reader(reader);
    match res {
        Ok(_) => println!("Successfully parsed mobilo_a.json"),
        Err(e) => panic!("Failed to parse mobilo_a.json: {}", e),
    }
}
