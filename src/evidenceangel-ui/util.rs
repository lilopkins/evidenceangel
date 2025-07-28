use std::path::PathBuf;

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
#[boxed_type(name = "BoxedTestCase")]
#[getset(get = "pub")]
pub struct BoxedTestCase {
    evidence_package_path: PathBuf,
    test_case_id: Uuid,
}

impl BoxedTestCase {
    pub fn new(evidence_package_path: PathBuf, test_case_id: Uuid) -> Self {
        Self { evidence_package_path, test_case_id }
    }
}
