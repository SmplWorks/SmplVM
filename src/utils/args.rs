use clap::Parser;

/// Virtual Machine for SmplCore
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to file to execute
    pub in_path: String,

    /// Treat file as .sasm and compile it
    #[arg(short, long, default_value_t = false)]
    pub compile : bool,
    
    /// Number of instructions to execute
    #[arg(long)]
    pub reps : usize,
}

impl Args {
    #[allow(unused)]
    pub fn load() -> Self {
        Self::parse()
    }
}
