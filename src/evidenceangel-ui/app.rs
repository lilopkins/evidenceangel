use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use adw::prelude::*;
use evidenceangel::{
    exporters::{excel::ExcelExporter, html::HtmlExporter, Exporter},
    Author, Evidence, EvidencePackage,
};
#[allow(unused)]
use gtk::prelude::*;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw,
    factory::FactoryVecDeque,
    gtk,
    prelude::*,
    Component, ComponentParts, ComponentSender,
};
use uuid::Uuid;

use crate::{
    author_factory::{AuthorFactoryModel, AuthorFactoryOutput},
    dialogs::{add_evidence::*, error::*, export::*, new_author::*},
    evidence_factory::{EvidenceFactoryInit, EvidenceFactoryModel},
    filter, lang,
    nav_factory::{NavFactoryInit, NavFactoryInput, NavFactoryModel, NavFactoryOutput},
};

relm4::new_action_group!(MenuActionGroup, "menu");
relm4::new_stateless_action!(NewAction, MenuActionGroup, "new");
relm4::new_stateless_action!(OpenAction, MenuActionGroup, "open");
relm4::new_stateless_action!(SaveAction, MenuActionGroup, "save");
relm4::new_stateless_action!(SaveAsAction, MenuActionGroup, "save-as");
relm4::new_stateless_action!(CloseAction, MenuActionGroup, "close");
relm4::new_stateless_action!(AboutAction, MenuActionGroup, "about");
relm4::new_stateless_action!(ExportPackageAction, MenuActionGroup, "export-package");
relm4::new_stateless_action!(ExportTestCaseAction, MenuActionGroup, "export-test-case");

pub struct AppModel {
    open_package: Option<Arc<RwLock<EvidencePackage>>>,
    open_path: Option<PathBuf>,
    open_case: OpenCase,

    latest_new_author_dlg: Option<Controller<NewAuthorDialogModel>>,
    latest_add_evidence_text_dlg: Option<Controller<AddTextEvidenceDialogModel>>,
    latest_add_evidence_http_dlg: Option<Controller<AddHttpEvidenceDialogModel>>,
    latest_add_evidence_image_dlg: Option<Controller<AddImageEvidenceDialogModel>>,
    latest_error_dlg: Option<Controller<ErrorDialogModel>>,
    latest_export_dlg: Option<Controller<ExportDialogModel>>,

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
                .to_path_buf(),
            title,
            authors,
        )?;
        log::debug!("Package opened: {pkg:?}");
        self.open_package = Some(Arc::new(RwLock::new(pkg)));
        self.update_nav_menu()?;
        Ok(())
    }

    fn open(&mut self, path: PathBuf) -> evidenceangel::Result<()> {
        let pkg = EvidencePackage::open(path.clone())?;
        log::debug!("Package opened: {pkg:?}");
        self.open_package = Some(Arc::new(RwLock::new(pkg)));
        self.open_path = Some(path);
        self.update_nav_menu()?;
        Ok(())
    }

    fn close(&mut self) {
        self.open_package = None;
        self.open_path = None;
        log::debug!("Package closed.");
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

    fn get_package(&self) -> Option<Arc<RwLock<EvidencePackage>>> {
        self.open_package.as_ref().map(|pkg| pkg.clone())
    }
}

#[derive(Debug)]
pub enum AppInput {
    // Menu options
    NewFile,
    _NewFile,
    OpenFile,
    _OpenFile(PathBuf),
    SaveFile,
    SaveAsFile,
    CloseFile,
    OpenAboutDialog,

    CloseFileIfOpenThen(Box<AppInput>),
    _SetPathThen(PathBuf, Box<AppInput>),

    #[allow(private_interfaces)]
    /// NavigateTo ignores the index provided as part of OpenCase::Case and establishes
    /// it automatically.
    NavigateTo(OpenCase),
    DeleteCase(Uuid),
    CreateCaseAndSelect,
    SetMetadataTitle(String),
    CreateAuthor,
    _CreateAuthor(Author),
    DeleteAuthor(Author),

    SetTestCaseTitle(String),
    AddTextEvidence,
    AddHttpEvidence,
    AddImageEvidence,
    #[allow(dead_code)]
    AddFileEvidence,
    _AddEvidence(Evidence),
    /// Show an error dialog.
    ShowError {
        title: String,
        message: String,
    },

