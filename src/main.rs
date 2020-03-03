// 予約システムのエンドポイント
const ENDPOINT: &str = "http://example.com/booking.dll";

// 予約システムの開始時刻
const SYSTEM_START_TIME_HOUR: u32 = 8;
const SYSTEM_START_TIME_MIN: u32 = 0;
const SYSTEM_START_TIME_SEC: u32 = 0;

// 予約システムの終了時刻
const SYSTEM_END_TIME_HOUR: u32 = 19;
const SYSTEM_END_TIME_MIN: u32 = 0;
const SYSTEM_END_TIME_SEC: u32 = 0;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 予約を取りたい時間番号を指定
    let target_time: Vec<Vec<u8>> = vec![
        vec![11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
        vec![11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
    ];
    let args: Vec<String> = std::env::args().collect();

    let user = &args[1];
    let password = &args[2];

    loop {
        // 予約を取得する時刻まで待機
        let now = chrono::Local::now();
        if chrono::Local::today()
            .and_time(chrono::NaiveTime::from_hms(
                SYSTEM_START_TIME_HOUR,
                SYSTEM_START_TIME_MIN,
                SYSTEM_START_TIME_SEC,
            ))
            .unwrap()
            .lt(&now)
            && chrono::Local::today()
                .and_time(chrono::NaiveTime::from_hms(
                    SYSTEM_END_TIME_HOUR,
                    SYSTEM_END_TIME_MIN,
                    SYSTEM_END_TIME_SEC,
                ))
                .unwrap()
                .gt(&now)
        {
            // ログインする
            let mut cookie: String = signin(user, password).await;
            let mut signin_count: u32 = 1;
            while cookie == "" {
                print_now_time();
                println!("{}回目のログインに失敗しました。", signin_count);
                cookie = signin(user, password).await;
                signin_count += 1;
            }
            print_now_time();
            println!("ログインに成功しました。");
            let cookie_str = cookie.as_str();

            // 有効な日があるページを取得する
            let available_date_res: String = get_available_date(cookie_str, user).await;

            // 正規表現で有効な日の値を取り出す
            let date_val_re = regex::Regex::new(r"(-N1,-N\d{8})").unwrap();
            let mut date_val: Vec<String> = Vec::new();
            for caps in date_val_re.captures_iter(available_date_res.as_str()) {
                date_val.push(String::from(&caps[0][4..]));
            }

            // 正規表現で有効な日の文字列を取り出す
            let data_string_re = regex::Regex::new(r"(\d+月\d+日（.）)").unwrap();
            let mut date_string: Vec<String> = Vec::new();
            for caps in data_string_re.captures_iter(available_date_res.as_str()) {
                date_string.push(String::from(&caps[0]));
            }
            print_now_time();
            println!("有効な日を取得しました。\n");
            // 予約する
            for day in (0..2).rev() {
                for time in &target_time[day] {
                    let time_str = match time {
                        1 => "8:00 ~ 9:50",
                        2 => "9:00 ~ 10:50",
                        3 => "10:00 ~ 11:50",
                        4 => "11:00 ~ 12:50",
                        5 => "12:00 ~ 13:50",
                        6 => "13:00 ~ 14:50",
                        7 => "14:00 ~ 15:50",
                        8 => "15:00 ~ 16:50",
                        9 => "16:00 ~ 17:50",
                        10 => "17:00 ~ 18:50",
                        11 => "18:00 ~ 19:50",
                        _ => "",
                    };
                    // ログインする
                    let mut cookie: String = signin(user, password).await;
                    let mut signin_count: u32 = 1;
                    while cookie == "" {
                        print_now_time();
                        println!("{}回目のログインに失敗しました。", signin_count);
                        cookie = signin(user, password).await;
                        signin_count += 1;
                    }
                    print_now_time();
                    println!("ログインに成功しました。");
                    let cookie_str = cookie.as_str();
                    // 予約する
                    print_now_time();
                    println!("{}{}の技能教習を予約します。", date_string[day], time_str);
                    let res = book(cookie_str, user, date_val[day].as_str(), *time).await;
                    if res.contains("予約を取得しました") {
                        print_now_time();
                        println!("⭕ 予約に成功しました。");
                    } else if res.contains("すでに予約済みです。") {
                        print_now_time();
                        println!("✔️ すでに予約済みです。");
                    } else {
                        print_now_time();
                        println!("❌ 予約に失敗しました。");
                    }
                    println!("");
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(60));
        } else {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

// ログインする
async fn signin(user: &str, password: &str) -> String {
    let req_body = format!(
        "APPNAME=YKOK&PRGNAME=NET_IDCHECK&ARGUMENTS=-,SEINO,PASS,-,AGREE&SEINO={}&PASS={}",
        user, password
    );
    let client = reqwest::Client::new();
    let res = client.post(ENDPOINT).body(req_body).send().await.unwrap();

    match res.headers().get("set-cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(cookie) => String::from(cookie),
            Err(_) => String::new(),
        },
        None => String::new(),
    }
}

// 有効な日を取得する
async fn get_available_date(cookie: &str, user: &str) -> String {
    let req_body = format!(
        "APPNAME=YKOK&PRGNAME=NET_SYORI&ARGUMENTS=-,-N0,-N0000{},-N1",
        user
    );
    let client = reqwest::Client::new();
    let res = client
        .post(ENDPOINT)
        .header("Cookie", cookie)
        .body(req_body)
        .send()
        .await
        .unwrap();
    let res_body = res.text_with_charset("Shift_JIS").await.unwrap();
    String::from(res_body)
}

// 予約する
async fn book(cookie: &str, user: &str, date: &str, time: u8) -> String {
    let req_body = format!(
        "APPNAME=YKOK&PRGNAME=NET_YOYAKU_BAT&ARGUMENTS=-,-N0000{},-N1,{},-N{},SIMEI",
        user, date, time
    );
    let client = reqwest::Client::new();
    let res = client
        .post(ENDPOINT)
        .header("Cookie", cookie)
        .body(req_body)
        .send()
        .await
        .unwrap();
    let res_body = res.text_with_charset("Shift_JIS").await.unwrap();
    String::from(res_body)
}

// 現在時刻を表示する
fn print_now_time() {
    print!("{}", chrono::Local::now().format("[%H:%M:%S] "));
}
