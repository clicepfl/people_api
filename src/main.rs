#[macro_use]
extern crate rocket;
use regex::regex;
use rocket::State;
use rocket::response::status::NotFound;
use rocket::serde::Serialize;
use rocket::serde::json::Json;

use tokio::sync::Mutex;

use ldap3::{Ldap, LdapConnAsync};

mod ldap;
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
    shared_ldap: &State<SharedLdap>,
    client: &State<reqwest::Client>,
) -> Result<Json<UnifiedEPFLPerson>, NotFound<String>> {
    let is_nebis = regex!(r"^[a-zA-Z][0-9]+").is_match(&data);
    let is_email = regex!(r"^[a-zA-Z_-]+\.[a-zA-Z_-]+@epfl\.ch").is_match(&data);
    let is_sciper = regex!(r"^[0-9]{6}").is_match(&data);

    if !is_nebis && !is_email && !is_sciper {
        return Err(NotFound("not found".to_string()));
    }
    if is_email || is_sciper {
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

    // TODO: handle this in the ldap module
    let pattern = if is_nebis {
        format!("swissEduPersonCardUID={}@bibliopass.ch", data)
    } else if is_email {
        format!("mail={}", data)
    } else if is_sciper {
        format!("uniqueIdentifier={}", data)
    } else {
        return Err(NotFound("not found".to_string()));
    };

    let mut ldap = shared_ldap.lock().await;

    let result = ldap::get_person_from_ldap(&mut ldap, &pattern).await;
    match result {
        Ok(Some(person)) => Ok(Json(person)),
        Ok(None) => Err(NotFound("not found".to_string())),
        _ => Err(NotFound("hm".to_string())),
    }
}

type SharedLdap = Mutex<Ldap>;

#[launch]
async fn rocket() -> _ {
    let client = reqwest::Client::builder().build().unwrap();
    let (conn, ldap) = LdapConnAsync::new("ldaps://ldap.epfl.ch").await.unwrap();
    ldap3::drive!(conn);

    rocket::build()
        .manage(client)
        .manage(Mutex::new(ldap))
        .mount("/", routes![person_from_many])
}