    ExportPackage,
    _ExportPackage(String, PathBuf),
    ExportTestCase,
    _ExportTestCase(String, PathBuf),
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
            set_width_request: 800,
            set_height_request: 600,

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
                                set_visible: model.open_package.is_some(),
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
                                set_title: &if let Some(pkg) = model.open_package.as_ref() {
                                    pkg.read().unwrap().metadata().title().clone()
                                } else {
                                    lang::lookup("title-no-package")
                                },
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
                                                }
                                            }
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
                                                    add_css_class: "flat",

                                                    connect_clicked[sender] => move |_entry| {
                                                        sender.input(AppInput::CreateAuthor);
                                                    }
                                                }
                                            },
                                        }
                                    },
                                    OpenCase::Case { .. } => gtk::Box {
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
                                                        }
                                                    },

                                                    #[name = "test_execution"]
                                                    adw::ActionRow {
                                                        set_title: &lang::lookup("test-execution"),
                                                    }
                                                },

                                                // Test Case Screen
                                                #[local_ref]
                                                evidence_list -> gtk::Box {
                                                    set_orientation: gtk::Orientation::Vertical,
                                                    set_spacing: 8,
                                                    set_margin_top: 8,
                                                },

                                                gtk::Box {
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_margin_top: 8,
                                                    set_spacing: 4,
                                                    set_halign: gtk::Align::Center,

                                                    gtk::Button {
                                                        connect_clicked => AppInput::AddTextEvidence,

                                                        gtk::Box {
                                                            set_orientation: gtk::Orientation::Horizontal,

                                                            gtk::Image::from_icon_name(relm4_icons::icon_names::PLUS),
                                                            gtk::Label {
                                                                set_label: "Text",
                                                            },
                                                        }
                                                    },
                                                    gtk::Button {
                                                        connect_clicked => AppInput::AddHttpEvidence,

                                                        gtk::Box {
                                                            set_orientation: gtk::Orientation::Horizontal,

                                                            gtk::Image::from_icon_name(relm4_icons::icon_names::PLUS),
                                                            gtk::Label {
                                                                set_label: "HTTP Request",
                                                            },
                                                        }
                                                    },
                                                    gtk::Button {
                                                        connect_clicked => AppInput::AddImageEvidence,

                                                        gtk::Box {
                                                            set_orientation: gtk::Orientation::Horizontal,

                                                            gtk::Image::from_icon_name(relm4_icons::icon_names::PLUS),
                                                            gtk::Label {
                                                                set_label: "Image",
                                                            },
                                                        }
                                                    },
                                                    /* gtk::Button {
                                                        connect_clicked => AppInput::AddFileEvidence,

                                                        gtk::Box {
                                                            set_orientation: gtk::Orientation::Horizontal,

                                                            gtk::Image::from_icon_name(relm4_icons::icon_names::PLUS),
                                                            gtk::Label {
                                                                set_label: "File",
                                                            },
                                                        }
                                                    }, */
                                                },
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
            &lang::lookup("header-save-as") => SaveAsAction,
            &lang::lookup("header-close") => CloseAction,
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
        let model = AppModel {
            open_package: None,
            open_path: None,
            open_case: OpenCase::Nothing,

            latest_error_dlg: None,
            latest_new_author_dlg: None,
            latest_add_evidence_text_dlg: None,
            latest_add_evidence_http_dlg: None,
            latest_add_evidence_image_dlg: None,
            latest_export_dlg: None,

            test_case_nav_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |output| match output {
                    NavFactoryOutput::NavigateTo(index, id) => {
                        AppInput::NavigateTo(OpenCase::Case { index, id })
                    }
                    NavFactoryOutput::DeleteCase(_index, id) => AppInput::DeleteCase(id),
                },
            ),
            authors_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |output| match output {
                    AuthorFactoryOutput::DeleteAuthor(author) => AppInput::DeleteAuthor(author),
                },
            ),
            test_evidence_factory: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {}),
        };

        let test_case_list = model.test_case_nav_factory.widget();
        let authors_list = model.authors_factory.widget();
        let evidence_list = model.test_evidence_factory.widget();
        let widgets = view_output!();

        let sender_c = sender.clone();
        let new_action: RelmAction<NewAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::CloseFileIfOpenThen(Box::new(AppInput::NewFile)));
        });
        relm4::main_application().set_accelerators_for_action::<NewAction>(&["<primary>N"]);

        let sender_c = sender.clone();
        let open_action: RelmAction<OpenAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::CloseFileIfOpenThen(Box::new(AppInput::OpenFile)));
        });
        relm4::main_application().set_accelerators_for_action::<OpenAction>(&["<primary>O"]);

        let sender_c = sender.clone();
        let save_action: RelmAction<SaveAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::SaveFile);
        });
        relm4::main_application().set_accelerators_for_action::<SaveAction>(&["<primary>S"]);

        let sender_c = sender.clone();
        let save_as_action: RelmAction<SaveAsAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::SaveAsFile);
        });
        relm4::main_application()
            .set_accelerators_for_action::<SaveAsAction>(&["<primary><shift>S"]);

        let sender_c = sender.clone();
        let close_action: RelmAction<CloseAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::CloseFile);
        });
        relm4::main_application().set_accelerators_for_action::<CloseAction>(&["<primary>W"]);

        let sender_c = sender.clone();
        let export_package_action: RelmAction<ExportPackageAction> =
            RelmAction::new_stateless(move |_| {
                sender_c.input(AppInput::ExportPackage);
            });
        relm4::main_application()
            .set_accelerators_for_action::<ExportPackageAction>(&["<primary>E"]);

        let sender_c = sender.clone();
        let export_test_case_action: RelmAction<ExportTestCaseAction> =
            RelmAction::new_stateless(move |_| {
                sender_c.input(AppInput::ExportTestCase);
            });
        relm4::main_application()
            .set_accelerators_for_action::<ExportTestCaseAction>(&["<primary><shift>E"]);

        let sender_c = sender.clone();
        let about_action: RelmAction<AboutAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(AppInput::OpenAboutDialog);
        });
        relm4::main_application().set_accelerators_for_action::<AboutAction>(&["F1"]);

        let mut group = RelmActionGroup::<MenuActionGroup>::new();
        group.add_action(new_action);
        group.add_action(open_action);
        group.add_action(save_action);
        group.add_action(save_as_action);
        group.add_action(close_action);
        group.add_action(about_action);
        group.add_action(export_package_action);
        group.add_action(export_test_case_action);
        group.register_for_widget(&root);

        if let Some(file) = init {
            sender.input(AppInput::_OpenFile(file));
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
        log::debug!("Handling event: {message:?}");
        match message {
            AppInput::NewFile => {
                // TODO Propose to save if needed
                // Show file selection dialog
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("header-save"))
                    .filters(&filter::filter_list(vec![filter::packages()]))
                    .initial_folder(&gtk::gio::File::for_path("."))
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
                                Box::new(AppInput::_NewFile),
                            ));
                        }
                    },
                );
            }
            AppInput::_NewFile => {
                // Set default name, author, execution date and path.
                sender.input(AppInput::NavigateTo(OpenCase::Nothing));
                if let Err(e) = self.open_new() {
                    let error_dlg = ErrorDialogModel::builder()
                        .launch(ErrorDialogInit {
                            title: Box::new(lang::lookup("error-failed-new-title")),
                            body: Box::new(lang::lookup_with_args("error-failed-new-body", {
                                let mut map = HashMap::new();
                                map.insert("error", e.to_string().into());
                                map
                            })),
                        })
                        .forward(sender.input_sender(), |msg| match msg {});
                    error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                    self.latest_error_dlg = Some(error_dlg);
                }
            }
            AppInput::OpenFile => {
                // TODO Propose to save if needed
                // Show file selection dialog
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("header-open"))
                    .filters(&filter::filter_list(vec![filter::packages()]))
                    .initial_folder(&gtk::gio::File::for_path("."))
                    .build();

                let sender_c = sender.clone();
                dialog.open(
                    Some(&root.toplevel_window().unwrap()),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let path = file.path().unwrap();
                            // Open this package
                            sender_c.input(AppInput::_OpenFile(path));
                        }
                    },
                );
            }
            AppInput::_OpenFile(path) => {
                if let Err(e) = self.open(path) {
                    let error_dlg = ErrorDialogModel::builder()
                        .launch(ErrorDialogInit {
                            title: Box::new(lang::lookup("error-failed-open-title")),
                            body: Box::new(lang::lookup_with_args("error-failed-open-body", {
                                let mut map = HashMap::new();
                                map.insert("error", e.to_string().into());
                                map
                            })),
                        })
                        .forward(sender.input_sender(), |msg| match msg {});
                    error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                    self.latest_error_dlg = Some(error_dlg);
                }
            }
            AppInput::SaveFile => {
                if let Some(package) = self.get_package() {
                    if let Err(e) = package.write().unwrap().save() {
                        // Show error dialog
                        let error_dlg = ErrorDialogModel::builder()
                            .launch(ErrorDialogInit {
                                title: Box::new(lang::lookup("error-failed-save-title")),
                                body: Box::new(lang::lookup_with_args("error-failed-save-body", {
                                    let mut map = HashMap::new();
                                    map.insert("error", e.to_string().into());
                                    map
                                })),
                            })
                            .forward(sender.input_sender(), |msg| match msg {});
                        error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                        self.latest_error_dlg = Some(error_dlg);
                    } else {
                        let toast = adw::Toast::new(&lang::lookup("toast-saved"));
                        toast.set_timeout(3);
                        widgets.toast_target.add_toast(toast);
                    }
                }
            }
            AppInput::SaveAsFile => {
                // TODO Propose a new file location
                // Then save
                sender.input(AppInput::SaveFile);
            }
            AppInput::CloseFileIfOpenThen(then) => {
                // TODO Propose to save if needed
                sender.input(AppInput::NavigateTo(OpenCase::Nothing));
                self.close();
                sender.input(*then);
            }
            AppInput::CloseFile => {
                // TODO Propose to save if needed
                sender.input(AppInput::NavigateTo(OpenCase::Nothing));
                self.close();
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

                // Then select the new case
                match target {
                    OpenCase::Metadata => {
                        // Update fields
                        widgets.metadata_title.set_text(
                            &self
                                .open_package
                                .as_ref()
                                .map(|pkg| pkg.read().unwrap().metadata().title().clone())
                                .expect("Cannot navigate to metadata when no package is open"),
                        );
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

                        self.test_case_nav_factory
                            .send(index, NavFactoryInput::ShowAsSelected(true));

                        let mut new_evidence = vec![];
                        if let Some(pkg) = self.get_package() {
                            if let Some(tc) = pkg.read().unwrap().test_case(id).ok().flatten() {
                                // Update test case metadata on screen
                                widgets.test_title.set_text(tc.metadata().title());
                                widgets.test_execution.set_subtitle(&format!(
                                    "{}",
                                    tc.metadata()
                                        .execution_datetime()
                                        .format(&lang::lookup("date-time"))
                                ));

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

                // Switch to case
                sender.input(AppInput::NavigateTo(OpenCase::Case {
                    // index will be calculated by NavigateTo
                    index: 0,
                    id: case_id,
                }));
            }
            AppInput::SetMetadataTitle(new_title) => {
                if !new_title.trim().is_empty() {
                    if new_title.len() <= 30 {
                        if let Some(pkg) = self.get_package() {
                            pkg.write().unwrap().metadata_mut().set_title(new_title);
                        }
                    } else {
                        let toast = adw::Toast::new(&lang::lookup("toast-name-too-long"));
                        toast.set_timeout(1);
                        widgets.toast_target.add_toast(toast);
                    }
                } else {
                    let toast = adw::Toast::new(&lang::lookup("toast-name-cant-be-empty"));
                    toast.set_timeout(1);
                    widgets.toast_target.add_toast(toast);
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
                                    {
                                        let mut map = HashMap::new();
                                        map.insert("error", e.to_string().into());
                                        map
                                    },
                                )),
                            })
                            .forward(sender.input_sender(), |msg| match msg {});
                        error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                        self.latest_error_dlg = Some(error_dlg);
                    }
                    self.update_nav_menu().unwrap(); // doesn't fail
                    sender.input(AppInput::NavigateTo(OpenCase::Metadata));
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
                        .push(author);
                    sender.input(AppInput::NavigateTo(OpenCase::Metadata)); // to refresh author list
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
                    sender.input(AppInput::NavigateTo(OpenCase::Metadata)); // to refresh author list
                }
            }
            AppInput::SetTestCaseTitle(new_title) => {
                if !new_title.trim().is_empty() {
                    if new_title.len() <= 30 {
                        if let OpenCase::Case { index, id, .. } = &self.open_case {
                            if let Some(pkg) = self.get_package() {
                                if let Some(tc) = pkg.write().unwrap().test_case_mut(*id).ok().flatten()
                                {
                                    tc.metadata_mut().set_title(new_title.clone());
                                    self.test_case_nav_factory
                                        .send(*index, NavFactoryInput::UpdateTitle(new_title));
                                }
                            }
                        }
                    } else {
                        let toast = adw::Toast::new(&lang::lookup("toast-name-too-long"));
                        toast.set_timeout(1);
                        widgets.toast_target.add_toast(toast);
                    }
                } else {
                    let toast = adw::Toast::new(&lang::lookup("toast-name-cant-be-empty"));
                    toast.set_timeout(1);
                    widgets.toast_target.add_toast(toast);
                }
            }
            AppInput::AddTextEvidence => {
                let add_evidence_text_dlg = AddTextEvidenceDialogModel::builder()
                    .launch(self.get_package().unwrap())
                    .forward(sender.input_sender(), |msg| match msg {
                        AddEvidenceOutput::AddEvidence(ev) => AppInput::_AddEvidence(ev),
                        AddEvidenceOutput::Error { title, message } => {
                            AppInput::ShowError { title, message }
                        }
                    });
                add_evidence_text_dlg.emit(AddEvidenceInput::Present(root.clone()));
                self.latest_add_evidence_text_dlg = Some(add_evidence_text_dlg);
            }
            AppInput::AddHttpEvidence => {
                let add_evidence_http_dlg = AddHttpEvidenceDialogModel::builder()
                    .launch(self.get_package().unwrap())
                    .forward(sender.input_sender(), |msg| match msg {
                        AddEvidenceOutput::AddEvidence(ev) => AppInput::_AddEvidence(ev),
                        AddEvidenceOutput::Error { title, message } => {
                            AppInput::ShowError { title, message }
                        }
                    });
                add_evidence_http_dlg.emit(AddEvidenceInput::Present(root.clone()));
                self.latest_add_evidence_http_dlg = Some(add_evidence_http_dlg);
            }
            AppInput::AddImageEvidence => {
                let add_evidence_image_dlg = AddImageEvidenceDialogModel::builder()
                    .launch(self.get_package().unwrap())
                    .forward(sender.input_sender(), |msg| match msg {
                        AddEvidenceOutput::AddEvidence(ev) => AppInput::_AddEvidence(ev),
                        AddEvidenceOutput::Error { title, message } => {
                            AppInput::ShowError { title, message }
                        }
                    });
                add_evidence_image_dlg.emit(AddEvidenceInput::Present(root.clone()));
                self.latest_add_evidence_image_dlg = Some(add_evidence_image_dlg);
            }
            AppInput::AddFileEvidence => (),
            AppInput::_AddEvidence(ev) => {
                if let Some(pkg) = self.get_package() {
                    if let OpenCase::Case { id, .. } = &self.open_case {
                        pkg.write()
                            .unwrap()
                            .test_case_mut(*id)
                            .ok()
                            .flatten()
                            .unwrap()
                            .evidence_mut()
                            .push(ev);
                        // to refresh evidence
                        sender.input(AppInput::NavigateTo(self.open_case));
                    }
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
                        test_case_name: None,
                    })
                    .forward(sender.input_sender(), |msg| match msg {
                        ExportOutput::Export { format, path } => {
                            AppInput::_ExportPackage(format, path)
                        }
                        ExportOutput::Error { title, message } => {
                            AppInput::ShowError { title, message }
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
                            test_case_name: Some(case_name),
                        })
                        .forward(sender.input_sender(), |msg| match msg {
                            ExportOutput::Export { format, path } => {
                                AppInput::_ExportTestCase(format, path)
                            }
                            ExportOutput::Error { title, message } => {
                                AppInput::ShowError { title, message }
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
                        "html" => HtmlExporter.export_package(&mut pkg, path),
                        "excel" => ExcelExporter.export_package(&mut pkg, path),
                        _ => {
                            log::error!("Invalid format specified.");
                            Ok(())
                        }
                    } {
                        // Show error dialog
                        let error_dlg = ErrorDialogModel::builder()
                            .launch(ErrorDialogInit {
                                title: Box::new(lang::lookup("export-error-failed-title")),
                                body: Box::new(lang::lookup_with_args(
                                    "export-error-failed-message",
                                    {
                                        let mut map = HashMap::new();
                                        map.insert("error", e.to_string().into());
                                        map
                                    },
                                )),
                            })
                            .forward(sender.input_sender(), |msg| match msg {});
                        error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                        self.latest_error_dlg = Some(error_dlg);
                    } else {
                        let toast = adw::Toast::new(&lang::lookup("toast-export-complete"));
                        toast.set_timeout(1);
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
                            "html" => HtmlExporter.export_case(&mut pkg, *id, path),
                            "excel" => ExcelExporter.export_case(&mut pkg, *id, path),
                            _ => {
                                log::error!("Invalid format specified.");
                                Ok(())
                            }
                        } {
                            // Show error dialog
                            let error_dlg = ErrorDialogModel::builder()
                                .launch(ErrorDialogInit {
                                    title: Box::new(lang::lookup("export-error-failed-title")),
                                    body: Box::new(lang::lookup_with_args(
                                        "export-error-failed-message",
                                        {
                                            let mut map = HashMap::new();
                                            map.insert("error", e.to_string().into());
                                            map
                                        },
                                    )),
                                })
                                .forward(sender.input_sender(), |msg| match msg {});
                            error_dlg.emit(ErrorDialogInput::Present(root.clone()));
                            self.latest_error_dlg = Some(error_dlg);
                        } else {
                            let toast = adw::Toast::new(&lang::lookup("toast-export-complete"));
                            toast.set_timeout(1);
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
        }
        self.update_view(widgets, sender)
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
