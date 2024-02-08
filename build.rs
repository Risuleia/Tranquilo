fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");

    slint_build::compile("ui/appwindow.slint").unwrap();
}