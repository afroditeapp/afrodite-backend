use model::ProfileAge;
use simple_backend_model::NonEmptyString;

pub struct ProfileNameAndAge {
    pub name: Option<NonEmptyString>,
    pub age: ProfileAge,
}
