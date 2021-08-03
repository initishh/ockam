use anyhow::Result;
use duct::cmd;
use ron::de::from_str;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub type Step = String;

pub type Stage = Vec<Step>;

#[derive(Deserialize, Debug)]
pub struct Script {
    title: String,
    stages: Vec<Stage>,
}

fn run_stage(stage: Stage, stop: Arc<AtomicBool>) -> Result<()> {
    let mut procs = Vec::new();
    for mut step in stage {
        let stop = stop.clone();
        if step.starts_with("sleep ") {
            let duration = step.split_off(6);
            let duration = duration.trim();
            let duration: u64 = duration.parse()?;
            println!("Sleeping for {} seconds", duration);
            std::thread::sleep(Duration::from_secs(duration));
            continue;
        }
        let j = std::thread::spawn(move || {
            let cmd = cmd!("cargo", "run", "--example", step);
            let handle = cmd.start().unwrap();
            while !stop.load(Ordering::Release) {
                std::thread::sleep(Duration::from_secs(1));
            }
            handle.kill().unwrap();
        });
        procs.push(j);
        std::thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}

fn run(script: Script) -> Result<()> {
    println!("Running {}", script.title);
    let stop = Arc::new(AtomicBool::new(false));
    for stage in script.stages {
        run_stage(stage, stop.clone())?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let file = std::env::args()
        .skip(1)
        .next()
        .expect("missing script file");
    let mut file = File::open(file).expect("unable to open script");
    let mut guide = String::new();

    file.read_to_string(&mut guide)?;

    let script: Script = from_str(guide.as_str()).expect("script is invalid");
    run(script)
}
