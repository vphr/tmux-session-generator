use std::{
    error::Error,
    fs::{write, File},
    io::{Read, Write},
    process::Command,
};

extern crate xdg;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug)]
struct Environment {
    inside_tmux: bool,
    full_path: String,
    working_directory: String,
    session: Option<Session>,
}

impl Environment {
    fn new() -> Result<Self> {
        let key = "TMUX";
        let inside_tmux = std::env::var(key).is_ok();
        let path = std::env::current_dir()?;
        let full_path = path.to_string_lossy().to_string();
        let working_directory = full_path.split("/").into_iter().last().unwrap().to_string();
        Ok(Self {
            inside_tmux,
            full_path,
            working_directory,
            session: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Session {
    name: String,
    window_count: u8,
    windows: Option<Vec<Window>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Window {
    id: u8,
    name: String,
    layout: String,
    pane_count: usize,
    panes: Option<Vec<Pane>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pane {
    id: u8,
}

fn main() -> Result<()> {
    let mut env = Environment::new()?;

    let generate_file = false;
    if generate_file {
        match generate_tmux_config() {
            Ok(()) => println!("Wrote to file"),
            Err(e) => eprintln!("Failed to write to file with error: {}", e),
        };
    }
    let read_file = true;

    if read_file {
        let mut file = File::open("example.yaml")?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let sessions: Vec<Session> = serde_yaml::from_str(&contents)?;

        if sessions.is_empty() || sessions.len() > 1 {
            return Err(anyhow!(
                "Invalid configuration file provided with length: {}, expected length 1",
                sessions.len()
            ));
        }
        env.session = Some(sessions[0].to_owned());
        execute_tmux_config(env);
    }
    Ok(())
}

fn execute_tmux_config(env: Environment) {
    let config = env.session.unwrap();

    run_tmux_command(&["new-session", "-d", "-s", &config.name]);

    let config_windows = config.windows.as_ref().unwrap();

    let (first_window, rest) = config_windows.split_first().unwrap();

    let initial_window_index = &get_tmux_info(&["list-windows", "-t", &config.name]).unwrap()[0];

    let (initial_window_index, _) = initial_window_index.split_once(":").unwrap();

    run_tmux_command(&[
        "rename-window",
        "-t",
        &format!("{}:{}", &config.name, &initial_window_index),
        &first_window.name,
    ]);

    let window_arg = format!("{}:{}", &config.name, &first_window.id);
    for _ in 0..first_window.pane_count - 1 {
        //tmux splitw -h -p 5 -t sess:0.0
        run_tmux_command(&["split-window", "-h", "-t", &window_arg]);
    }
    run_tmux_command(&["select-layout", "-t", &window_arg, &first_window.layout]);

    for window in rest {
        run_tmux_command(&["new-window", "-d", "-t", &config.name, "-n", &window.name]);
        let window_arg = format!("{}:{}", &config.name, &window.id);
        for _ in 0..window.pane_count - 1 {
            //tmux splitw -h -p 5 -t sess:0.0
            run_tmux_command(&["split-window", "-h", "-t", &window_arg]);
        }
        run_tmux_command(&["select-layout", "-t", &window_arg, &window.layout]);
    }
    
    
    if env.inside_tmux{
    run_tmux_command(&["switch", "-t", &config.name]);
    }
    else {
    run_tmux_command(&["attach", "-t", &config.name]);
    }
}

fn run_tmux_command(args: &[&str]) {
    let mut cmd = Command::new("tmux");
    cmd.args(args)
        .spawn()
        .expect("failed to kill session")
        .wait();
}

fn generate_tmux_config() -> Result<()> {
    let raw_sessions = get_tmux_info(&["list-sessions"]).unwrap();
    let mut parsed_sessions: Vec<Session> = vec![];
    for session in raw_sessions {
        match extract_session(&session) {
            Ok(val) => {
                println!("successfully parsed session: {}", val.name);
                parsed_sessions.push(val)
            }
            Err(e) => eprintln!("could not process session: {}", e),
        }
    }
    let yaml = serde_yaml::to_string(&parsed_sessions).unwrap();
    Ok(write("example.yaml", &yaml)?)
}

fn get_tmux_info(args: &[&str]) -> Result<Vec<String>> {
    let mut cmd = Command::new("tmux");
    let output = cmd.args(args).output()?;

    Ok(std::str::from_utf8(&output.stdout)?
        .split_terminator('\n')
        .map(|v| v.to_string())
        .collect())
}
fn parse_raw_string_tuple(input: &str) -> Result<(&str, &str)> {
    let input_vec = input.split_whitespace().take(2).collect::<Vec<_>>();

    let first = input_vec.first().expect("first element if unaccessable");
    let second = input_vec.get(1).expect("second element if unaccessable");

    Ok((first, second))
}

fn extract_session(input: &str) -> Result<Session, Box<dyn Error>> {
    let (first, second) = parse_raw_string_tuple(input)?;

    let name = first
        .trim_end_matches(|v| !char::is_alphabetic(v))
        .to_string();
    let window_count = second.parse::<u8>()?;

    let windows = extract_windows(&name).ok();
    Ok(Session {
        name,
        window_count,
        windows,
    })
}

//TODO: Find a better way to do this
fn extract_layout(window: &str) -> String {
    let mut pane_layout = window.chars().peekable();
    let mut layout = String::new();
    let mut flag = false;
    while let Some(ch) = &pane_layout.next() {
        if *ch == '[' && pane_layout.peek() == Some(&'l') {
            flag = true;
            loop {
                if pane_layout.next() == Some(' ') {
                    break;
                }
                pane_layout.next();
            }
            continue;
        }
        if *ch == ']' && pane_layout.peek() == Some(&' ') && flag {
            break;
        }
        if flag {
            layout.push(*ch);
        }
    }
    layout
}

fn extract_windows(session_name: &str) -> Result<Vec<Window>> {
    let raw_windows = get_tmux_info(&["list-windows", "-t", session_name]).unwrap();

    let mut windows: Vec<Window> = vec![];

    for window in raw_windows {
        let layout = extract_layout(&window);

        let (first, second) = parse_raw_string_tuple(&window)?;

        let id: u8 = first
            .chars()
            .filter(|v| v.is_numeric())
            .collect::<String>()
            .parse()?;

        let window_arg = format!("{}:{}", session_name, id);
        let pane_count = get_tmux_info(&["list-pane", "-t", &window_arg])
            .unwrap()
            .len();

        //remove * and -  from the window name
        let name = second
            .trim_end_matches(|v| !char::is_alphanumeric(v))
            .to_string();

        let panes = extract_panes(session_name, id).ok();

        windows.push(Window {
            name,
            id,
            panes,
            pane_count,
            layout,
        });
    }
    Ok(windows)
}

fn extract_panes(session_name: &str, window_id: u8) -> Result<Vec<Pane>> {
    let window_arg = format!("{}:{}", session_name, window_id);
    let panes = get_tmux_info(&["list-pane", "-t", &window_arg]).unwrap();
    let mut pane_vec: Vec<Pane> = vec![];

    for pane in panes {
        let (first, _) = parse_raw_string_tuple(&pane)?;
        let id: u8 = first
            .chars()
            .filter(|v| v.is_numeric())
            .collect::<String>()
            .parse()?;

        pane_vec.push(Pane { id });
    }
    Ok(pane_vec)
}
