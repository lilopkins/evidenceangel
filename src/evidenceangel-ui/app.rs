use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use adw::prelude::*;
use evidenceangel::{
    exporters::{
        excel::ExcelExporter, html::HtmlExporter, zip_of_files::ZipOfFilesExporter, Exporter,
    },
    Author, Evidence, EvidenceData, EvidenceKind, EvidencePackage, MediaFile,
};
#[allow(unused)]
use gtk::prelude::*;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw,
    factory::FactoryVecDeque,
    gtk::{self, gio::Cancellable},
    prelude::*,
    Component, ComponentParts, ComponentSender,
};
use uuid::Uuid;

use crate::{
    author_factory::{AuthorFactoryModel, AuthorFactoryOutput},
    dialogs::{add_evidence::*, error::*, export::*, new_author::*},
    evidence_factory::{EvidenceFactoryInit, EvidenceFactoryModel, EvidenceFactoryOutput},
    filter, lang, lang_args,
    nav_factory::{NavFactoryInit, NavFactoryInput, NavFactoryModel, NavFactoryOutput},
    util::BoxedEvidenceJson,
};

relm4::new_action_group!(MenuActionGroup, "menu");
relm4::new_stateless_action!(NewAction, MenuActionGroup, "new");
relm4::new_stateless_action!(OpenAction, MenuActionGroup, "open");
relm4::new_stateless_action!(SaveAction, MenuActionGroup, "save");
relm4::new_stateless_action!(CloseAction, MenuActionGroup, "close");
relm4::new_stateless_action!(AboutAction, MenuActionGroup, "about");
relm4::new_stateless_action!(PasteEvidenceAction, MenuActionGroup, "paste-evidence");
relm4::new_stateless_action!(ExportPackageAction, MenuActionGroup, "export-package");
relm4::new_stateless_action!(ExportTestCaseAction, MenuActionGroup, "export-test-case");

pub struct AppModel {
    open_package: Option<Arc<RwLock<EvidencePackage>>>,
    open_path: Option<PathBuf>,
    open_case: OpenCase,
    needs_saving: bool,

    action_save: RelmAction<SaveAction>,
    action_close: RelmAction<CloseAction>,
    action_export_package: RelmAction<ExportPackageAction>,
    action_export_test_case: RelmAction<ExportTestCaseAction>,
    action_paste_evidence: RelmAction<PasteEvidenceAction>,

    latest_new_author_dlg: Option<Controller<NewAuthorDialogModel>>,
    latest_add_evidence_image_dlg: Option<Controller<AddImageEvidenceDialogModel>>,
    latest_add_evidence_file_dlg: Option<Controller<AddFileEvidenceDialogModel>>,
    latest_error_dlg: Option<Controller<ErrorDialogModel>>,
    latest_export_dlg: Option<Controller<ExportDialogModel>>,
    latest_delete_toasts: Vec<adw::Toast>,

    test_case_nav_factory: FactoryVecDeque<NavFactoryModel>,
    authors_factory: FactoryVecDeque<AuthorFactoryModel>,
    test_evidence_factory: FactoryVecDeque<EvidenceFactoryModel>,
}

impl AppModel {
    fn open_new(&mut self) -> evidenceangel::Result<()> {
        let title = lang::lookup("default-title");
        let authors = vec![Author::new(lang::lookup("default-author"))];
        let pkg = EvidencePackage::new(
            self.open_path
                .as_ref()
                .expect("Path must be set before calling open_new")
                .clone(),
            title,
            authors,
        )?;
        tracing::debug!("Package opened: {pkg:?}");
        self.open_package = Some(Arc::new(RwLock::new(pkg)));
        self.needs_saving = false;
        self.action_save.set_enabled(true);
        self.action_close.set_enabled(true);
        self.action_export_package.set_enabled(true);
        self.update_nav_menu()?;
        Ok(())
    }

    fn open(&mut self, path: PathBuf) -> evidenceangel::Result<()> {
        let pkg = EvidencePackage::open(path.clone())?;
        tracing::debug!("Package opened: {pkg:?}");
        self.open_package = Some(Arc::new(RwLock::new(pkg)));
        self.needs_saving = false;
        self.open_path = Some(path);
        self.action_save.set_enabled(true);
        self.action_close.set_enabled(true);
        self.action_export_package.set_enabled(true);
        self.update_nav_menu()?;
        Ok(())
    }

    fn close(&mut self) {
        self.open_package = None;
        self.open_path = None;
        self.needs_saving = false;
        self.action_save.set_enabled(false);
        self.action_close.set_enabled(false);
        self.action_export_package.set_enabled(false);
        tracing::debug!("Package closed.");
    }

    /// Update nav menu with test cases from the currently open package.
    fn update_nav_menu(&mut self) -> evidenceangel::Result<()> {
        let mut test_case_data = self.test_case_nav_factory.guard();
        test_case_data.clear();
        if let Some(pkg) = self.open_package.as_ref() {
            let pkg = pkg.read().unwrap();
            let mut ordered_cases = vec![];
            for case in pkg.test_case_iter()? {
                ordered_cases.push((
                    case.metadata().execution_datetime(),
                    NavFactoryInit {
                        id: *case.id(),
                        name: case.metadata().title().clone(),
                    },
                ));
            }
            // Sort
            ordered_cases.sort_by(|(a, _), (b, _)| b.cmp(a));
            for (_exdt, case) in ordered_cases {
                test_case_data.push_back(case);
            }
        }
        Ok(())
    }

    fn create_needs_saving_dialog(transient_for: &impl IsA<gtk::Window>) -> adw::MessageDialog {
        let dialog = adw::MessageDialog::builder()
            .transient_for(transient_for)
            .title(lang::lookup("needs-saving-title"))
            .heading(lang::lookup("needs-saving-title"))
            .body(lang::lookup("needs-saving-message"))
            .modal(true)
            .build();
        dialog.add_response("cancel", &lang::lookup("cancel"));
        dialog.add_response("no", &lang::lookup("needs-saving-no"));
        dialog.add_response("yes", &lang::lookup("needs-saving-yes"));
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");
        dialog.set_response_appearance("no", adw::ResponseAppearance::Destructive);
        dialog.set_response_appearance("yes", adw::ResponseAppearance::Suggested);

        dialog
    }

    fn get_package(&self) -> Option<Arc<RwLock<EvidencePackage>>> {
        self.open_package.as_ref().map(Clone::clone)
    }
}

#[derive(Clone, Debug)]
pub enum AppInput {
    Exit,
    NoOp,
    // Menu options
    NewFile,
    _NewFile,
    __NewFile,
    OpenFile,
    _OpenFile,
    __OpenFile(PathBuf),
    SaveFileThen(Box<AppInput>),
    OpenAboutDialog,

    CloseFileIfOpenThen(Box<AppInput>),
    _CloseFileIfOpenThen(Box<AppInput>, bool),
    _SetPathThen(PathBuf, Box<AppInput>),

    #[allow(private_interfaces)]
    /// `NavigateTo` ignores the index provided as part of [`OpenCase::Case`] and establishes
    /// it automatically.
    NavigateTo(OpenCase),
    DeleteCase(Uuid),
    CreateCaseAndSelect,
    SetMetadataTitle(String),
    SetMetadataDescription(String),
    CreateAuthor,
    _CreateAuthor(Author),
    DeleteAuthor(Author),

