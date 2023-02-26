fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    battleground_viewer::main()
}
