use ldap3::{Ldap, Scope, SearchEntry, SearchOptions};
use std::collections::HashMap;

use crate::UnifiedEPFLPerson;

fn parse_u32_attr(
    attrs: &HashMap<String, Vec<String>>,
    attr: &'static str,
) -> Result<Option<u32>, DataError> {
    attrs
        .get(attr)
        .and_then(|vals| vals.first())
        .map(|s| s.parse::<u32>())
        .transpose() // making this look relatively clean totally didnt take an embarrassing amount of time nooo
        .map_err(move |e| DataError::ParseError(attr, e))
}
fn parse_u32_mattr(
    attrs: &HashMap<String, Vec<String>>,
    attr: &'static str,
) -> Result<u32, DataError> {
    parse_u32_attr(attrs, attr)
        .transpose()
        .ok_or_else(move || DataError::MissingRequired(attr))?
}

fn parse_string_attr(attrs: &HashMap<String, Vec<String>>, attr: &'static str) -> Option<String> {
    attrs.get(attr).and_then(|v| v.first()).cloned()
}
fn parse_string_mattr(
    attrs: &HashMap<String, Vec<String>>,
    attr: &'static str,
) -> Result<String, DataError> {
    parse_string_attr(attrs, attr).ok_or_else(move || DataError::MissingRequired(attr))
}

#[derive(Debug)]
pub(crate) enum DataError {
    ParseError(&'static str, std::num::ParseIntError),
    MissingRequired(&'static str),
}
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Person {
    uid: Vec<String>,
    cn: Vec<String>,
    sn: Vec<String>,
    object_class: Vec<String>,

    given_name: String,   // givenName
    display_name: String, // displayName
    gecos: String,
    uid_number: u32,        // uidNumber
    gid_number: u32,        // gidNumber
    unique_identifier: u32, // uniqueIdentifier
    mail: String,

    login_shell: String,    // loginShell
    home_directory: String, // homeDirectory

    l: String,
    employee_type: String,
    description: Vec<String>,      // description and description;lang-en
    user_class: String,            // userClass
    epfl_accred_order: u32,        // EPFLAccredOrder
    organizational_status: String, // organizationalStatus
    ou: Vec<String>,               // ou and ou;lang-en (we only care about the short one)
    member_of: Vec<String>,        // memberOf

    edu_person_affiliation: String,             // eduPersonAffiliation
    edu_person_entitlement: String,             // eduPersonEntitlement
    swiss_edu_person_card_uid: String,          // swissEduPersonCardUID
    swiss_edu_person_matriculation_number: u32, // swissEduPersonMatriculationNumber
    swiss_edu_person_study_branch1: u32,        // swissEduPersonStudyBranch1
    swiss_edu_person_study_branch3: u32,        // swissEduPersonStudyBranch3
    swiss_edu_person_study_level: u32,          // swissEduPersonStudyLevel
    swiss_edu_person_unique_id: String,         // swissEduPersonUniqueID
}

impl TryFrom<SearchEntry> for Person {
    type Error = DataError;

    fn try_from(entry: SearchEntry) -> Result<Person, Self::Error> {
        let a = &entry.attrs;

        Ok(Person {
            uid: a.get("uid").cloned().unwrap_or_default(),
            cn: a.get("cn").cloned().unwrap_or_default(),
            sn: a.get("sn").cloned().unwrap_or_default(),
            object_class: a.get("objectClass").cloned().unwrap_or_default(),

            given_name: parse_string_mattr(a, "givenName")?,
            display_name: parse_string_mattr(a, "displayName")?,
            gecos: parse_string_mattr(a, "gecos")?,
            mail: parse_string_mattr(a, "mail")?,
            login_shell: parse_string_mattr(a, "loginShell")?,
            home_directory: parse_string_mattr(a, "homeDirectory")?,
            l: parse_string_mattr(a, "l")?,
            employee_type: parse_string_mattr(a, "employeeType")?,
            user_class: parse_string_mattr(a, "userClass")?,
            organizational_status: parse_string_mattr(a, "organizationalStatus")?,

            uid_number: parse_u32_mattr(a, "uidNumber")?,
            gid_number: parse_u32_mattr(a, "gidNumber")?,
            unique_identifier: parse_u32_mattr(a, "uniqueIdentifier")?,
            epfl_accred_order: parse_u32_mattr(a, "EPFLAccredOrder")?,
            swiss_edu_person_matriculation_number: parse_u32_mattr(
                a,
                "swissEduPersonMatriculationNumber",
            )?,
            swiss_edu_person_study_branch1: parse_u32_mattr(a, "swissEduPersonStudyBranch1")?,
            swiss_edu_person_study_branch3: parse_u32_mattr(a, "swissEduPersonStudyBranch3")?,
            swiss_edu_person_study_level: parse_u32_mattr(a, "swissEduPersonStudyLevel")?,

            description: a.get("description").cloned().unwrap_or_default(),
            ou: a.get("ou").cloned().unwrap_or_default(),
            member_of: a.get("memberOf").cloned().unwrap_or_default(),

            edu_person_affiliation: parse_string_mattr(a, "eduPersonAffiliation")?,
            edu_person_entitlement: parse_string_mattr(a, "eduPersonEntitlement")?,
            swiss_edu_person_card_uid: parse_string_mattr(a, "swissEduPersonCardUID")?,
            swiss_edu_person_unique_id: parse_string_mattr(a, "swissEduPersonUniqueID")?,
        })
    }
}

impl Into<UnifiedEPFLPerson> for Person {
    fn into(self: Self) -> UnifiedEPFLPerson {
        let mut uid = self.uid.clone();
        uid.sort();

        let mut ou = self.ou.clone();
        ou.sort();
        let short_ou: Vec<&str> = ou
            .first()
            .expect("an ou is necessary.... right?")
            .split("-")
            .collect();
        println!("{:?}", self);

        UnifiedEPFLPerson {
            sciper: self.unique_identifier,

            first_name: self.given_name.clone(),
            last_name: self.sn[0].clone(),
            email: self.mail.clone(),

            section: Some(short_ou[0].to_string()),
            semester: short_ou.last().map_or_else(
                || None,
                |sem| {
                    if *sem == short_ou[0] {
                        None
                    } else {
                        Some((*sem).to_string())
                    }
                },
            ),
            person_type: Some(self.employee_type.clone()),
        }
    }
}

pub(crate) async fn get_person_from_ldap(
    ldap: &mut Ldap,
    matching: &str,
) -> Result<Option<UnifiedEPFLPerson>, Box<dyn std::error::Error>> {
    let opts = SearchOptions::default().sizelimit(1);
    let (rs, _res) = ldap
        .with_search_options(opts)
        .search(
            "o=epfl,c=ch",
            Scope::Subtree,
            &format!("(&(objectClass=person)({}))", matching),
            vec!["*"],
        )
        .await?
        .success()?;
    match rs.first() {
        Some(result) => {
            let person: Person = SearchEntry::construct(result.to_owned())
                .try_into()
                .map_err(|e| {
                    eprintln!("ERROR: {e:?}");
                    panic!();
                })
                .expect("no");
            Ok(Some(person.into()))
        }
        None => Ok(None),
    }
}
