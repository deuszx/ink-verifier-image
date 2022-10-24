use clap::Parser;
use std::{
    env,
    io::{Error, ErrorKind},
    path::PathBuf,
    process::{Command, ExitStatus},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

mod pg;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ink! verifier image tag
    #[arg(short, long, default_value = "develop")]
    tag: String,

    /// Source folder [default: $CWD]
    #[arg(short, long, value_parser)]
    source: Option<PathBuf>,

    /// Container engine
    #[arg(long, default_value = "docker")]
    engine: String,
}

/// Executes the contract build process.
///
/// This function will spawn a guarded child process for docker.
/// It requires the docker command to be installed in the system.
fn exec_build(args: Args) -> Result<ExitStatus, Error> {
    let tag = args.tag;
    let path: PathBuf = args.source.unwrap_or(env::current_dir()?);

    assert!(path.exists());

    let build_dir = path.into_os_string().into_string().unwrap();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut pg = pg::ProcessGuard::spawn(
        Command::new(args.engine)
            .args([
                "run",
                "--entrypoint",
                "package-contract",
                "-v",
                &format!("{build_dir}:/build",),
                "--rm",
                &format!("ink-verifier:{tag}"),
            ])
            // TODO: check env handling
            .envs(env::vars()),
    )?;

    while running.load(Ordering::SeqCst) {
        match pg.try_wait()? {
            Some(status) => return Ok(status),
            // Busy wait :/
            None => thread::sleep(Duration::from_millis(400)),
        }
    }

    Err(Error::new(ErrorKind::Interrupted, "Interrupted"))
}

/// Entry point for the CLI program.
fn main() -> Result<(), Error> {
    let args = Args::parse();
    let status = exec_build(args)?;

    match status.code() {
        Some(code) => std::process::exit(code),
        None => std::process::exit(2),
    }
}