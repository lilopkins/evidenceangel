use evidenceangel::TestCasePassStatus;
use gtk::prelude::*;
use relm4::{
    FactorySender,
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
};
use uuid::Uuid;

use crate::{lang, util::BoxedTestCaseById};

pub struct NavFactoryModel {
    selected: bool,
    pub name: String,
    pub status: Option<TestCasePassStatus>,
    pub id: Uuid,
}

#[derive(Clone, Debug)]
pub enum NavFactoryInput {
    ShowAsSelected(bool),
    UpdateTitle(String),
    UpdateStatus(Option<TestCasePassStatus>),
}

#[derive(Debug)]
pub enum NavFactoryOutput {
    NavigateTo(usize, Uuid),
    MoveBefore { case_to_move: Uuid, before: Uuid },
}

pub struct NavFactoryInit {
    pub id: Uuid,
    pub name: String,
    pub status: Option<TestCasePassStatus>,
}

#[relm4::factory(pub)]
impl FactoryComponent for NavFactoryModel {
    type ParentWidget = gtk::Box;
    type Input = NavFactoryInput;
    type Output = NavFactoryOutput;
    type Init = NavFactoryInit;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            gtk::Button {
                add_css_class: "flat",
                set_hexpand: true,
                #[watch]
                set_has_frame: self.selected,

                add_controller = gtk::DragSource {
                    set_actions: gtk::gdk::DragAction::MOVE,

                    connect_prepare[id] => move |_slf, _x, _y| {
                        let dnd_data = BoxedTestCaseById::new(id);
                        tracing::debug!("Drag case started: {dnd_data:?}");
                        Some(gtk::gdk::ContentProvider::for_value(&dnd_data.to_value()))
                    }
                },
                add_controller = gtk::DropTarget {
                    set_actions: gtk::gdk::DragAction::MOVE,
                    set_types: &[BoxedTestCaseById::static_type()],

                    connect_drop[sender, id] => move |_slf, val, _x, _y| {
                        tracing::debug!("Dropped type: {:?}", val.type_());
                        if let Ok(data) = val.get::<BoxedTestCaseById>() {
                            let dropped_case = data.inner();
                            tracing::debug!("Dropped case: {dropped_case:?}");
                            sender.output(NavFactoryOutput::MoveBefore { case_to_move: dropped_case, before: id }).unwrap();
                            return true;
                        }
                        false
                    },
                },

                connect_clicked[sender, index, id] => move |_| {
                    let _ = sender.output(NavFactoryOutput::NavigateTo(index.current_index(), id));
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,

                    gtk::Label {
                        #[watch]
                        set_text: &self.name,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_halign: gtk::Align::Start,
                    },
                    gtk::Label {
                        #[watch]
                        set_text: &match &self.status {
                            None => lang::lookup("test-status-unset-display"),
                            Some(TestCasePassStatus::Pass) => lang::lookup("test-status-pass-display"),
                            Some(TestCasePassStatus::Fail) => lang::lookup("test-status-fail-display"),
                        },
                        set_halign: gtk::Align::Start,
                        add_css_class: "caption",
                        add_css_class: "dimmed",
                    },
                }
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            selected: false,
            name: init.name,
            id: init.id,
            status: init.status,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let id = self.id;
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            NavFactoryInput::ShowAsSelected(sel) => {
                self.selected = sel;
            }
            NavFactoryInput::UpdateTitle(new_title) => {
                self.name = new_title;
            }
            NavFactoryInput::UpdateStatus(new_status) => {
                self.status = new_status;
            }
        }
    }
}
