fn main() {
    if let Err(error) = rustone_lib::run_mcp_stdio() {
        eprintln!("[xingshi-mcp] {error}");
        std::process::exit(1);
    }
}
