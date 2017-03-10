use super::error::{Error, RepoError, ParameterError};
use std::result;
use chrono::*;
use entities::*;
use super::db::Repo;
use super::validate::Validate;
use uuid::Uuid;

////////////////
// USE CASE: user requests an entry
//
// What should happen:
// * assume the user has already the base ID (e.g. from a research by name
//   or by tag)
// * get the entry base of that ID
// * get the newest entry of that ID -- TODO: this is a DB job
// ==> just return the fitting entry the ID
// * get the list of tags that links to that ID, updated to the newest state
//   (respecting all additions and deletions of tags)
//
// * return the entry and the list of tags

pub fn request_entry<RE : Repo<Entry>, RT : Repo<Tag>, RS : Repo<SentenceTriple>>(re : &RE, rt : &RT, rs : &RS, id : &str) -> Result<Entry> {
    match re.get(id) {
        Ok(e) => {
            let tags = get_tags_for_entry_id(rt, rs, id)?;
            let entry_with_tags = Entry {
                id          :  e.id,
                created     :  e.created,
                version     :  e.version+1,
                title       :  e.title,
                description :  e.description,
                lat         :  e.lat,
                lng         :  e.lng,
                street      :  e.street,
                zip         :  e.zip,
                city        :  e.city,
                country     :  e.country,
                email       :  e.email,
                telephone   :  e.telephone,
                homepage    :  e.homepage,
                categories  :  e.categories,
                tags        :  tags,
                license     :  e.license
            };
            Ok(entry_with_tags)
        },
        Err(e) => Err(super::error::Error::Repo(e))
    } 
}

pub fn get_tags_for_entry_id<RT : Repo<Tag>, RS : Repo<SentenceTriple>>(rt : &RT, rs : &RS, id : &str) -> Result<Vec<String>> {
    // nur die SentenceTriples aus rs auslesen, die auf die id referenzieren
    // und die Tag-IDs extrahieren
    //let mut matching_tag_ids : Vec<String> = vec![];

    Ok(rs.all()?
        .into_iter()
        .filter_map(|t|
            match t {
                SentenceTriple { subject : id, predicate : Predicate::IsTaggedAs, object } => {
                    Some(object)
                },
                _ => None
            }
        )
        .collect())
}

// Now, as you have the tag IDs, you can get the names.
pub fn get_tag_names_from_ids<RT : Repo<Tag>>(rt : RT, id : &str) -> Result<Vec<String>> {
    Ok(rt.all()?
        .into_iter()
        .filter(|t| t.id == id)
        .map(|t| t.name)
        .collect())
}

//
// USE CASE: user requests an entry (head entry, no date restriction)
////////////////

////////////////
// USE CASE: user adds a tag to an entry
//
// What should happen:
// * test whether the entry already is connected to that tag
// * if not, search for that tag
// * if non-existent, generate tag
// ** save tag to repo
// * connect entry and tag
// ** save conection to repo

pub fn add_tag_to_entry<RE : Repo<Entry>, RT : Repo<Tag>, RS : Repo<SentenceTriple>>(re : &RE, rt : &mut RT, rs : &mut RS, tag : &str, entry_id : &str) -> Result<()> {
    if tag == ""
    {
        return Err(Error::Parameter(ParameterError::Tag));
    }

    let tag_id_res = find_or_create_tag_id_by_name(rt, tag);
    let tag_ids_of_entry : Vec<String> = get_tags_for_entry_id(rt, rs, entry_id)?;
    match tag_id_res {
        Ok(tag_id) => {
            match tag_ids_of_entry.iter().find(|id| **id == tag_id) {
                Some(t) => {
                    Ok(()) // we are done
                }
                None => {
                    // add a triple to the sentence repo
                    add_is_tagged_relation(rs, entry_id, &tag_id);
                    Ok(())
                }
                // TODO: Is there an Err case?  When?
            }
        }
        Err(e) => Err(e)
    }
}

pub fn find_or_create_tag_id_by_name<RT : Repo<Tag>>(rt : &mut RT, tag : &str) -> Result<String> {
    match rt.all()?
        .into_iter()
        .find(|t| t.name == tag)
    {
        Some(x) => Ok(x.id),
        None => {
            let tag_id = create_new_tag(rt, NewTag { name : tag.to_string()  })?;
            Ok(tag_id)
        }
    }
}

pub fn add_is_tagged_relation<RS : Repo<SentenceTriple>>(rs : &mut RS, entry_id : &str, tag_id : &str) {
    rs.create(&SentenceTriple {
        subject   : entry_id.to_string(),
        predicate : Predicate::IsTaggedAs,
        object    : tag_id.to_string()
    });
}
// USE CASE: user adds a tag to an entry
////////////////

