use crate::UnifiedEPFLPerson;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct EPFLAccreds {
    name: String,
    phoneList: Vec<String>,
    officeList: Vec<String>,
    path: String,
    acronym: String,
    order: u32,
    position: String,
    rank: u32,
    code: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EPFLPerson {
    sciper: String,
    profile: String,
    firstname: String,
    name: String,
    email: String,
    rank: u32,
    accreds: Vec<EPFLAccreds>,
}

#[derive(Debug)]
pub enum DataError {
    ParseError(&'static str, std::num::ParseIntError),
    MissingRequired(&'static str),
}

fn string_to_u32(string: String, field: &'static str) -> Result<u32, DataError> {
    string
        .parse::<u32>()
        .map_err(move |e| DataError::ParseError(field, e))
}

impl TryInto<UnifiedEPFLPerson> for EPFLPerson {
    type Error = DataError;

    fn try_into(self: Self) -> Result<UnifiedEPFLPerson, Self::Error> {
        let (section, semester) = self
            .accreds
            .iter()
            .find(|accred| accred.path.starts_with("EPFL/ETU/"))
            .and_then(|accred| accred.path.rsplit('/').next())
            .map(|s| {
                let mut parts = s.split('-');
                (
                    parts.next().map(str::to_owned),
                    parts.next_back().map(str::to_owned),
                )
            })
            .unwrap_or((None, None));

        Ok(UnifiedEPFLPerson {
            sciper: string_to_u32(self.sciper, "sciper")?,

            first_name: self.firstname.clone(),
            last_name: self.name.clone(),
            email: self.email.clone(),

            section: section,
            semester: semester.clone(),
            person_type: if !semester.is_none() {
                Some("Etudiant".to_string())
            } else {
                None
            },
        })
    }
}
