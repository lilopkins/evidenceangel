use std::{collections::HashMap, path::PathBuf};

use adw::prelude::*;
use evidenceangel::{Author, EvidencePackage};
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
    dialogs::{
        error::{ErrorDialogInit, ErrorDialogInput, ErrorDialogModel},
        new_author::{NewAuthorDialogModel, NewAuthorInput, NewAuthorOutput},
    },
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

pub struct AppModel {
    open_package: Option<EvidencePackage>,
    open_path: Option<PathBuf>,
    open_case: OpenCase,

    latest_new_author_dlg: Option<Controller<NewAuthorDialogModel>>,
    latest_error_dlg: Option<Controller<ErrorDialogModel>>,

    test_case_nav_factory: FactoryVecDeque<NavFactoryModel>,
    authors_factory: FactoryVecDeque<AuthorFactoryModel>,
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
        self.open_package = Some(pkg);
        self.update_nav_menu()?;
        Ok(())
    }

    fn open(&mut self, path: PathBuf) -> evidenceangel::Result<()> {
        let pkg = EvidencePackage::open(path.clone())?;
        log::debug!("Package opened: {pkg:?}");
        self.open_package = Some(pkg);
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
            for case in pkg.test_case_iter()? {
                test_case_data.push_back(NavFactoryInit {
                    id: *case.id(),
                    name: case.metadata().title().clone(),
                });
            }
        }
        Ok(())
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
    NavigateTo(OpenCase),
    DeleteCase(Uuid),
    CreateCaseAndSelect,
    SetMetadataTitle(String),
    CreateAuthor,
    _CreateAuthor(Author),
    DeleteAuthor(Author),

    SetTestCaseTitle(String),
}

