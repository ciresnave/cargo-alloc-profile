use cargo_alloc_profile::reporter::{GroupBy, OutputFormat, ReportOptions, SortBy};
use clap::{Parser, Subcommand, ValueEnum};
use std::process;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    AllocProfile(AllocProfileArgs),
}

#[derive(Clone, ValueEnum)]
enum OutputFormatArg {
    Text,
    Json,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Text => OutputFormat::Text,
            OutputFormatArg::Json => OutputFormat::Json,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum SortByArg {
    Count,
    Size,
    Name,
}

impl From<SortByArg> for SortBy {
    fn from(arg: SortByArg) -> Self {
        match arg {
            SortByArg::Count => SortBy::Count,
            SortByArg::Size => SortBy::Size,
            SortByArg::Name => SortBy::Name,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum GroupByArg {
    Function,
    Module,
    File,
}

impl From<GroupByArg> for GroupBy {
    fn from(arg: GroupByArg) -> Self {
        match arg {
            GroupByArg::Function => GroupBy::Function,
            GroupByArg::Module => GroupBy::Module,
            GroupByArg::File => GroupBy::File,
        }
    }
}

#[derive(Parser)]
#[command(author, version, about = "Profile heap allocations in Rust programs", long_about = None)]
struct AllocProfileArgs {
    #[command(subcommand)]
    command: Commands,

    /// Increase verbosity (can be specified multiple times: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Filter allocations by function name
    #[arg(short, long, global = true)]
    filter: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text", global = true)]
    output: OutputFormatArg,

    /// Only show allocations that occurred at least this many times
    #[arg(long, global = true)]
    min_count: Option<usize>,

    /// Only show allocations that allocated at least this many bytes total
    #[arg(long, global = true)]
    threshold_bytes: Option<usize>,

    /// Sort results by count, size, or name
    #[arg(long, value_enum, default_value = "count", global = true)]
    sort_by: SortByArg,

    /// Show only the top N allocation sites
    #[arg(long, global = true)]
    limit: Option<usize>,

    /// Save profiling data to file for later comparison
    #[arg(long, global = true)]
    save: Option<String>,

    /// Compare current run with previously saved profiling data
    #[arg(long, global = true)]
    compare: Option<String>,

    /// Aggregate allocations by function, module, or file
    #[arg(long, value_enum, default_value = "function", global = true)]
    group_by: GroupByArg,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a binary with allocation profiling
    Run {
        /// Name of the binary to run
        #[arg(long)]
        bin: Option<String>,

        /// Name of the example to run
        #[arg(long)]
        example: Option<String>,

        /// Arguments to pass to the binary
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Run tests with allocation profiling
    Test {
        /// Test name filter
        test_name: Option<String>,

        /// Arguments to pass to cargo test
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Run benchmarks with allocation profiling
    Bench {
        /// Benchmark name filter
        bench_name: Option<String>,

        /// Arguments to pass to cargo bench
        #[arg(last = true)]
        args: Vec<String>,
    },
}

fn main() {
    let CargoCli::AllocProfile(args) = CargoCli::parse();

    let report_options = ReportOptions {
        verbosity: args.verbose,
        filter: args.filter.clone(),
        format: args.output.clone().into(),
        min_count: args.min_count,
        threshold_bytes: args.threshold_bytes,
        sort_by: args.sort_by.clone().into(),
        limit: args.limit,
        save: args.save.clone(),
        compare: args.compare.clone(),
        group_by: args.group_by.clone().into(),
    };

    let result = match args.command {
        Commands::Run {
            bin,
            example,
            args: run_args,
        } => run_command(bin, example, run_args, report_options),
        Commands::Test {
            test_name,
            args: test_args,
        } => test_command(test_name, test_args, report_options),
        Commands::Bench {
            bench_name,
            args: bench_args,
        } => bench_command(bench_name, bench_args, report_options),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_command(
    bin: Option<String>,
    example: Option<String>,
    args: Vec<String>,
    report_options: ReportOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Only print status messages for text output
    if report_options.format == OutputFormat::Text {
        println!("🔍 Starting allocation profiling...\n");
    }

    // Create a temporary file for the profiling output
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join(format!("cargo-alloc-profile-{}.json", process::id()));

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run");

    if let Some(bin_name) = bin {
        cmd.arg("--bin").arg(bin_name);
    } else if let Some(example_name) = example {
        cmd.arg("--example").arg(example_name);
    }

    // Add environment variables to enable profiling and set output file
    cmd.env("CARGO_ALLOC_PROFILE", "1");
    cmd.env("CARGO_ALLOC_PROFILE_OUTPUT", &output_file);

    if !args.is_empty() {
        cmd.arg("--").args(args);
    }

    // In JSON mode, suppress the program's output
    if report_options.format == OutputFormat::Json {
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err("Command failed".into());
    }

    // Read and display the profiling report
    if output_file.exists() {
        match std::fs::read_to_string(&output_file) {
            Ok(json_data) => {
                match serde_json::from_str::<cargo_alloc_profile::ProfileSnapshot>(&json_data) {
                    Ok(snapshot) => {
                        cargo_alloc_profile::Reporter::print_report(snapshot, report_options);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse profiling data: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read profiling data: {}", e);
            }
        }
        // Clean up the temp file
        let _ = std::fs::remove_file(&output_file);
    } else {
        eprintln!("Warning: No profiling data was generated");
    }

    Ok(())
}

fn test_command(
    test_name: Option<String>,
    args: Vec<String>,
    report_options: ReportOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Only print status messages for text output
    if report_options.format == OutputFormat::Text {
        println!("🔍 Running tests with allocation profiling...\n");
    }

    // Create a temporary file for the profiling output
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join(format!("cargo-alloc-profile-{}.json", process::id()));

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("test");

    if let Some(name) = test_name {
        cmd.arg(name);
    }

    cmd.env("CARGO_ALLOC_PROFILE", "1");
    cmd.env("CARGO_ALLOC_PROFILE_OUTPUT", &output_file);
    cmd.args(args);

    // In JSON mode, suppress the program's output
    if report_options.format == OutputFormat::Json {
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err("Tests failed".into());
    }

    // Read and display the profiling report
    if output_file.exists() {
        match std::fs::read_to_string(&output_file) {
            Ok(json_data) => {
                match serde_json::from_str::<cargo_alloc_profile::ProfileSnapshot>(&json_data) {
                    Ok(snapshot) => {
                        cargo_alloc_profile::Reporter::print_report(snapshot, report_options);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse profiling data: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read profiling data: {}", e);
            }
        }
        // Clean up the temp file
        let _ = std::fs::remove_file(&output_file);
    } else {
        eprintln!("Warning: No profiling data was generated");
    }

    Ok(())
}

fn bench_command(
    bench_name: Option<String>,
    args: Vec<String>,
    report_options: ReportOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Only print status messages for text output
    if report_options.format == OutputFormat::Text {
        println!("🔍 Running benchmarks with allocation profiling...\n");
    }

    // Create a temporary file for the profiling output
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join(format!("cargo-alloc-profile-{}.json", process::id()));

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("bench");

    if let Some(name) = bench_name {
        cmd.arg(name);
    }

    cmd.env("CARGO_ALLOC_PROFILE", "1");
    cmd.env("CARGO_ALLOC_PROFILE_OUTPUT", &output_file);
    cmd.args(args);

    // In JSON mode, suppress the program's output
    if report_options.format == OutputFormat::Json {
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err("Benchmarks failed".into());
    }

    // Read and display the profiling report
    if output_file.exists() {
        match std::fs::read_to_string(&output_file) {
            Ok(json_data) => {
                match serde_json::from_str::<cargo_alloc_profile::ProfileSnapshot>(&json_data) {
                    Ok(snapshot) => {
                        cargo_alloc_profile::Reporter::print_report(snapshot, report_options);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse profiling data: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read profiling data: {}", e);
            }
        }
        // Clean up the temp file
        let _ = std::fs::remove_file(&output_file);
    } else {
        eprintln!("Warning: No profiling data was generated");
    }

    Ok(())
}
