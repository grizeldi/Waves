use gtk::glib;

mod imp {
    use epoxy::types::GLint;
    use gtk::gdk::GLContext;
    use gtk::subclass::prelude::*;
    use gtk::glib;
    use gtk::glib::Propagation;
    use log::{debug, trace};

    #[derive(Default, Debug)]
    pub struct WaveformWidget {
        framebuffer_handle: GLint,
        offscreen_texture: GLint,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WaveformWidget {
        const NAME: &'static str = "WaveformWidget";
        type Type = super::WaveformWidget;
        type ParentType = gtk::GLArea;

        fn new() -> Self {
            Self {
                offscreen_texture: -1,
                framebuffer_handle: -1
            }
        }
    }

    impl ObjectImpl for WaveformWidget {}

    impl WidgetImpl for WaveformWidget {
        fn realize(&self) {
            self.parent_realize();
            debug!("Initializing OpenGL for {:?}", self);

            //TODO set up OpenGL stuff
        }

        fn unrealize(&self) {
            self.parent_unrealize();
            trace!("WaveformWidget::unrealize");

            //TODO clean up OpenGL stuff
        }
    }

    impl GLAreaImpl for WaveformWidget {
        fn render(&self, context: &GLContext) -> Propagation {
            Propagation::Stop
        }

        fn resize(&self, width: i32, height: i32) {
            debug!("Resizing WaveformWidget to {}x{}px.", width, height);
            //TODO figure out how to resize offscreen framebuffer
        }
    }
}

glib::wrapper! {
    pub struct WaveformWidget(ObjectSubclass<imp::WaveformWidget>)
    @extends gtk::GLArea, gtk::Widget;
}

impl WaveformWidget {
    pub fn new() -> Self {
        let waveform_widget = glib::Object::builder::<WaveformWidget>()
            .build();
        waveform_widget
    }

    pub fn set_audio_file() {
        todo!()
    }
}