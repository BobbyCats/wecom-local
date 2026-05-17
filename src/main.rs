mod cli;
mod conversations;
mod decoder;
mod doctor;
mod local_query;
mod members;
mod output;
mod runtime_bridge;
mod stats;
mod store_probe;

fn main() -> anyhow::Result<()> {
    cli::run()
}
