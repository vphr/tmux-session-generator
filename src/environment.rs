use std::{fs::File, io::Read,fs::write,  process::Command};

use anyhow::{anyhow, Result};

use crate::{arguments::CreateArgs, object::Session};

#[derive(Debug)]
pub struct Environment {
    inside_tmux: bool,
    full_path: String,
    working_directory: String,
    session: Option<Session>,
}

impl Environment {
    pub fn new() -> Result<Self> {
        let inside_tmux = std::env::var("TMUX").is_ok();
        let path = std::env::current_dir()?;
        let full_path = path.to_string_lossy().to_string();
        let working_directory = full_path
            .split("/")
            .into_iter()
            .last()
            .ok_or(anyhow!(
                "Could not get current working directory from: {}",
                full_path
            ))?
            .to_string();
        Ok(Self {
            inside_tmux,
            full_path,
            working_directory,
            session: None,
        })
    }
    pub fn create_session(&mut self, args: CreateArgs) -> Result<()> {
        let raw_sessions = get_info(&["list-sessions"])?;

        let session_names: Vec<&String> = raw_sessions
            .iter()
            .filter(|raw_session| {
                let (raw_session_name, _) = raw_session.split_once(":").unwrap();
                raw_session_name.eq(&args.session_name)
            })
            .collect();

        let session_name = session_names.get(0).ok_or(anyhow!(
            "could not find the specified session: {} found: {}",
            &args.session_name,
            &raw_sessions.join(",")
        ))?;
        let parsed_session = Session::extract_session(session_name)?;
        dbg!(&parsed_session);
        
        // TODO: save file in XDG_DATA_HOME
        let yaml = serde_yaml::to_string(&parsed_session)?;
        write(format!("{}.yaml",parsed_session.name), &yaml)?;

        Ok(())
    }
    pub fn get_session(&mut self, filename: &str) -> Result<()> {
        // TODO: get file from XDG_DATA_HOME
        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let session: Session = serde_yaml::from_str(&contents)?;

        self.session = Some(session.clone());
        self.execute_config()?;
        Ok(())
    }
    fn execute_config(&mut self) -> Result<()> {
        match &self.session {
            Some(config) => {
                run_command(&["new-session", "-d", "-s", &config.name])?;

                // handle first window differently because it's created when
                // tmux is initially launched.

                let (first_window, rest) = config.windows.as_ref().unwrap().split_first().unwrap();

                let window_index = get_info(&["list-windows", "-t", &config.name])
                    .unwrap()
                    .into_iter()
                    .rev()
                    .collect::<Vec<String>>()
                    .pop()
                    .unwrap();

                let (initial_window_index, _) = window_index.split_once(":").unwrap();
                run_command(&[
                    "rename-window",
                    "-t",
                    &format!("{}:{}", &config.name, &initial_window_index),
                    &first_window.name,
                ])?;

                let window_arg = format!("{}:{}", &config.name, &first_window.id);

                for _ in 1..first_window.pane_count {
                    // Example:
                    // tmux splitw -h -p 5 -t { session_name }:{ window_name}.{pane_id}
                    run_command(&["split-window", "-h", "-t", &window_arg])?;
                }

                for window in rest {
                    run_command(&["new-window", "-d", "-t", &config.name, "-n", &window.name])?;
                    let window_arg = format!("{}:{}", &config.name, &window.id);
                    for _ in 1..window.pane_count {
                        run_command(&["split-window", "-h", "-t", &window_arg])?;
                    }
                    run_command(&["select-layout", "-t", &window_arg, &window.layout])?;
                }
                if self.inside_tmux {
                    run_command(&["switch", "-t", &config.name])?;
                } else {
                    run_command(&["attach", "-t", &config.name])?;
                }

                Ok(())
            }
            None => Err(anyhow!(
                "Could not get session for {}",
                &self.working_directory
            )),
        }
    }

}

pub fn run_command(args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("tmux");
    cmd.args(args).spawn()?.wait()?;
    Ok(())
}

pub fn get_info(args: &[&str]) -> Result<Vec<String>> {
    let mut cmd = Command::new("tmux");
    let output = cmd.args(args).output()?;

    Ok(std::str::from_utf8(&output.stdout)?
        .split_terminator('\n')
        .map(|v| v.to_string())
        .collect())
}

