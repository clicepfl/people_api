#[macro_use]
extern crate rocket;
use regex::regex;
use rocket::State;
use rocket::response::status::NotFound;
use rocket::serde::Serialize;
use rocket::serde::json::Json;

use tokio::sync::Mutex;

mod person;
mod search_api;

#[derive(Debug, Serialize)]
struct UnifiedEPFLPerson {
    sciper: u32,

    first_name: String,
    last_name: String,
    email: String,

    section: Option<String>,
    semester: Option<String>,
    person_type: Option<String>,
}

#[get("/person/<data>")]
async fn person_from_many(
    data: &str,
    client: &State<reqwest::Client>,
) -> Result<Json<UnifiedEPFLPerson>, NotFound<String>> {
    let is_nebis = regex!(r"[a-zA-Z][0-9]+").is_match(&data);
    let is_email = regex!(r"[a-zA-Z_-]+\.[a-zA-Z_-]+@epfl\.ch").is_match(&data);
    let is_sciper = regex!(r"[0-9]{6}").is_match(&data);
    if !is_nebis && !is_email && !is_sciper {
        return Err(NotFound("not found".to_string()));
    }
    if (is_email || is_sciper) {
        match client
            .get(format!("https://search-api.epfl.ch/api/ldap?q={data}"))
            .send()
            .await
        {
            Ok(res) => {
                let person: search_api::EPFLPerson = res
                    .json::<Vec<search_api::EPFLPerson>>()
                    .await
                    .map_err(|e| {
                        rocket::error!("Error in parsing from search-api: {:?}", e);
                        NotFound("not found")
                    })
                    .map(|p| p[0].clone())
                    .expect("nahhhh");

                return Ok(Json(person.try_into().expect("aaa")));
            }
            Err(e) => {
                rocket::error!("Error in fetching from search-api: {}", e.to_string());
                return Err(NotFound("not found".to_string()));
            }
        }
    }

    Err(NotFound("not implemented yet".to_string()))
}

#[launch]
async fn rocket() -> _ {
    let client = reqwest::Client::builder().build().unwrap();

    rocket::build()
        .manage(client)
        .mount("/", routes![person_from_many])
}
