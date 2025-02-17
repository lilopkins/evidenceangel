use evidenceangel::Evidence;
use getset::Getters;
use relm4::gtk::glib;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Getters, glib::Boxed)]
#[boxed_type(name = "BoxedEvidenceJson")]
#[getset(get = "pub")]
pub struct BoxedEvidenceJson {
    data: Evidence,
}

impl BoxedEvidenceJson {
    pub fn new(data: Evidence) -> Self {
        Self { data }
    }

    pub fn inner(self) -> Evidence {
        self.data
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Getters, glib::Boxed)]
#[boxed_type(name = "BoxedTestCaseById")]
#[getset(get = "pub")]
pub struct BoxedTestCaseById {
    data: Uuid,
}

impl BoxedTestCaseById {
    pub fn new(data: Uuid) -> Self {
        Self { data }
    }

    pub fn inner(self) -> Uuid {
        self.data
    }
}
