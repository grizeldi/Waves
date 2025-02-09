use gtk::glib;

mod imp {
    use epoxy::types::{GLuint};
    use epoxy::{BindFramebuffer, BindTexture, BlitFramebuffer, Clear, ClearColor, DeleteFramebuffers, DeleteTextures, FramebufferTexture2D, GenFramebuffers, GenTextures, GetIntegerv, TexStorage2DMultisample, COLOR_ATTACHMENT0, COLOR_BUFFER_BIT, DRAW_FRAMEBUFFER, DRAW_FRAMEBUFFER_BINDING, FRAMEBUFFER, NEAREST, RGBA8, TEXTURE_2D_MULTISAMPLE};
    use gtk::gdk::GLContext;
    use gtk::glib;
    use gtk::glib::Propagation;
    use gtk::prelude::WidgetExt;
    use gtk::subclass::prelude::*;
    use log::{debug, error, trace};
    use std::cell::Cell;

    #[derive(Default, Debug)]
    pub struct WaveformWidget {
        offscreen_framebuffer_handle: Cell<GLuint>,
        offscreen_texture_handle: Cell<GLuint>,
        original_framebuffer_handle: Cell<GLuint>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WaveformWidget {
        const NAME: &'static str = "WaveformWidget";
        type Type = super::WaveformWidget;
        type ParentType = gtk::GLArea;

        fn new() -> Self {
            Self {
                offscreen_framebuffer_handle: Cell::new(0),
                offscreen_texture_handle: Cell::new(0),
                original_framebuffer_handle: Cell::new(0),
            }
        }
    }

    impl ObjectImpl for WaveformWidget {}

    impl WidgetImpl for WaveformWidget {
        fn realize(&self) {
            self.parent_realize();
            debug!("Initializing OpenGL off screen frame buffer for WaveformWidget.");

            unsafe {
                // Create the off screen buffer
                GenFramebuffers(1, self.offscreen_framebuffer_handle.as_ptr());
                if self.offscreen_framebuffer_handle.get() == 0 {
                    error!("Failed to create off screen frame buffer!");
                    return;
                }

                // Get the handle of the main buffer
                let mut original_handle= -1;
                GetIntegerv(DRAW_FRAMEBUFFER_BINDING, &mut original_handle);
                self.original_framebuffer_handle.set(original_handle as GLuint);
            }
        }

        fn unrealize(&self) {
            self.parent_unrealize();
            trace!("WaveformWidget::unrealize");

            if self.offscreen_framebuffer_handle.get() != 0 {
                unsafe {DeleteFramebuffers(1, self.offscreen_framebuffer_handle.as_ptr());}
            }
            if self.offscreen_texture_handle.get() != 0 {
                unsafe {DeleteTextures(1, self.offscreen_texture_handle.as_ptr());}
            }
        }
    }

    impl GLAreaImpl for WaveformWidget {
        fn render(&self, _context: &GLContext) -> Propagation {
            trace!("WaveformWidget::render");
            unsafe {
                BindFramebuffer(FRAMEBUFFER, self.offscreen_framebuffer_handle.get());

                ClearColor(0.15, 0.155, 0.17, 1.0);
                Clear(COLOR_BUFFER_BIT);

                BindFramebuffer(DRAW_FRAMEBUFFER, self.original_framebuffer_handle.get());
                BlitFramebuffer(0, 0, self.obj().width(), self.obj().height(),
                                0, 0, self.obj().width(), self.obj().height(),
                                COLOR_BUFFER_BIT, NEAREST);
                BindFramebuffer(FRAMEBUFFER, self.original_framebuffer_handle.get());
            }
            Propagation::Stop
        }

        fn resize(&self, width: i32, height: i32) {
            debug!("Resizing WaveformWidget to {}x{}px.", width, height);

            if self.offscreen_framebuffer_handle.get() == 0 {
                error!("Cannot resize waveform texture as the framebuffer doesn't exist.");
                return;
            }

            unsafe {
                if self.offscreen_texture_handle.get() != 0 {
                    DeleteTextures(1, self.offscreen_texture_handle.as_ptr());
                }

                GenTextures(1, self.offscreen_texture_handle.as_ptr());
                if self.offscreen_texture_handle.get() == 0 {
                    error!("Failed to create off screen texture!");
                    return;
                }

                BindTexture(TEXTURE_2D_MULTISAMPLE, self.offscreen_texture_handle.get());
                TexStorage2DMultisample(TEXTURE_2D_MULTISAMPLE, 8, RGBA8, width, height, 1);
                BindFramebuffer(FRAMEBUFFER, self.offscreen_framebuffer_handle.get());
                FramebufferTexture2D(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D_MULTISAMPLE, self.offscreen_texture_handle.get(), 0);
                BindFramebuffer(FRAMEBUFFER, self.original_framebuffer_handle.get());
            }
        }
    }
}

glib::wrapper! {
    pub struct WaveformWidget(ObjectSubclass<imp::WaveformWidget>)
    @extends gtk::GLArea, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
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