mod environment;
mod object;
mod arguments;
extern crate xdg;
use anyhow::Result;

use environment::Environment;
use clap::Parser;
use arguments::*;

fn main() -> Result<()> {
    let mut env = Environment::new()?;
    match TmuxArgs::parse().action{
        Some(Action::Create(args)) => env.create_session(args),
        Some(Action::Update) => todo!("update existing configuration"),
        Some(Action::Load(file)) => env.get_session(&file.path),
        None => env.get_session("example.yaml"),
    }
}
