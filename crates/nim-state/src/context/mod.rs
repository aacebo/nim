mod object;
mod person;
mod place;
mod time;

pub use object::*;
pub use person::*;
pub use place::*;
pub use time::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ContextId {
    Object(ObjectId),
    Person(PersonId),
    Place(PlaceId),
    Time(TimeId),
}

impl From<ObjectId> for ContextId {
    fn from(value: ObjectId) -> Self {
        Self::Object(value)
    }
}

impl From<PersonId> for ContextId {
    fn from(value: PersonId) -> Self {
        Self::Person(value)
    }
}

impl From<PlaceId> for ContextId {
    fn from(value: PlaceId) -> Self {
        Self::Place(value)
    }
}

impl From<TimeId> for ContextId {
    fn from(value: TimeId) -> Self {
        Self::Time(value)
    }
}

impl std::fmt::Display for ContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(v) => write!(f, "{v}"),
            Self::Person(v) => write!(f, "{v}"),
            Self::Place(v) => write!(f, "{v}"),
            Self::Time(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Context {
    Object(Object),
    Person(Person),
    Place(Place),
    Time(Time),
}

impl Context {
    pub fn id(self) -> ContextId {
        match self {
            Self::Object(v) => v.id.into(),
            Self::Person(v) => v.id.into(),
            Self::Place(v) => v.id.into(),
            Self::Time(v) => v.id.into(),
        }
    }
}

impl From<Object> for Context {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

impl From<Person> for Context {
    fn from(value: Person) -> Self {
        Self::Person(value)
    }
}

impl From<Place> for Context {
    fn from(value: Place) -> Self {
        Self::Place(value)
    }
}

impl From<Time> for Context {
    fn from(value: Time) -> Self {
        Self::Time(value)
    }
}