////////////////
// USE CASE: user researches a tag
//
// What should happen:
// * assume the user only knows the keyword he wants to research
// * find the ID associated with the keyword
// * get a list of all entries that are linked with that tag
// ** i.e. first, get all IDs associated with the tag
// ** then, get the entries associated with the IDs
// ** (future) follow equivalence and sub-class links
//
// * return the newest state of each entry

pub fn search_by_tags<RE : Repo<Entry>, RT : Repo<Tag>, RS : Repo<SentenceTriple>>(re : &RE, rt : &mut RT, rs : &RS, tags : &Vec<String>) -> Result<Vec<Entry>> {
    let tag_ids = get_tag_ids_by_tags(rt, tags)?;
    let ids = get_associated_entry_ids_of_tags(rs, &tag_ids)?;
    let entries = get_entries_by_ids(re, &ids)?;
    Ok(entries)
}

pub fn get_tag_ids_by_tags<RT : Repo<Tag>>(rt : &RT, tag_names : &Vec<String>) -> Result<Vec<String>> {
    Ok(rt.all()?
        .into_iter()
        .filter_map(|tag|
            if tag_names.iter().any(|name| **name == tag.name) {
                Some(tag.id)
            } else {
                None
            }
        )
        .collect())
}

pub fn get_associated_entry_ids_of_tags<RS : Repo<SentenceTriple>>(rs : &RS, tag_ids : &Vec<String>) -> Result<Vec<String>> {
    let mut ids = rs.all()?
        .into_iter()
        .filter(|triple| tag_ids.iter().any(|tag_id| *tag_id == triple.object))
        .filter_map(|triple|
            match triple {
                SentenceTriple { subject, predicate : Predicate::IsTaggedAs, object } => {
                    Some(subject)
                }
                _ => None
            }
        )
        .collect::<Vec<String>>();
    ids.dedup();
    Ok(ids)
}

pub fn get_entries_by_ids<RE : Repo<Entry>>(re : &RE, ids : &Vec<String>) -> Result<Vec<Entry>> {
    Ok(re.all()?
        .into_iter()
        .filter(|entry| ids.iter().any(|id| **id == entry.id))
        .collect())
}

//
// USE CASE: user researches a tag
////////////////



////////////////
// USE CASE: (future) onthological researches
// 
// What should happen:
// * the user researches sub-class / super-class / equivalence / similarity
//   relations to a keyword
// * return all sub-class tags / all super-class tags / all equivalent tags /
//   all direct similarities

////////
// sub-case: sub-classes

//
////////

////////
// sub-case: super-classes

//
////////

////////
// sub-case: equivalences

//
////////

////////
// sub-case: similarities

//
////////

// USE CASE: (future) onthological researches
////////////////


type Result<T> = result::Result<T,Error>;

trait Id {
    fn id(&self) -> &str;
}

