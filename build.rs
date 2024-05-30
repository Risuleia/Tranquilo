fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");

    #[cfg(windows)]
    windres::Build::new().compile("tranquilo.rc").unwrap();

    slint_build::compile("ui/appwindow.slint").unwrap();
}