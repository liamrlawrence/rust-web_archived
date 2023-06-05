use std::env;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use deadpool_postgres::{Client, Pool};



#[get("/api/ai/bills")]
async fn ai_bills(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let client: Client = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("PoolError: {}", e))
    })?;

    let statement = client
        .prepare("SELECT billed_to::TEXT, model::TEXT, prompt_tokens::INT, completion_tokens::INT, total_cost::FLOAT FROM View_AI_Bills;")
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let rows = client
        .query(&statement, &[])
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    #[derive(Debug, Serialize)]
    struct QueryTable {
        billed_to:          String,
        model:              String,
        prompt_tokens:      i32,
        completion_tokens:  i32,
        total_cost:         f64,
    }

    let bills: Vec<_> = rows
        .iter()
        .map(|row| QueryTable {
            billed_to:          row.get::<_, String>(0),
            model:              row.get::<_, String>(1),
            prompt_tokens:      row.get::<_, i32>(2),
            completion_tokens:  row.get::<_, i32>(3),
            total_cost:         row.get::<_, f64>(4),
        })
    .collect();

    // println!("{:?}", bills);
    Ok(HttpResponse::Ok().json(bills))
}


#[post("/api/ai/gpt4")]
pub async fn openai_chatgpt() -> impl Responder {
    println!("/api/ai/gpt4");

    #[derive(Serialize, Deserialize, Debug)]
    struct OpenaiRequestMessage {
        role: String,
        content: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct OpenaiRequest {
        model: String,
        messages: Vec<OpenaiRequestMessage>,
        temperature: f32,
    }

    let request = OpenaiRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            OpenaiRequestMessage {
                role: "user".to_string(),
                content: "Say hi".to_string(),
            }
        ],
        temperature: 0.7,
    };

    #[derive(Serialize, Deserialize, Debug)]
    struct OpenaiResponseChoiceMessage {
        content: String,
        role: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct OpenaiResponseChoice {
        finish_reason: String,
        index: i32,
        message: OpenaiResponseChoiceMessage,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct OpenaiResponseUsage {
        completion_tokens: i32,
        prompt_tokens: i32,
        total_tokens: i32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct OpenaiResponse {
        choices: Vec<OpenaiResponseChoice>,
        created: f64,
        id: String,
        model: String,
        object: String,
        usage: OpenaiResponseUsage,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct OutputResponseData{
        message: String,
        usage: OpenaiResponseUsage
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct OutputResponse{
        status: u16,
        message: String,
        data: OutputResponseData,
    }

    let api_key = env::var("OPENAI_API_KEY").unwrap();

    let client = reqwest::Client::new();
    match client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await {
            Ok(res) => {
                let response_body = res.text().await.unwrap();
                let parsed_response: OpenaiResponse =
                    serde_json::from_str(&response_body).expect("Unable to parse the JSON");

                let ai_response = &parsed_response.choices[0].message.content;
                let usage_data = parsed_response.usage;
                
                let output = OutputResponse {
                    status: 200,
                    message: "success".to_string(),
                    data: OutputResponseData {
                        message: ai_response.to_string(),
                        usage: usage_data,
                    }
                };
                HttpResponse::Ok().json(output)
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                HttpResponse::InternalServerError().body(format!("Error: {}", e))
            }
        }
}

