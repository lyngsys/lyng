fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if let Err(message) = lyng_js_bench::run(&args) {
        if message.starts_with("Usage:") {
            println!("{message}");
            return;
        }
        eprintln!("{message}");
        std::process::exit(2);
    }
}
