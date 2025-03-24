use gtk::glib::wrapper;

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
        pub waveform_widget: TemplateChild<WaveformWidget>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WavesWindow {
        const NAME: &'static str = "WavesWindow";
        type Type = super::WavesWindow;
        type ParentType = gtk::Window;

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
    impl WidgetImpl for WavesWindow {
        fn realize(&self) {

        }
    }
    impl WindowImpl for WavesWindow {

    }
}

wrapper! {
    pub struct WavesWindow(ObjectSubclass<imp::WavesWindow>)
    @extends gtk::Window, gtk::Widget,
    @implements gtk::Buildable;
}

impl WavesWindow {

}