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

    /// Enable debugging
    #[arg(long, default_value_t = false)]
    pub debug : bool,

    /// Prompt before beginning execution
    #[arg(long, default_value_t = false)]
    pub first_prompt : bool,

    /// Extra breakpoints to use during execution along with the configuration file
    #[arg(short, long, num_args = 1.., value_delimiter = ',')]
    pub breakpoints : Vec<u16>,
}

impl Args {
    #[allow(unused)]
    pub fn load() -> Self {
        Self::parse()
    }
}
