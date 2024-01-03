extern crate embed_resource;
fn main() {
    slint_build::compile("ui/ui.slint").unwrap();
    #[cfg(windows)]
    embed_resource::compile("resources/windows/res.rc", embed_resource::NONE);
}