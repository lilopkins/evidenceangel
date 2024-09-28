use evidenceangel::Evidence;
use getset::Getters;
use relm4::gtk::glib;

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