impl Id for Entry {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Category {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Tag {
    fn id(&self) -> &str {
        &self.id
    }
}


// TODO: Use database trait to make IDs for sentence triples obsolete
impl Id for SentenceTriple {
    fn id(&self) -> &str { "" }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewEntry {
    title       : String,
    description : String,
    lat         : f64,
    lng         : f64,
    street      : Option<String>,
    zip         : Option<String>,
    city        : Option<String>,
    country     : Option<String>,
    email       : Option<String>,
    telephone   : Option<String>,
    homepage    : Option<String>,
    categories  : Vec<String>,
    tags        : Vec<String>,
    license     : String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTag {
    name : String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateEntry {
    id          : String,
    version     : u64,
    title       : String,
    description : String,
    lat         : f64,
    lng         : f64,
    street      : Option<String>,
    zip         : Option<String>,
    city        : Option<String>,
    country     : Option<String>,
    email       : Option<String>,
    telephone   : Option<String>,
    homepage    : Option<String>,
    categories  : Vec<String>,
    tags        : Vec<String>,
}

pub fn create_new_entry<R: Repo<Entry>>(r: &mut R, e: NewEntry) -> Result<String>
 {
    let e = Entry{
        id          :  Uuid::new_v4().simple().to_string(),
        created     :  UTC::now().timestamp() as u64,
        version     :  0,
        title       :  e.title,
        description :  e.description,
        lat         :  e.lat,
        lng         :  e.lng,
        street      :  e.street,
        zip         :  e.zip,
        city        :  e.city,
        country     :  e.country,
        email       :  e.email,
        telephone   :  e.telephone,
        homepage    :  e.homepage,
        categories  :  e.categories,
        tags        :  e.tags,
        license     :  Some(e.license)
    };
    e.validate()?;
    r.create(&e)?;
    Ok(e.id)
}

pub fn create_new_tag<R: Repo<Tag>>(r: &mut R, t: NewTag) -> Result<String> {
    let t = Tag {
        id          :  Uuid::new_v4().simple().to_string(),
        created     :  UTC::now().timestamp() as u64,
        version     :  0,
        name        :  t.name
    };
    r.create(&t)?;
    Ok(t.id)
}

pub fn update_entry<R: Repo<Entry>>(r: &mut R, e: UpdateEntry) -> Result<()> {
    let old : Entry = r.get(&e.id)?;
    if old.version != e.version {
        return Err(Error::Repo(RepoError::InvalidVersion))
    }
    let e = Entry{
        id          :  e.id,
        created     :  UTC::now().timestamp() as u64,
        version     :  e.version+1,
        title       :  e.title,
        description :  e.description,
        lat         :  e.lat,
        lng         :  e.lng,
        street      :  e.street,
        zip         :  e.zip,
        city        :  e.city,
        country     :  e.country,
        email       :  e.email,
        telephone   :  e.telephone,
        homepage    :  e.homepage,
        categories  :  e.categories,
        tags        :  e.tags,
        license     :  old.license
    };
    r.update(&e)?;
    Ok(())
}

////////////////
// TESTS
#[cfg(test)]
pub mod tests {

    use super::*;

    /////////////////////////
    // Entry Search Tests
    /////////////////////////

    #[ignore]
    #[test]
    // ENVIRONMENT: tags, entries, some links from tags to entries
    // INPUT: empty vec of tags
    // OUTPUT: vec of entities
    // ASSERT: vec of entities is empty
    fn empty_search_by_tags()
    {
       // SETUP
       let mut rt : MockRepo<Tag> = MockRepo { objects : vec![] };
       let mut re : MockRepo<Entry> = MockRepo { objects : vec![] };
       let mut rs : MockRepo<SentenceTriple> = MockRepo { objects : vec![] };

       // RUN
       // CHECK
        unimplemented!();
    }

    #[ignore]
    #[test]
    // ENVIRONMENT:
    // * no tags, no entries, no links
    // * alternate: tags, and entries, but no links
    // * alternate: tags, but no entries and links
    // * alternate: entries, but no tags and links
    // INPUT: a tag vec
    // OUTPUT: vec of entities
    // ASSERT: vec of entities is empty
    fn search_on_empty_db()
    {
    }

    #[ignore]
    #[test]
    // ENVIRONMENT: tags, entries, some links from tags to entries
    // INPUT: vec of one tag (existing)
    // OUTPUT: vec of associated entries
    // ASSERT: only the fitting entries are given back
    fn search_by_one_tag()
    {
        unimplemented!();
    }

    #[ignore]
    #[test]
    // ENVIRONMENT: tags, entries, some links from tags to entries
    // INPUT: vec of one tag (undefined)
    // OUTPUT: vec of associated entries
    // ASSERT: output should be empty
    fn search_by_undefined_tag()
    {
        unimplemented!();
    }

    ////////////////////////////////
    // Tag Addition Tests
    ////////////////////////////////

    #[test]
    fn add_empty_tag_to_entry()
    {
        // SETUP
        let mut rt : MockRepo<Tag> = MockRepo { objects : vec![] };
        let mut re : MockRepo<Entry> = MockRepo { objects : vec![] };
        let mut rs : MockRepo<SentenceTriple> = MockRepo { objects : vec![] };

        let x = NewEntry {
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : "CC0-1.0".into()
        };

        let entry_id = create_new_entry(&mut re, x).unwrap();

        let tag_name  = "";

        // RUN

        let result = add_tag_to_entry(&re, &mut rt, &mut rs, tag_name, &entry_id);

        // CHECK

        assert_eq!(rs.objects.len(), 0);
        assert_eq!(rt.objects.len(), 0);
        assert!(result.is_err());
    }

    #[test]
    fn add_valid_tag_to_entry()
    {
        // SETUP
        let mut rt : MockRepo<Tag> = MockRepo { objects : vec![] };
        let mut re : MockRepo<Entry> = MockRepo { objects : vec![] };
        let mut rs : MockRepo<SentenceTriple> = MockRepo { objects : vec![] };

        // RUN
        // CHECK
    }

    #[test]
    fn add_uppercase_tag_to_entry()
    {
        // SETUP
        let mut rt : MockRepo<Tag> = MockRepo { objects : vec![] };
        let mut re : MockRepo<Entry> = MockRepo { objects : vec![] };
        let mut rs : MockRepo<SentenceTriple> = MockRepo { objects : vec![] };

        // RUN
        // CHECK
    }

    #[test]
    fn add_untrimmed_tag_to_entry()
    {
        // SETUP
        let mut rt : MockRepo<Tag> = MockRepo { objects : vec![] };
        let mut re : MockRepo<Entry> = MockRepo { objects : vec![] };
        let mut rs : MockRepo<SentenceTriple> = MockRepo { objects : vec![] };

        // RUN
        // CHECK
    }

    #[test]
    fn add_tag_with_invalid_characters_to_entry()
    {
    }



    type RepoResult<T> = result::Result<T, RepoError>;

    pub struct MockRepo<T> {
        objects: Vec<T>,
    }

    impl<T> MockRepo<T> {
        pub fn new() -> MockRepo<T> {
            MockRepo {
                objects: vec![]
            }
        }

        pub fn clear(&mut self) {
            self.objects = vec![];
        }
    }

    impl<T:Id + Clone> Repo<T> for MockRepo<T> {

        fn get(&self, id: &str) -> RepoResult<T> {
            match self.objects.iter().find(|x| x.id() == id) {
                Some(x) => Ok(x.clone()),
                None => Err(RepoError::NotFound),
            }
        }

        fn all(&self) -> RepoResult<Vec<T>> {
            Ok(self.objects.clone())
        }

        fn create(&mut self, e: &T) -> RepoResult<()> {
            if self.objects.iter().any(|x| x.id() == e.id()) {
                return Err(RepoError::AlreadyExists)
            } else {
                self.objects.push(e.clone());
            }
            Ok(())
        }

        fn update(&mut self, e: &T) -> RepoResult<()> {
            if let Some(pos) = self.objects.iter().position(|x| x.id() == e.id()) {
                self.objects[pos] = e.clone();
            } else {
                return Err(RepoError::NotFound)
            }
            Ok(())
        }
    }

    #[test]
    fn create_new_valid_entry() {
        let x = NewEntry {
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : "CC0-1.0".into()
        };
        let mut mock_db: MockRepo<Entry> = MockRepo { objects: vec![] };
        let now = UTC::now();
        let id = create_new_entry(&mut mock_db, x).unwrap();
        assert!(Uuid::parse_str(&id).is_ok());
        assert_eq!(mock_db.objects.len(),1);
        let x = &mock_db.objects[0];
        assert_eq!(x.title, "foo");
        assert_eq!(x.description, "bar");
        assert_eq!(x.version, 0);
        assert!(x.created as i64 >= now.timestamp());
        assert!(Uuid::parse_str(&x.id).is_ok());
        assert_eq!(x.id, id);
    }

    #[test]
    fn create_entry_with_invalid_email() {
        let x = NewEntry {
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : Some("fooo-not-ok".into()),
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : "CC0-1.0".into()
        };
        let mut mock_db: MockRepo<Entry> = MockRepo { objects: vec![] };
        assert!(create_new_entry(&mut mock_db, x).is_err());
    }

    #[test]
    fn update_valid_entry(){
        let id = Uuid::new_v4().simple().to_string();
        let old = Entry {
            id          : id.clone(),
            version     : 1,
            created     : 0,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : None
        };
        let new = UpdateEntry {
            id          : id.clone(),
            version     : 1,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : Some("street".into()),
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
        };
        let mut mock_db : MockRepo<Entry> = MockRepo{ objects: vec![old]};
        let now = UTC::now();
        assert!(update_entry(&mut mock_db, new).is_ok());
        assert_eq!(mock_db.objects.len(),1);
        let x = &mock_db.objects[0];
        assert_eq!(x.street, Some("street".into()));
        assert_eq!(x.description, "bar");
        assert_eq!(x.version, 2);
        assert!(x.created as i64 >= now.timestamp());
        assert!(Uuid::parse_str(&x.id).is_ok());
    }

    #[test]
    fn update_entry_with_invalid_version(){
        let id = Uuid::new_v4().simple().to_string();
        let old = Entry {
            id          : id.clone(),
            version     : 3,
            created     : 0,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : None
        };
        let new = UpdateEntry {
            id          : id.clone(),
            version     : 4,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : Some("street".into()),
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
        };
        let mut mock_db : MockRepo<Entry> = MockRepo{ objects: vec![old]};
        let result = update_entry(&mut mock_db, new);
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::Repo(err) => {
                match err {
                    RepoError::InvalidVersion => { },
                    _ => {
                        panic!("invalid error type");
                    }
                }
            },
            _ => {
                panic!("invalid error type");
            }
        }
        assert_eq!(mock_db.objects.len(),1);
    }

    #[test]
    fn update_non_existing_entry(){
        let id = Uuid::new_v4().simple().to_string();
        let new = UpdateEntry {
            id          : id.clone(),
            version     : 4,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : Some("street".into()),
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
        };
        let mut mock_db : MockRepo<Entry> = MockRepo{ objects: vec![]};
        let result = update_entry(&mut mock_db, new);
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::Repo(err) => {
                match err {
                    RepoError::NotFound => { },
                    _ => {
                        panic!("invalid error type");
                    }
                }
            },
            _ => {
                panic!("invalid error type");
            }
        }
        assert_eq!(mock_db.objects.len(),0);
    }
}