    SetTestCaseTitle(String),
    TrySetExecutionDateTime(String),
    ValidateExecutionDateTime(String),
    DeleteSelectedCase,
    _DeleteSelectedCase,
    AddTextEvidence,
    AddRichTextEvidence,
    AddHttpEvidence,
    AddImageEvidence,
    #[allow(dead_code)]
    AddFileEvidence,
    _AddEvidence(Evidence, Option<usize>),
    /// `InsertEvidenceAt` MUST NOT update the interface.
    InsertEvidenceAt(usize, Evidence),
    ReplaceEvidenceAt(DynamicIndex, Evidence),
    DeleteEvidenceAt(DynamicIndex, bool),
    _AddMedia(MediaFile),
    /// Show an error dialog.
    ShowError {
        title: String,
        message: String,
    },

    ExportPackage,
    _ExportPackage(String, PathBuf),
    ExportTestCase,
    _ExportTestCase(String, PathBuf),
    PasteEvidence,
    ShowToast(String),
    ReinstatePaste,
}

#[relm4::component(pub)]
impl Component for AppModel {
    type CommandOutput = ();
    type Input = AppInput;
    type Output = ();
    type Init = Option<PathBuf>;

    view! {
        #[root]
        adw::ApplicationWindow {
            set_title: Some(&lang::lookup("app-name")),
            set_width_request: 800,
            set_height_request: 600,

            connect_close_request[sender] => move |_| {
                sender.input(AppInput::CloseFileIfOpenThen(Box::new(AppInput::Exit)));
                gtk::glib::Propagation::Stop
            },

            #[name = "split_view"]
            adw::NavigationSplitView {
                #[wrap(Some)]
                set_sidebar = &adw::NavigationPage {
                    set_title: &lang::lookup("app-name"),

                    adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            pack_start = &gtk::Button {
                                add_css_class: "flat",
                                set_icon_name: relm4_icons::icon_names::PLUS,
                                set_tooltip: &lang::lookup("nav-create-case"),
                                #[watch]
                                set_sensitive: model.open_package.is_some(),
                                connect_clicked => AppInput::CreateCaseAndSelect,
                            },
                            pack_end = &gtk::MenuButton {
                                set_icon_name: relm4_icons::icon_names::MENU,
                                set_tooltip: &lang::lookup("header-menu"),
                                set_direction: gtk::ArrowType::Down,

                                #[wrap(Some)]
                                set_popover = &gtk::PopoverMenu::from_model(Some(&menu)) {
                                    set_position: gtk::PositionType::Bottom,
                                },
                            },
                        },

                        // Content
                        #[name = "nav_scrolled_window"]
                        gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_margin_horizontal: 8,
                                set_spacing: 2,
                                #[watch]
                                set_visible: model.open_package.is_some(),

                                #[name = "nav_metadata"]
                                gtk::Button {
                                    set_label: &lang::lookup("nav-metadata"),
                                    add_css_class: "flat",
                                    connect_clicked => AppInput::NavigateTo(OpenCase::Metadata),
                                },

                                #[local_ref]
                                test_case_list -> gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 2,
                                },
                            }
                        }
                    },
                },

                #[wrap(Some)]
                set_content = &adw::NavigationPage {
                    #[watch]
                    set_title: &format!("{} · {}", if let Some(pkg) = model.open_package.as_ref() {
                        pkg.read().unwrap().metadata().title().clone()
                    } else {
                        lang::lookup("title-no-package")
                    }, match model.open_case {
                        OpenCase::Nothing => lang::lookup("title-no-case"),
                        OpenCase::Metadata => lang::lookup("nav-metadata"),
                        OpenCase::Case { id, .. } => {
                            if let Some(pkg) = model.open_package.as_ref() {
                                if let Some(case) = pkg.read().unwrap().test_case(id).ok().flatten() {
                                    case.metadata().title().clone()
                                } else {
                                    // This is very briefly hit as a case is deleted
                                    lang::lookup("title-no-case")
                                }
                            } else {
                                // This is hit when a case is open and the "Open" button is selected again
                                lang::lookup("title-no-case")
                            }
                        },
                    }),

                    adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            #[wrap(Some)]
                            set_title_widget = &adw::WindowTitle {
                                #[watch]
                                set_title: &format!("{}{}", if model.needs_saving {
                                    "• ".to_string()
                                } else {
                                    String::new()
                                }, if let Some(pkg) = model.open_package.as_ref() {
                                    pkg.read().unwrap().metadata().title().clone()
                                } else {
                                    lang::lookup("title-no-package")
                                }),
                                #[watch]
                                set_subtitle: &match model.open_case {
                                    OpenCase::Nothing => lang::lookup("title-no-case"),
                                    OpenCase::Metadata => lang::lookup("nav-metadata"),
                                    OpenCase::Case { id, .. } => {
                                        if let Some(pkg) = model.open_package.as_ref() {
                                            if let Some(case) = pkg.read().unwrap().test_case(id).ok().flatten() {
                                                case.metadata().title().clone()
                                            } else {
                                                // This is very briefly hit as a case is deleted
                                                lang::lookup("title-no-case")
                                            }
                                        } else {
                                            // This is hit when a case is open and the "Open" button is selected again
                                            lang::lookup("title-no-case")
                                        }
                                    },
                                },
                            }
                        },

                        #[name = "toast_target"]
                        adw::ToastOverlay {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_margin_all: 16,

                                adw::StatusPage {
                                    set_title: &lang::lookup("nothing-open"),
                                    #[watch]
                                    set_description: Some(&if model.open_package.is_some() {
                                        lang::lookup("nothing-open-case-description")
                                    } else {
                                        lang::lookup("nothing-open-package-description")
                                    }),
                                    set_icon_name: Some(relm4_icons::icon_names::LIGHTBULB),
                                    #[watch]
                                    set_visible: model.open_case == OpenCase::Nothing,
                                    set_vexpand: true,
                                },

                                // Content
                                match model.open_case {
                                    OpenCase::Nothing => gtk::Box,
                                    OpenCase::Metadata => gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,

                                        // Generic Metadata
                                        adw::PreferencesGroup {
                                            set_title: &lang::lookup("metadata-group-title"),

                                            #[name = "metadata_title"]
                                            adw::EntryRow {
                                                set_title: &lang::lookup("metadata-title"),
                                                // TODO After adwaita 1.6 set_max_length: 30,

                                                connect_changed[sender] => move |entry| {
                                                    sender.input(AppInput::SetMetadataTitle(entry.text().to_string()));
                                                } @metadata_title_changed
                                            },

                                            #[name = "metadata_description"]
                                            adw::EntryRow {
                                                set_title: &lang::lookup("metadata-description"),

                                                connect_changed[sender] => move |entry| {
                                                    sender.input(AppInput::SetMetadataDescription(entry.text().to_string()));
                                                } @metadata_description_changed
                                            },

                                            #[name = "metadata_title_error_popover"]
                                            gtk::Popover {
                                                set_autohide: false,

                                                #[name = "metadata_title_error_popover_label"]
                                                gtk::Label {
                                                    set_text: &lang::lookup("toast-name-too-long"),
                                                    add_css_class: "error",
                                                }
                                            },
                                        },

                                        // Authors
                                        #[local_ref]
                                        authors_list -> adw::PreferencesGroup {
                                            set_title: &lang::lookup("metadata-authors"),
                                            set_margin_top: 16,
                                            #[wrap(Some)]
                                            set_header_suffix = &adw::Bin {
                                                gtk::Button {
                                                    set_icon_name: relm4_icons::icon_names::PLUS,
                                                    set_tooltip: &lang::lookup("author-create-title"),
                                                    add_css_class: "flat",

                                                    connect_clicked[sender] => move |_entry| {
                                                        sender.input(AppInput::CreateAuthor);
                                                    }
                                                }
                                            },
                                        }
                                    },
                                    OpenCase::Case { .. } => gtk::Box {
                                        #[name = "test_case_scrolled"]
                                        gtk::ScrolledWindow {
                                            set_hscrollbar_policy: gtk::PolicyType::Never,
                                            set_vexpand: true,

                                            gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,

                                                adw::PreferencesGroup {
                                                    set_title: &lang::lookup("test-group-title"),

                                                    #[name = "test_title"]
                                                    adw::EntryRow {
                                                        set_title: &lang::lookup("test-title"),
                                                        // TODO After adwaita 1.6 set_max_length: 30,

                                                        connect_changed[sender] => move |entry| {
                                                            sender.input(AppInput::SetTestCaseTitle(entry.text().to_string()));
                                                        } @test_title_changed_handler
                                                    },

                                                    #[name = "test_title_error_popover"]
                                                    gtk::Popover {
                                                        set_autohide: false,

                                                        #[name = "test_title_error_popover_label"]
                                                        gtk::Label {
                                                            set_text: &lang::lookup("toast-name-too-long"),
                                                            add_css_class: "error",
                                                        }
                                                    },

                                                    #[name = "test_execution"]
                                                    adw::EntryRow {
                                                        set_title: &lang::lookup("test-execution"),

                                                        connect_changed[sender] => move |entry| {
                                                            sender.input(AppInput::TrySetExecutionDateTime(entry.text().to_string()));
                                                        } @execution_time_changed_handler,

                                                        connect_changed[sender] => move |entry| {
                                                            sender.input(AppInput::ValidateExecutionDateTime(entry.text().to_string()));
                                                        }
                                                    },

                                                    #[name = "test_execution_error_popover"]
                                                    gtk::Popover {
                                                        set_autohide: false,

                                                        #[name = "test_execution_error_popover_label"]
                                                        gtk::Label {
                                                            set_text: &lang::lookup("toast-name-too-long"),
                                                            add_css_class: "error",
                                                        }
                                                    },
                                                },

                                                // Test Case Screen
                                                #[local_ref]
                                                evidence_list -> gtk::Box {
                                                    set_orientation: gtk::Orientation::Vertical,
                                                    set_spacing: 8,
                                                    set_margin_top: 8,
                                                },

                                                gtk::Box {
                                                    set_orientation: gtk::Orientation::Vertical,
                                                    set_hexpand: true,
                                                    set_halign: gtk::Align::Fill,
                                                    set_spacing: 2,

                                                    add_controller = gtk::DropTarget {
                                                        set_actions: gtk::gdk::DragAction::MOVE,
                                                        set_types: &[BoxedEvidenceJson::static_type()],

                                                        connect_drop[sender] => move |_slf, val, _x, _y| {
                                                            tracing::debug!("Dropped type: {:?}", val.type_());
                                                            if let Ok(data) = val.get::<BoxedEvidenceJson>() {
                                                                let ev = data.inner();
                                                                tracing::debug!("Dropped data: {ev:?}");
                                                                sender.input(AppInput::_AddEvidence(ev, None));
                                                                return true;
                                                            }
                                                            false
                                                        },
                                                    },

                                                    gtk::Box {
                                                        set_orientation: gtk::Orientation::Horizontal,
                                                        set_margin_top: 8,
                                                        set_halign: gtk::Align::Center,
                                                        //set_spacing: 8,
                                                        add_css_class: "linked",

                                                        gtk::Button {
                                                            connect_clicked => AppInput::AddTextEvidence,
                                                            add_css_class: "pill",

                                                            adw::ButtonContent {
                                                                set_icon_name: relm4_icons::icon_names::PLUS,
                                                                set_label: &lang::lookup("evidence-text"),
                                                            }
                                                        },
                                                        gtk::Button {
                                                            connect_clicked => AppInput::AddRichTextEvidence,
                                                            add_css_class: "pill",

                                                            adw::ButtonContent {
                                                                set_icon_name: relm4_icons::icon_names::PLUS,
                                                                set_label: &lang::lookup("evidence-richtext"),
                                                            }
                                                        },
                                                        gtk::Button {
                                                            connect_clicked => AppInput::AddHttpEvidence,
                                                            add_css_class: "pill",

                                                            adw::ButtonContent {
                                                                set_icon_name: relm4_icons::icon_names::PLUS,
                                                                set_label: &lang::lookup("evidence-http"),
                                                            }
                                                        },
                                                        gtk::Button {
                                                            connect_clicked => AppInput::AddImageEvidence,
                                                            add_css_class: "pill",

                                                            adw::ButtonContent {
                                                                set_icon_name: relm4_icons::icon_names::PLUS,
                                                                set_label: &lang::lookup("evidence-image"),
                                                            }
                                                        },
                                                        gtk::Button {
                                                            connect_clicked => AppInput::AddFileEvidence,
                                                            add_css_class: "pill",

                                                            adw::ButtonContent {
                                                                set_icon_name: relm4_icons::icon_names::PLUS,
                                                                set_label: &lang::lookup("evidence-file"),
                                                            }
                                                        },
                                                    },

                                                    gtk::Separator {
                                                        add_css_class: "spacer",
                                                    },

                                                    gtk::Button {
                                                        add_css_class: "pill",
                                                        add_css_class: "destructive-action",
                                                        set_margin_top: 8,
                                                        set_halign: gtk::Align::Center,

                                                        connect_clicked => AppInput::DeleteSelectedCase,

                                                        adw::ButtonContent {
                                                            set_icon_name: relm4_icons::icon_names::DELETE_FILLED,
                                                            set_label: &lang::lookup("nav-delete-case"),
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                    },
                                }
                            },
                        }
                    },
                },
            },
        },
    }

    menu! {
        menu: {
            &lang::lookup("header-new") => NewAction,
            &lang::lookup("header-open") => OpenAction,
            &lang::lookup("header-save") => SaveAction,
            &lang::lookup("header-close") => CloseAction,
            section! {
                &lang::lookup("header-paste-evidence") => PasteEvidenceAction,
            },
            section! {
                &lang::lookup("header-export-package") => ExportPackageAction,
                &lang::lookup("header-export-test-case") => ExportTestCaseAction,
            },
            section! {
                &lang::lookup("header-about") => AboutAction,
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let sender_c = sender.clone();
        let action_new: RelmAction<NewAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::CloseFileIfOpenThen(Box::new(AppInput::NewFile)));
        });
        relm4::main_application().set_accelerators_for_action::<NewAction>(&["<primary>N"]);

        let sender_c = sender.clone();
        let action_open: RelmAction<OpenAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::CloseFileIfOpenThen(Box::new(AppInput::OpenFile)));
        });
        relm4::main_application().set_accelerators_for_action::<OpenAction>(&["<primary>O"]);

        let sender_c = sender.clone();
        let action_save: RelmAction<SaveAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::SaveFileThen(Box::new(AppInput::NoOp)));
        });
        action_save.set_enabled(false);
        relm4::main_application().set_accelerators_for_action::<SaveAction>(&["<primary>S"]);

        let sender_c = sender.clone();
        let action_close: RelmAction<CloseAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::CloseFileIfOpenThen(Box::new(AppInput::NoOp)));
        });
        action_close.set_enabled(false);
        relm4::main_application().set_accelerators_for_action::<CloseAction>(&["<primary>W"]);

        let sender_c = sender.clone();
        let action_paste_evidence: RelmAction<PasteEvidenceAction> =
            RelmAction::new_stateless(move |_| {
                sender_c.input(AppInput::PasteEvidence);
            });
        action_paste_evidence.set_enabled(false);
        relm4::main_application()
            .set_accelerators_for_action::<PasteEvidenceAction>(&["<primary><shift>V"]);

        let sender_c = sender.clone();
        let action_export_package: RelmAction<ExportPackageAction> =
            RelmAction::new_stateless(move |_| {
                sender_c.input(AppInput::ExportPackage);
            });
        action_export_package.set_enabled(false);
        relm4::main_application()
            .set_accelerators_for_action::<ExportPackageAction>(&["<primary>E"]);

        let sender_c = sender.clone();
        let action_export_test_case: RelmAction<ExportTestCaseAction> =
            RelmAction::new_stateless(move |_| {
                sender_c.input(AppInput::ExportTestCase);
            });
        action_export_test_case.set_enabled(false);
        relm4::main_application()
            .set_accelerators_for_action::<ExportTestCaseAction>(&["<primary><shift>E"]);

        let sender_c = sender.clone();
        let action_about: RelmAction<AboutAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::OpenAboutDialog);
        });
        relm4::main_application().set_accelerators_for_action::<AboutAction>(&["F1"]);

        let mut group = RelmActionGroup::<MenuActionGroup>::new();
        group.add_action(action_new);
        group.add_action(action_open);
        group.add_action(action_save.clone());
        group.add_action(action_close.clone());
        group.add_action(action_about);
        group.add_action(action_paste_evidence.clone());
        group.add_action(action_export_package.clone());
        group.add_action(action_export_test_case.clone());
        group.register_for_widget(&root);

        let model = AppModel {
            open_package: None,
            open_path: None,
            open_case: OpenCase::Nothing,
            needs_saving: false,

            action_save,
            action_export_package,
            action_export_test_case,
            action_close,
            action_paste_evidence,

            latest_error_dlg: None,
            latest_new_author_dlg: None,
            latest_add_evidence_image_dlg: None,
            latest_add_evidence_file_dlg: None,
            latest_export_dlg: None,
            latest_delete_toasts: vec![],

            test_case_nav_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |output| match output {
                    NavFactoryOutput::NavigateTo(index, id) => {
                        AppInput::NavigateTo(OpenCase::Case { index, id })
                    }
                },
            ),
            authors_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |output| match output {
                    AuthorFactoryOutput::DeleteAuthor(author) => AppInput::DeleteAuthor(author),
                },
            ),
            test_evidence_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    EvidenceFactoryOutput::UpdateEvidence(at, new_ev) => {
                        AppInput::ReplaceEvidenceAt(at, new_ev)
                    }
                    EvidenceFactoryOutput::InsertEvidenceAt(index, offset, ev) => {
                        let idx_with_offset = match offset.cmp(&0isize) {
                            std::cmp::Ordering::Greater => {
                                index.current_index().saturating_add(offset as usize)
                            }
                            std::cmp::Ordering::Less => {
                                index.current_index().saturating_sub((-offset) as usize)
                            }
                            std::cmp::Ordering::Equal => index.current_index(),
                        };
                        AppInput::InsertEvidenceAt(idx_with_offset, ev)
                    }
                    EvidenceFactoryOutput::DeleteEvidence(at, user_triggered) => {
                        AppInput::DeleteEvidenceAt(at, user_triggered)
                    }
                },
            ),
        };

        let test_case_list = model.test_case_nav_factory.widget();
        let authors_list = model.authors_factory.widget();
        let evidence_list = model.test_evidence_factory.widget();
        let widgets = view_output!();
        if cfg!(debug_assertions) {
            root.add_css_class("devel");
        }

        if let Some(file) = init {
            sender.input(AppInput::__OpenFile(file));
            root.set_visible(true);
        }

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        tracing::debug!("Handling event: {message:?}");
        match message {
            AppInput::Exit => {
                relm4::main_application().quit();
            }
            AppInput::NoOp => (),
            AppInput::NewFile => {
                sender.input(AppInput::CloseFileIfOpenThen(Box::new(AppInput::_NewFile)));
            }
            AppInput::_NewFile => {
                // Show file selection dialog
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("header-save"))
                    .filters(&filter::filter_list(vec![filter::packages()]))
                    .build();

                let sender_c = sender.clone();
                dialog.save(
                    Some(&root.toplevel_window().unwrap()),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let path = file.path().unwrap();
                            // Add extension
                            // Open this package
                            sender_c.input(AppInput::_SetPathThen(
                                path.with_extension("evp"),
                                Box::new(AppInput::__NewFile),
                            ));
                        }
                    },
                );
            }
            AppInput::__NewFile => {
                // Set default name, author, execution date and path.
                sender.input(AppInput::NavigateTo(OpenCase::Nothing));
                if let Err(e) = self.open_new() {
                    let error_dlg = ErrorDialogModel::builder()
                        .launch(ErrorDialogInit {
                            title: Box::new(lang::lookup("error-failed-new-title")),
                            body: Box::new(lang::lookup_with_args(
                                "error-failed-new-body",
                                &lang_args!("error", e.to_string()),
                            )),
                        })
                        .forward(sender.input_sender(), |msg| match msg {});
                    error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                    self.latest_error_dlg = Some(error_dlg);
                }
            }
            AppInput::OpenFile => {
                sender.input(AppInput::CloseFileIfOpenThen(Box::new(AppInput::_OpenFile)));
            }
            AppInput::_OpenFile => {
                // Show file selection dialog
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("header-open"))
                    .filters(&filter::filter_list(vec![filter::packages()]))
                    .build();

                let sender_c = sender.clone();
                dialog.open(
                    Some(&root.toplevel_window().unwrap()),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let path = file.path().unwrap();
                            // Open this package
                            sender_c.input(AppInput::__OpenFile(path));
                        }
                    },
                );
            }
            AppInput::__OpenFile(path) => {
                if let Err(e) = self.open(path) {
                    let error_dlg = ErrorDialogModel::builder()
                        .launch(ErrorDialogInit {
                            title: Box::new(lang::lookup("error-failed-open-title")),
                            body: Box::new(lang::lookup_with_args(
                                "error-failed-open-body",
                                &lang_args!("error", e.to_string()),
                            )),
                        })
                        .forward(sender.input_sender(), |msg| match msg {});
                    error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                    self.latest_error_dlg = Some(error_dlg);
                }
            }
            AppInput::SaveFileThen(then) => {
                if let Some(package) = self.get_package() {
                    self.latest_delete_toasts
                        .iter()
                        .for_each(adw::Toast::dismiss);
                    if let Err(e) = package.write().unwrap().save() {
                        // Show error dialog
                        let error_dlg = ErrorDialogModel::builder()
                            .launch(ErrorDialogInit {
                                title: Box::new(lang::lookup("error-failed-save-title")),
                                body: Box::new(lang::lookup_with_args(
                                    "error-failed-save-body",
                                    &lang_args!("error", e.to_string()),
                                )),
                            })
                            .forward(sender.input_sender(), |msg| match msg {});
                        error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                        self.latest_error_dlg = Some(error_dlg);
                    } else {
                        let toast = adw::Toast::new(&lang::lookup("toast-saved"));
                        toast.set_timeout(3);
                        widgets.toast_target.add_toast(toast);
                        self.needs_saving = false;
                        sender.input(*then);
                    }
                }
            }
            AppInput::CloseFileIfOpenThen(then) => {
                // Propose to save if needed
                if self.needs_saving {
                    // Show dialog
                    let dlg = Self::create_needs_saving_dialog(root);
                    let sender_c = sender.clone();
                    dlg.connect_response(None, move |dlg, res| {
                        if res == "no" {
                            // discard
                            sender_c.input(AppInput::_CloseFileIfOpenThen(then.clone(), false));
                        } else if res == "yes" {
                            // save
                            sender_c.input(AppInput::_CloseFileIfOpenThen(then.clone(), true));
                        }
                        dlg.close();
                    });
                    dlg.set_visible(true);
                } else {
                    sender.input(AppInput::NavigateTo(OpenCase::Nothing));
                    self.close();
                    sender.input(*then);
                }
            }
            AppInput::_CloseFileIfOpenThen(then, save) => {
                if save {
                    sender.input(AppInput::SaveFileThen(Box::new(
                        AppInput::_CloseFileIfOpenThen(then, false),
                    )));
                } else {
                    sender.input(AppInput::NavigateTo(OpenCase::Nothing));
                    self.close();
                    sender.input(*then);
                }
            }
            AppInput::OpenAboutDialog => {
                // Open about dialog
                crate::about::AppAbout::builder()
                    .transient_for(root)
                    .launch(())
                    .widget()
                    .set_visible(true);
            }
            AppInput::_SetPathThen(path, then) => {
                self.open_path = Some(path);
                sender.input(*then);
            }

            AppInput::NavigateTo(target) => {
                self.open_case = target;

                // First unselect all cases
                widgets.nav_metadata.set_has_frame(false);
                self.test_case_nav_factory
                    .broadcast(NavFactoryInput::ShowAsSelected(false));
                self.action_export_test_case.set_enabled(false);
                self.action_paste_evidence.set_enabled(false);

                // Then select the new case
                match target {
                    OpenCase::Metadata => {
                        // Update fields
                        widgets
                            .metadata_title
                            .block_signal(&widgets.metadata_title_changed);
                        widgets.metadata_title.set_text(
                            &self
                                .open_package
                                .as_ref()
                                .map(|pkg| pkg.read().unwrap().metadata().title().clone())
                                .expect("Cannot navigate to metadata when no package is open"),
                        );
                        widgets
                            .metadata_title
                            .unblock_signal(&widgets.metadata_title_changed);

                        widgets
                            .metadata_description
                            .block_signal(&widgets.metadata_description_changed);
                        widgets.metadata_description.set_text(
                            &self
                                .open_package
                                .as_ref()
                                .map(|pkg| pkg.read().unwrap().metadata().description().clone())
                                .expect("Cannot navigate to metadata when no package is open")
                                .unwrap_or_default(),
                        );
                        widgets
                            .metadata_description
                            .unblock_signal(&widgets.metadata_description_changed);

                        let mut authors = self.authors_factory.guard();
                        authors.clear();
                        let pkg_authors = self
                            .open_package
                            .as_ref()
                            .map(|pkg| pkg.read().unwrap().metadata().authors().clone())
                            .expect("Cannot navigate to metadata when no package is open");
                        for author in pkg_authors {
                            authors.push_back(author);
                        }
                        widgets.nav_metadata.set_has_frame(true);
                    }
                    OpenCase::Case { id, .. } => {
                        // Determine own index
                        let mut ordered_cases = vec![];
                        let index = {
                            let pkg = self.open_package.as_ref().unwrap();
                            let pkg = pkg.read().unwrap();
                            for case in pkg.test_case_iter().unwrap() {
                                ordered_cases
                                    .push((case.metadata().execution_datetime(), *case.id()));
                            }
                            // Sort
                            ordered_cases.sort_by(|(a, _), (b, _)| b.cmp(a));
                            ordered_cases
                                .iter()
                                .position(|(_dt, ocid)| *ocid == id)
                                .unwrap()
                        };
                        self.open_case = OpenCase::Case { index, id };
                        self.action_export_test_case.set_enabled(true);
                        self.action_paste_evidence.set_enabled(true);

                        self.test_case_nav_factory
                            .send(index, NavFactoryInput::ShowAsSelected(true));

                        let mut new_evidence = vec![];
                        if let Some(pkg) = self.get_package() {
                            if let Some(tc) = pkg.read().unwrap().test_case(id).ok().flatten() {
                                // Update test case metadata on screen
                                widgets
                                    .test_title
                                    .block_signal(&widgets.test_title_changed_handler);
                                widgets.test_title.set_text(tc.metadata().title());
                                widgets
                                    .test_title
                                    .unblock_signal(&widgets.test_title_changed_handler);
                                widgets
                                    .test_execution
                                    .block_signal(&widgets.execution_time_changed_handler);
                                widgets.test_execution.set_text(&format!(
                                    "{}",
                                    tc.metadata()
                                        .execution_datetime()
                                        .format("%Y-%m-%d %H:%M:%S")
                                ));
                                widgets
                                    .test_execution
                                    .unblock_signal(&widgets.execution_time_changed_handler);

                                for ev in tc.evidence() {
                                    new_evidence.push(EvidenceFactoryInit {
                                        evidence: ev.clone(),
                                        package: pkg.clone(),
                                    });
                                }
                            }
                        }

                        // This MUST be delayed so that the RwLock over the EvidencePackage is no longer in read mode.
                        // Otherwise, images cannot be loaded from media (as they need a write lock over the package).
                        let mut evidence = self.test_evidence_factory.guard();
                        evidence.clear();
                        for ev in new_evidence {
                            evidence.push_back(ev);
                        }
                    }
                    OpenCase::Nothing => (),
                }
            }
            AppInput::CreateCaseAndSelect => {
                if self.open_package.is_none() {
                    return;
                }

                let mut case_id = Uuid::default();
                if let Some(pkg) = self.get_package() {
                    let mut pkg = pkg.write().unwrap();
                    let case = pkg
                        .create_test_case(lang::lookup("default-case-title"))
                        .unwrap(); // doesn't fail
                    case_id = *case.id();
                }

                // Add case to navigation
                self.update_nav_menu().unwrap(); // doesn't fail
                self.needs_saving = true;

                // Switch to case
                sender.input(AppInput::NavigateTo(OpenCase::Case {
                    // index will be calculated by NavigateTo
                    index: 0,
                    id: case_id,
                }));

                // Move to top of list
                let adj = widgets.nav_scrolled_window.vadjustment();
                adj.set_value(adj.lower());
                widgets.nav_scrolled_window.set_vadjustment(Some(&adj));
            }
            AppInput::SetMetadataTitle(new_title) => {
                if new_title.trim().is_empty() {
                    widgets.metadata_title.add_css_class("error");
                    widgets
                        .metadata_title_error_popover_label
                        .set_text(&lang::lookup("toast-name-cant-be-empty"));
                    widgets.metadata_title_error_popover.set_visible(true);
                } else if new_title.len() <= 30 {
                    widgets.metadata_title.remove_css_class("error");
                    widgets.metadata_title_error_popover.set_visible(false);
                    if let Some(pkg) = self.get_package() {
                        pkg.write().unwrap().metadata_mut().set_title(new_title);
                        self.needs_saving = true;
                    }
                } else {
                    widgets.metadata_title.add_css_class("error");
                    widgets
                        .metadata_title_error_popover_label
                        .set_text(&lang::lookup("toast-name-too-long"));
                    widgets.metadata_title_error_popover.set_visible(true);
                }
            }
            AppInput::SetMetadataDescription(new_desc) => {
                if let Some(pkg) = self.get_package() {
                    pkg.write().unwrap().metadata_mut().set_description(
                        if new_desc.trim().is_empty() {
                            None
                        } else {
                            Some(new_desc)
                        },
                    );
                    self.needs_saving = true;
                }
            }
            AppInput::DeleteCase(id) => {
                if let Some(pkg) = self.get_package() {
                    if let Err(e) = pkg.write().unwrap().delete_test_case(id) {
                        let error_dlg = ErrorDialogModel::builder()
                            .launch(ErrorDialogInit {
                                title: Box::new(lang::lookup("error-failed-delete-case-title")),
                                body: Box::new(lang::lookup_with_args(
                                    "error-failed-delete-case-body",
                                    &lang_args!("error", e.to_string()),
                                )),
                            })
                            .forward(sender.input_sender(), |msg| match msg {});
                        error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                        self.latest_error_dlg = Some(error_dlg);
                    }
                    self.update_nav_menu().unwrap(); // doesn't fail
                    sender.input(AppInput::NavigateTo(OpenCase::Metadata));
                    self.needs_saving = true;
                }
            }
            AppInput::CreateAuthor => {
                let new_author_dlg = NewAuthorDialogModel::builder().launch(()).forward(
                    sender.input_sender(),
                    |msg| match msg {
                        NewAuthorOutput::CreateAuthor(author) => AppInput::_CreateAuthor(author),
                    },
                );
                new_author_dlg.emit(NewAuthorInput::Present(root.clone()));
                self.latest_new_author_dlg = Some(new_author_dlg);
            }
            AppInput::_CreateAuthor(author) => {
                if let Some(pkg) = self.get_package() {
                    pkg.write()
                        .unwrap()
                        .metadata_mut()
                        .authors_mut()
                        .push(author.clone());
                    self.needs_saving = true;
                    // Add to author list
                    let mut authors = self.authors_factory.guard();
                    authors.push_back(author);
                }
            }
            AppInput::DeleteAuthor(author) => {
                if let Some(pkg) = self.get_package() {
                    let idx = pkg
                        .read()
                        .unwrap()
                        .metadata()
                        .authors()
                        .iter()
                        .position(|a| *a == author)
                        .unwrap();
                    pkg.write()
                        .unwrap()
                        .metadata_mut()
                        .authors_mut()
                        .remove(idx);
                    self.needs_saving = true;
                    // refresh author list
                    let mut authors = self.authors_factory.guard();
                    authors.remove(idx);
                }
            }
            AppInput::SetTestCaseTitle(new_title) => {
                if new_title.trim().is_empty() {
                    widgets.test_title.add_css_class("error");
                    widgets
                        .test_title_error_popover_label
                        .set_text(&lang::lookup("toast-name-cant-be-empty"));
                    widgets.test_title_error_popover.set_visible(true);
                } else if new_title.len() <= 30 {
                    widgets.test_title.remove_css_class("error");
                    widgets.test_title_error_popover.set_visible(false);
                    if let OpenCase::Case { index, id, .. } = &self.open_case {
                        if let Some(pkg) = self.get_package() {
                            if let Some(tc) = pkg.write().unwrap().test_case_mut(*id).ok().flatten()
                            {
                                tc.metadata_mut().set_title(new_title.clone());
                                self.needs_saving = true;
                                self.test_case_nav_factory
                                    .send(*index, NavFactoryInput::UpdateTitle(new_title));
                            }
                        }
                    }
                } else {
                    widgets.test_title.add_css_class("error");
                    widgets
                        .test_title_error_popover_label
                        .set_text(&lang::lookup("toast-name-too-long"));
                    widgets.test_title_error_popover.set_visible(true);
                }
            }
            AppInput::TrySetExecutionDateTime(new_exec_time) => {
                match parse_datetime::parse_datetime_at_date(chrono::Local::now(), new_exec_time) {
                    Ok(dt) => {
                        tracing::debug!("Setting exec date time {dt}");

                        if let OpenCase::Case { id, .. } = &self.open_case {
                            if let Some(pkg) = self.get_package() {
                                if let Some(tc) =
                                    pkg.write().unwrap().test_case_mut(*id).ok().flatten()
                                {
                                    tc.metadata_mut().set_execution_datetime(dt);
                                    self.needs_saving = true;
                                }

                                // Fix for #59, before reordable TCs are implemented as part of #47.
                                self.update_nav_menu().unwrap(); // doesn't fail
                            }
                        }
                    }
                    Err(_e) => {
                        // Do nothing, validation is handled separately
                    }
                }
            }
            AppInput::ValidateExecutionDateTime(new_exec_time) => {
                match parse_datetime::parse_datetime_at_date(chrono::Local::now(), new_exec_time) {
                    Ok(_dt) => {
                        widgets.test_execution.remove_css_class("error");
                        widgets.test_execution_error_popover.set_visible(false);
                    }
                    Err(e) => {
                        widgets.test_execution.add_css_class("error");
                        widgets
                            .test_execution_error_popover_label
                            .set_text(&e.to_string());
                        widgets.test_execution_error_popover.set_visible(true);
                    }
                }
            }
            AppInput::DeleteSelectedCase => {
                if let Some(pkg) = &self.open_package {
                    let pkg = pkg.read().unwrap();

                    if let OpenCase::Case { id, .. } = &self.open_case {
                        let case = pkg
                            .test_case(*id)
                            .ok()
                            .flatten()
                            .expect("opened case must exist");

                        let dialog = adw::MessageDialog::builder()
                            .transient_for(root)
                            .title(lang::lookup_with_args(
                                "delete-case-title",
                                &lang_args!("name", case.metadata().title()),
                            ))
                            .heading(lang::lookup_with_args(
                                "delete-case-title",
                                &lang_args!("name", case.metadata().title()),
                            ))
                            .body(lang::lookup_with_args(
                                "delete-case-message",
                                &lang_args!("name", case.metadata().title()),
                            ))
                            .modal(true)
                            .build();
                        dialog.add_response("cancel", &lang::lookup("cancel"));
                        dialog.add_response("delete", &lang::lookup("delete-case-affirm"));
                        dialog.set_default_response(Some("cancel"));
                        dialog.set_close_response("cancel");
                        dialog.set_response_appearance(
                            "delete",
                            adw::ResponseAppearance::Destructive,
                        );
                        dialog.set_visible(true);

                        let sender_c = sender.clone();
                        dialog.connect_response(None, move |_dlg, res| {
                            if res == "delete" {
                                sender_c.input(AppInput::_DeleteSelectedCase);
                            }
                        });
                    }
                }
            }
            AppInput::_DeleteSelectedCase => {
                if let OpenCase::Case { id, .. } = &self.open_case {
                    sender.input(AppInput::DeleteCase(*id));
                }
            }
            AppInput::AddTextEvidence => {
                sender.input(AppInput::_AddEvidence(
                    Evidence::new(
                        EvidenceKind::Text,
                        EvidenceData::Text {
                            content: String::new(),
                        },
                    ),
                    None,
                ));
            }
            AppInput::AddRichTextEvidence => {
                sender.input(AppInput::_AddEvidence(
                    Evidence::new(
                        EvidenceKind::RichText,
                        EvidenceData::Text {
                            content: String::new(),
                        },
                    ),
                    None,
                ));
            }
            AppInput::AddHttpEvidence => {
                sender.input(AppInput::_AddEvidence(
                    Evidence::new(
                        EvidenceKind::Http,
                        EvidenceData::Base64 { data: vec![0x1e] },
                    ),
                    None,
                ));
            }
            AppInput::AddImageEvidence => {
                let add_evidence_image_dlg = AddImageEvidenceDialogModel::builder()
                    .launch(self.get_package().unwrap())
                    .forward(sender.input_sender(), |msg| match msg {
                        AddEvidenceOutput::AddEvidence(ev) => AppInput::_AddEvidence(ev, None),
                        AddEvidenceOutput::Error { title, message } => {
                            AppInput::ShowError { title, message }
                        }
                        AddEvidenceOutput::Closed => AppInput::ReinstatePaste,
                    });
                add_evidence_image_dlg.emit(AddEvidenceInput::Present(root.clone()));
                self.latest_add_evidence_image_dlg = Some(add_evidence_image_dlg);
                self.action_paste_evidence.set_enabled(false);
            }
            AppInput::AddFileEvidence => {
                let add_evidence_file_dlg = AddFileEvidenceDialogModel::builder()
                    .launch(self.get_package().unwrap())
                    .forward(sender.input_sender(), |msg| match msg {
                        AddEvidenceOutput::AddEvidence(ev) => AppInput::_AddEvidence(ev, None),
                        AddEvidenceOutput::Error { title, message } => {
                            AppInput::ShowError { title, message }
                        }
                        AddEvidenceOutput::Closed => AppInput::ReinstatePaste,
                    });
                add_evidence_file_dlg.emit(AddEvidenceInput::Present(root.clone()));
                self.latest_add_evidence_file_dlg = Some(add_evidence_file_dlg);
                self.action_paste_evidence.set_enabled(false);
            }
            AppInput::ReinstatePaste => self.action_paste_evidence.set_enabled(true),
            AppInput::_AddEvidence(ev, maybe_pos) => {
                if let Some(pkg) = self.get_package() {
                    if let OpenCase::Case { id, .. } = &self.open_case {
                        {
                            let mut pkg_guard = pkg.write().unwrap();
                            let evidence = pkg_guard
                                .test_case_mut(*id)
                                .ok()
                                .flatten()
                                .unwrap()
                                .evidence_mut();
                            if let Some(pos) = &maybe_pos {
                                evidence.insert(*pos, ev.clone());
                            } else {
                                evidence.push(ev.clone());
                            }
                        }
                        self.needs_saving = true;
                        // update evidence
                        let mut evidence = self.test_evidence_factory.guard();

                        if let Some(pos) = &maybe_pos {
                            evidence.insert(
                                *pos,
                                EvidenceFactoryInit {
                                    evidence: ev,
                                    package: pkg.clone(),
                                },
                            );
                        } else {
                            evidence.push_back(EvidenceFactoryInit {
                                evidence: ev,
                                package: pkg.clone(),
                            });
                            // scroll to the bottom
                            let adj = widgets.test_case_scrolled.vadjustment();
                            adj.set_value(adj.upper());
                            widgets.test_case_scrolled.set_vadjustment(Some(&adj));
                        }
                    }
                }
            }
            AppInput::InsertEvidenceAt(at, ev) => {
                if let Some(pkg) = self.get_package() {
                    if let OpenCase::Case { id, .. } = &self.open_case {
                        let at = {
                            // This block prevents a panic when only one item is present
                            let mut pkg_w = pkg.write().unwrap();
                            let evidence = pkg_w
                                .test_case_mut(*id)
                                .ok()
                                .flatten()
                                .unwrap()
                                .evidence_mut();
                            let at = at.min(evidence.len());
                            evidence.insert(at, ev.clone());
                            at
                        };
                        self.needs_saving = true;
                        // update evidence
                        let mut tef = self.test_evidence_factory.guard();
                        tef.insert(
                            at,
                            EvidenceFactoryInit {
                                package: pkg.clone(),
                                evidence: ev,
                            },
                        );
                    }
                }
            }
            AppInput::ReplaceEvidenceAt(at, new_ev) => {
                if let Some(pkg) = self.get_package() {
                    if let OpenCase::Case { id, .. } = &self.open_case {
                        let mut pkg = pkg.write().unwrap();
                        let evidence = pkg
                            .test_case_mut(*id)
                            .ok()
                            .flatten()
                            .unwrap()
                            .evidence_mut();
                        if let Some(ev) = evidence.get_mut(at.current_index()) {
                            *ev = new_ev;
                        }
                        self.needs_saving = true;
                        // No need to update here as this is only triggered by things that visually change anyway
                    }
                }
            }
            AppInput::DeleteEvidenceAt(at, user_triggered) => {
                if let Some(pkg) = self.get_package() {
                    if let OpenCase::Case { id, .. } = &self.open_case {
                        let mut pkg = pkg.write().unwrap();
                        let evidence = pkg
                            .test_case_mut(*id)
                            .ok()
                            .flatten()
                            .unwrap()
                            .evidence_mut();
                        let ev = evidence.remove(at.current_index());
                        self.needs_saving = true;
                        // update evidence
                        let mut tef = self.test_evidence_factory.guard();
                        let index = at.current_index();
                        tef.remove(at.current_index());

                        if user_triggered {
                            let toast = adw::Toast::new(&lang::lookup("toast-evidence-deleted"));
                            toast.set_timeout(5);
                            toast.set_button_label(Some(&lang::lookup("undo")));
                            let sender = sender.clone();
                            toast.connect_button_clicked(move |_| {
                                sender.input(AppInput::_AddEvidence(ev.clone(), Some(index)));
                            });

                            widgets.toast_target.add_toast(toast.clone());
                            self.latest_delete_toasts.push(toast);
                        }

                        // Fix for #73
                        widgets.test_case_scrolled.grab_focus();
                    }
                }
            }
            AppInput::_AddMedia(media) => {
                if let Some(pkg) = self.get_package() {
                    // unwraps here cannot fail
                    pkg.write().unwrap().add_media(media).unwrap();
                }
            }
            AppInput::ShowError { title, message } => {
                let error_dlg = ErrorDialogModel::builder()
                    .launch(ErrorDialogInit {
                        title: Box::new(title),
                        body: Box::new(message),
                    })
                    .forward(sender.input_sender(), |msg| match msg {});
                error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                self.latest_error_dlg = Some(error_dlg);
            }
            AppInput::ExportPackage => {
                if self.open_package.is_none() {
                    return;
                }
                let export_dlg = ExportDialogModel::builder()
                    .launch(ExportDialogInit {
                        package_name: self
                            .open_package
                            .as_ref()
                            .unwrap()
                            .read()
                            .unwrap()
                            .metadata()
                            .title()
                            .clone(),
                        package_path: self.open_path.clone().unwrap(),
                        test_case_name: None,
                    })
                    .forward(sender.input_sender(), |msg| match msg {
                        ExportOutput::Export { format, path } => {
                            AppInput::_ExportPackage(format, path)
                        }
                    });
                export_dlg.emit(ExportInput::Present(root.clone()));
                self.latest_export_dlg = Some(export_dlg);
            }
            AppInput::ExportTestCase => {
                if self.open_package.is_none() {
                    return;
                }
                if let OpenCase::Case { id, .. } = &self.open_case {
                    let pkg = self.open_package.as_ref().unwrap().read().unwrap();
                    let case_name = pkg
                        .test_case(*id)
                        .map(|r| r.map(|c| c.metadata().title().clone()))
                        .ok()
                        .flatten()
                        .unwrap_or_default();
                    let export_dlg = ExportDialogModel::builder()
                        .launch(ExportDialogInit {
                            package_name: pkg.metadata().title().clone(),
                            package_path: self.open_path.clone().unwrap(),
                            test_case_name: Some(case_name),
                        })
                        .forward(sender.input_sender(), |msg| match msg {
                            ExportOutput::Export { format, path } => {
                                AppInput::_ExportTestCase(format, path)
                            }
                        });
                    export_dlg.emit(ExportInput::Present(root.clone()));
                    self.latest_export_dlg = Some(export_dlg);
                }
            }
            AppInput::_ExportPackage(format, path) => {
                if let Some(pkg) = &self.open_package {
                    let mut pkg = pkg.write().unwrap();
                    if let Err(e) = match format.as_str() {
                        "html document" => HtmlExporter.export_package(&mut pkg, path.clone()),
                        "excel workbook" => ExcelExporter.export_package(&mut pkg, path.clone()),
                        "zip archive of files" => {
                            ZipOfFilesExporter.export_package(&mut pkg, path.clone())
                        }
                        _ => {
                            tracing::error!("Invalid format specified.");
                            Ok(())
                        }
                    } {
                        // Show error dialog
                        let error_dlg = ErrorDialogModel::builder()
                            .launch(ErrorDialogInit {
                                title: Box::new(lang::lookup("export-error-failed-title")),
                                body: Box::new(lang::lookup_with_args(
                                    "export-error-failed-message",
                                    &lang_args!("error", e.to_string()),
                                )),
                            })
                            .forward(sender.input_sender(), |msg| match msg {});
                        error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                        self.latest_error_dlg = Some(error_dlg);
                    } else {
                        let toast = adw::Toast::new(&lang::lookup("toast-export-complete"));
                        toast.set_timeout(5);
                        toast.set_button_label(Some(&lang::lookup("header-open")));
                        toast.connect_button_clicked(move |_| {
                            let _ = open::that(path.clone());
                        });
                        widgets.toast_target.add_toast(toast);
                    }
                } else {
                    // Show error dialog
                    let error_dlg = ErrorDialogModel::builder()
                        .launch(ErrorDialogInit {
                            title: Box::new(lang::lookup("export-error-nothing-open-title")),
                            body: Box::new(lang::lookup("export-error-nothing-open-message")),
                        })
                        .forward(sender.input_sender(), |msg| match msg {});
                    error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                    self.latest_error_dlg = Some(error_dlg);
                }
            }
            AppInput::_ExportTestCase(format, path) => {
                if let Some(pkg) = &self.open_package {
                    let mut pkg = pkg.write().unwrap();

                    if let OpenCase::Case { id, .. } = &self.open_case {
                        if let Err(e) = match format.as_str() {
                            "html document" => {
                                HtmlExporter.export_case(&mut pkg, *id, path.clone())
                            }
                            "excel workbook" => {
                                ExcelExporter.export_case(&mut pkg, *id, path.clone())
                            }
                            "zip archive of files" => {
                                ZipOfFilesExporter.export_case(&mut pkg, *id, path.clone())
                            }
                            _ => {
                                tracing::error!("Invalid format specified.");
                                Ok(())
                            }
                        } {
                            // Show error dialog
                            let error_dlg = ErrorDialogModel::builder()
                                .launch(ErrorDialogInit {
                                    title: Box::new(lang::lookup("export-error-failed-title")),
                                    body: Box::new(lang::lookup_with_args(
                                        "export-error-failed-message",
                                        &lang_args!("error", e.to_string()),
                                    )),
                                })
                                .forward(sender.input_sender(), |msg| match msg {});
                            error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                            self.latest_error_dlg = Some(error_dlg);
                        } else {
                            let toast = adw::Toast::new(&lang::lookup("toast-export-complete"));
                            toast.set_timeout(5);
                            toast.set_button_label(Some(&lang::lookup("header-open")));
                            toast.connect_button_clicked(move |_| {
                                let _ = open::that(path.clone());
                            });
                            widgets.toast_target.add_toast(toast);
                        }
                    } else {
                        // Show error dialog
                        let error_dlg = ErrorDialogModel::builder()
                            .launch(ErrorDialogInit {
                                title: Box::new(lang::lookup("export-error-nothing-open-title")),
                                body: Box::new(lang::lookup("export-error-nothing-open-message")),
                            })
                            .forward(sender.input_sender(), |msg| match msg {});
                        error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                        self.latest_error_dlg = Some(error_dlg);
                    }
                } else {
                    // Show error dialog
                    let error_dlg = ErrorDialogModel::builder()
                        .launch(ErrorDialogInit {
                            title: Box::new(lang::lookup("export-error-nothing-open-title")),
                            body: Box::new(lang::lookup("export-error-nothing-open-message")),
                        })
                        .forward(sender.input_sender(), |msg| match msg {});
                    error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                    self.latest_error_dlg = Some(error_dlg);
                }
            }
            AppInput::PasteEvidence => {
                if let Some(disp) = gtk::gdk::Display::default() {
                    let clipboard = disp.clipboard();
                    let mime_types = clipboard.formats().mime_types();
                    tracing::debug!("Clipboard MIME types: {mime_types:?}");
                    let mut matched_kind = false;
                    'mime_loop: for mime in mime_types {
                        match mime.as_str().to_lowercase().as_str() {
                            "text/plain" | "text/plain;charset=utf-8" => {
                                // Paste as text
                                let sender_c = sender.clone();
                                clipboard.read_text_async(Some(&Cancellable::new()), move |cb| {
                                    if let Some(data) = cb.ok().flatten() {
                                        let evidence = Evidence::new(
                                            EvidenceKind::Text,
                                            evidenceangel::EvidenceData::Text {
                                                content: data.to_string(),
                                            },
                                        );
                                        sender_c.input(AppInput::_AddEvidence(evidence, None));
                                    } else {
                                        sender_c.input(AppInput::ShowToast(lang::lookup(
                                            "paste-evidence-failed",
                                        )));
                                    }
                                });
                                matched_kind = true;
                                break 'mime_loop;
                            }
                            "image/png" | "image/jpeg" | "image/bmp" => {
                                // Paste as image
                                let sender_c = sender.clone();
                                clipboard.read_texture_async(
                                    Some(&Cancellable::new()),
                                    move |cb| match cb {
                                        Ok(texture) => {
                                            if let Some(data) = texture {
                                                let media = MediaFile::from(
                                                    data.save_to_png_bytes().to_vec(),
                                                );
                                                let evidence = Evidence::new(
                                                    EvidenceKind::Image,
                                                    evidenceangel::EvidenceData::Media {
                                                        hash: media.hash(),
                                                    },
                                                );
                                                sender_c.input(AppInput::_AddMedia(media));
                                                sender_c
                                                    .input(AppInput::_AddEvidence(evidence, None));
                                            } else {
                                                sender_c.input(AppInput::ShowToast(lang::lookup(
                                                    "paste-evidence-failed",
                                                )));
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to paste image: {e}");
                                            sender_c.input(AppInput::ShowToast(lang::lookup(
                                                "paste-evidence-failed",
                                            )));
                                        }
                                    },
                                );
                                matched_kind = true;
                                break 'mime_loop;
                            }
                            _ => (),
                        }
                    }
                    if !matched_kind {
                        sender.input(AppInput::ShowToast(lang::lookup(
                            "paste-evidence-wrong-type",
                        )));
                    }
                } else {
                    tracing::warn!("No default display! Cannot get clipboard!");
                }
            }
            AppInput::ShowToast(msg) => {
                let toast = adw::Toast::new(&msg);
                toast.set_timeout(3);
                widgets.toast_target.add_toast(toast);
            }
        }
        self.update_view(widgets, sender);
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum OpenCase {
    #[default]
    Nothing,
    Metadata,
    Case {
        index: usize,
        id: Uuid,
    },
}
