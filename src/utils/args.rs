use clap::Parser;

/// Virtual Machine for SmplCore
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file
    #[arg(default_value = "config.toml")]
    pub config_path : String,

    /// Disable display
    #[arg(long, default_value_t = false)]
    pub no_display : bool,
}

impl Args {
    #[allow(unused)]
    pub fn load() -> Self {
        Self::parse()
    }
}
