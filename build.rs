fn main() {
    glib_build_tools::compile_resources(&["res"], "res/waves.gresource.xml", "waves.gresources");
}