#[relm4::component(pub)]
impl Component for AppModel {
    type CommandOutput = ();
    type Input = AppInput;
    type Output = ();
    type Init = ();

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
                            pack_start = &gtk::MenuButton {
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

                                gtk::Button {
                                    set_label: &lang::lookup("nav-create-case"),
                                    add_css_class: "flat",
                                    set_icon_name: relm4_icons::icon_names::PLUS,
                                    connect_clicked => AppInput::CreateCaseAndSelect,
                                },
                            }
                        }
                    },
                },

                #[wrap(Some)]
                set_content = &adw::NavigationPage {
                    #[watch]
                    set_title: &format!("{} · {}", if let Some(pkg) = model.open_package.as_ref() {
                        pkg.metadata().title().clone()
                    } else {
                        lang::lookup("title-no-package")
                    }, match model.open_case {
                        OpenCase::Nothing => lang::lookup("title-no-case"),
                        OpenCase::Metadata => lang::lookup("nav-metadata"),
                        OpenCase::Case { id, .. } => {
                            if let Some(pkg) = model.open_package.as_ref() {
                                if let Some(case) = pkg.test_case(id).ok().flatten() {
                                    case.metadata().title().clone()
                                } else {
                                    // This is very briefly hit as a case is deleted
                                    lang::lookup("title-no-case")
                                }
                            } else {
                                unreachable!()
                            }
                        },
                    }),

                    adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar,

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
                                    adw::PreferencesGroup {
                                        set_title: &lang::lookup("test-group-title"),

                                        #[name = "test_title"]
                                        adw::EntryRow {
                                            set_title: &lang::lookup("test-title"),
                                            connect_changed[sender] => move |entry| {
                                                sender.input(AppInput::SetTestCaseTitle(entry.text().to_string()));
                                            }
                                        },

                                        #[name = "test_execution"]
                                        adw::ActionRow {
                                            set_title: &lang::lookup("test-execution"),
                                        }
                                    },

                                    // TODO Test Case Screen
                                },
                            }
                        },
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
                &lang::lookup("header-about") => AboutAction,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            open_package: None,
            open_path: None,
            open_case: OpenCase::Nothing,

            latest_error_dlg: None,
            latest_new_author_dlg: None,

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
        };

        let test_case_list = model.test_case_nav_factory.widget();
        let authors_list = model.authors_factory.widget();
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
        group.register_for_widget(root);

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
                if let Some(package) = self.open_package.as_mut() {
                    if let Err(e) = package.save() {
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
                self.close();
                sender.input(*then);
            }
            AppInput::CloseFile => {
                // TODO Propose to save if needed
                self.open_case = OpenCase::Nothing;
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
                                .map(|pkg| pkg.metadata().title().clone())
                                .expect("Cannot navigate to metadata when no package is open"),
                        );
                        let mut authors = self.authors_factory.guard();
                        authors.clear();
                        let pkg_authors = self
                            .open_package
                            .as_ref()
                            .map(|pkg| pkg.metadata().authors().clone())
                            .expect("Cannot navigate to metadata when no package is open");
                        for author in pkg_authors {
                            authors.push_back(author);
                        }
                        widgets.nav_metadata.set_has_frame(true);
                    }
                    OpenCase::Case { index, id } => {
                        self.test_case_nav_factory
                            .send(index, NavFactoryInput::ShowAsSelected(true));

                        if let Some(pkg) = self.open_package.as_ref() {
                            if let Some(tc) = pkg.test_case(id).ok().flatten() {
                                // Update test case metadata on screen
                                widgets.test_title.set_text(tc.metadata().title());
                                widgets
                                    .test_execution
                                    .set_subtitle(&tc.metadata().execution_datetime().to_rfc3339());
                            }
                        }
                    }
                    OpenCase::Nothing => (),
                }
            }
            AppInput::CreateCaseAndSelect => {
                if self.open_package.is_none() {
                    return;
                }

                let mut index = usize::MAX;
                let mut case_id = Uuid::default();
                if let Some(pkg) = self.open_package.as_mut() {
                    let case = pkg
                        .create_test_case(lang::lookup("default-case-title"))
                        .unwrap(); // doesn't fail
                    case_id = *case.id();
                }

                // Establish index
                for (idx, c) in self
                    .open_package
                    .as_ref()
                    .unwrap()
                    .test_case_iter()
                    .unwrap()
                    .enumerate()
                {
                    if c.id() == &case_id {
                        index = idx;
                    }
                }
                assert_ne!(index, usize::MAX);

                // Add case to navigation
                self.update_nav_menu().unwrap(); // doesn't fail

                // Switch to case
                // First unselect all cases
                widgets.nav_metadata.set_has_frame(false);
                self.test_case_nav_factory
                    .broadcast(NavFactoryInput::ShowAsSelected(false));
                // Then select the new case
                self.test_case_nav_factory
                    .send(index, NavFactoryInput::ShowAsSelected(true));
                self.open_case = OpenCase::Case { index, id: case_id };
            }
            AppInput::SetMetadataTitle(new_title) => {
                if let Some(pkg) = self.open_package.as_mut() {
                    pkg.metadata_mut().set_title(new_title);
                }
            }
            AppInput::DeleteCase(id) => {
                if let Some(pkg) = self.open_package.as_mut() {
                    if let Err(e) = pkg.delete_test_case(id) {
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
                if let Some(pkg) = self.open_package.as_mut() {
                    pkg.metadata_mut().authors_mut().push(author);
                    sender.input(AppInput::NavigateTo(OpenCase::Metadata)); // to refresh author list
                }
            }
            AppInput::DeleteAuthor(author) => {
                if let Some(pkg) = self.open_package.as_mut() {
                    let idx = pkg
                        .metadata()
                        .authors()
                        .iter()
                        .position(|a| *a == author)
                        .unwrap();
                    pkg.metadata_mut().authors_mut().remove(idx);
                    sender.input(AppInput::NavigateTo(OpenCase::Metadata)); // to refresh author list
                }
            }
            AppInput::SetTestCaseTitle(new_title) => {
                if !new_title.trim().is_empty() {
                    if let OpenCase::Case { index, id, .. } = &self.open_case {
                        if let Some(pkg) = self.open_package.as_mut() {
                            if let Some(tc) = pkg.test_case_mut(*id).ok().flatten() {
                                tc.metadata_mut().set_title(new_title.clone());
                                self.test_case_nav_factory
                                    .send(*index, NavFactoryInput::UpdateTitle(new_title));
                            }
                        }
                    }
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
