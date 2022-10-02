use std::{error::Error, fs::write, process::Command};

use serde::{Deserialize, Serialize};

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
    panes: Option<Vec<Pane>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pane {
    id: u8,
}

fn main() {
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
    match write("example.yaml", &yaml) {
        Ok(()) => println!("Wrote to file"),
        Err(e) => eprintln!("Failed to write to file with error: {}", e),
    };
    //print!("{:#}", yaml);
}

fn get_tmux_info(args: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
    let mut cmd = Command::new("tmux");
    let output = cmd.args(args).output()?;

    Ok(std::str::from_utf8(&output.stdout)?
        .split_terminator('\n')
        .map(|v| v.to_string())
        .collect())
}
fn parse_raw_string_tuple(input: &str) -> Result<(&str, &str), Box<dyn Error>> {
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
        if *ch == ']' && flag {
            break;
        }
        if flag {
            layout.push(*ch);
        }
    }
    println!("{}", &layout);
    layout
}

fn extract_windows(session_name: &str) -> Result<Vec<Window>, Box<dyn Error>> {
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

        //remove * and -  from the window name
        let name = second
            .trim_end_matches(|v| !char::is_alphanumeric(v))
            .to_string();
        let panes = extract_panes(session_name, id).ok();
        windows.push(Window {
            name,
            id,
            panes,
            layout,
        });
    }
    Ok(windows)
}

fn extract_panes(session_name: &str, window_id: u8) -> Result<Vec<Pane>, Box<dyn Error>> {
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
