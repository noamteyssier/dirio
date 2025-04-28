use std::{
    fs::File,
    io::{Write, stdout},
    path::Path,
    process::Command,
    thread,
    time::Instant,
};

use anyhow::{Result, bail};
use clap::Parser;
use memchr::memchr;
use serde::Serialize;

#[derive(Serialize)]
pub struct Row {
    pub elapsed: u128,
    pub disk_usage: isize,
    pub delta: isize,
    pub peak: isize,
}
impl Row {
    pub fn new(elapsed: u128, disk_usage: isize, initial_disk_usage: isize, peak: isize) -> Self {
        Self {
            elapsed,
            disk_usage,
            delta: disk_usage - initial_disk_usage,
            peak,
        }
    }
}

pub struct Monitor {
    output: csv::Writer<Box<dyn Write + Send>>,
    start_time: Instant,
    initial_disk_usage: isize,
    peak_disk_usage: isize,
}
impl Monitor {
    pub fn new(writer: Box<dyn Write + Send>, initial_disk_usage: isize) -> Self {
        let output = csv::WriterBuilder::default()
            .delimiter(b'\t')
            .has_headers(true)
            .from_writer(writer);
        Self {
            output,
            start_time: Instant::now(),
            initial_disk_usage,
            peak_disk_usage: initial_disk_usage,
        }
    }
    pub fn add_disk_usage(&mut self, size: isize) -> Result<()> {
        let elapsed = self.start_time.elapsed().as_millis();
        self.peak_disk_usage = self.peak_disk_usage.max(size);
        let row = Row::new(elapsed, size, self.initial_disk_usage, self.peak_disk_usage);
        self.output.serialize(row)?;
        self.output.flush()?;
        Ok(())
    }
}

fn get_disk_usage(path: &str) -> Result<isize> {
    let cmd = Command::new("du").arg("-d").arg("0").arg(path).output()?;
    let whitespace_idx = memchr(b'\t', &cmd.stdout).expect("Failed to find directory size");
    let dir_size_text = std::str::from_utf8(&cmd.stdout[..whitespace_idx])?;
    let dir_size = dir_size_text.parse()?;
    Ok(dir_size)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let directory = Path::new(&args.path);
    if !directory.exists() {
        bail!("Provided directory ({}) does not exist", &args.path);
    }
    if !directory.is_dir() {
        bail!("Provided path ({}) is not a directory", &args.path);
    }
    let output_handle = args.output_handle()?;

    // Initialize the monitor
    let initial_disk_usage = get_disk_usage(&args.path)?;
    let mut monitor = Monitor::new(output_handle, initial_disk_usage);

    // Start the child process
    let mut child = Command::new("sh").arg("-c").arg(&args.command).spawn()?;

    // Start the monitoring thread
    let monitor = thread::spawn(move || -> Result<()> {
        // Loop until the child process exits
        while child.try_wait()?.is_none() {
            let dir_size = get_disk_usage(&args.path)?;
            monitor.add_disk_usage(dir_size)?;
            std::thread::sleep(std::time::Duration::from_millis(args.rate));
        }

        let dir_size = get_disk_usage(&args.path)?;
        monitor.add_disk_usage(dir_size)?;

        Ok(())
    });
    monitor.join().unwrap()?;

    Ok(())
}

#[derive(Parser)]
pub struct Cli {
    #[clap(required = true)]
    pub command: String,

    /// The rate at which to measure the directory disk usage (in milliseconds)
    #[clap(short, long, default_value = "100")]
    pub rate: u64,

    /// The path to the directory to measure disk usage for
    #[clap(short, long, default_value = ".")]
    pub path: String,

    /// The path to the output [default: stdout]
    #[clap(short, long)]
    pub output: Option<String>,
}
impl Cli {
    pub fn output_handle(&self) -> Result<Box<dyn Write + Send>> {
        match &self.output {
            Some(path) => Ok(Box::new(File::create(path)?)),
            None => Ok(Box::new(stdout())),
        }
    }
}
