use gtk::Application;
use gtk::glib::{wrapper, Object};
use gtk::subclass::prelude::ObjectSubclassIsExt;
use lofty::prelude::*;
use log::info;

mod imp {
    use gtk::{glib, CompositeTemplate, TemplateChild};
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::subclass::window::WindowImpl;
    use crate::waveformwidget::WaveformWidget;

    #[derive(CompositeTemplate, Default, Debug)]
    #[template(resource = "/com/github/grizeldi/waves/waveswindow.ui")]
    pub struct WavesWindow {
        #[template_child]
        pub waveform_widget: TemplateChild<WaveformWidget>,
        #[template_child]
        pub album_cover_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub author_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WavesWindow {
        const NAME: &'static str = "WavesWindow";
        type Type = super::WavesWindow;
        type ParentType = gtk::ApplicationWindow;

        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WavesWindow {}
    impl WidgetImpl for WavesWindow {}
    impl WindowImpl for WavesWindow {}
    impl ApplicationWindowImpl for WavesWindow {}
}

wrapper! {
    pub struct WavesWindow(ObjectSubclass<imp::WavesWindow>)
    @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
                gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl WavesWindow {
    pub fn new(application : &Application) -> Self {
        Object::builder()
            .property("application", &application)
            .build()
    }

    pub fn open_file(&self, file_path : &str) {
        info!("Opening file {:?}.", file_path);
        let tagged_file = lofty::read_from_path(file_path).expect("Failed to read file tags.");
        if let Some(tag) = tagged_file.primary_tag() {
            if let Some(title) = tag.title() {
                self.imp().title_label.set_label(&title.to_string());
            }
            if let Some(author) = tag.artist() {
                self.imp().author_label.set_label(&author.to_string());
            }
        }

        self.imp().waveform_widget.set_audio_file(file_path);
    }
}