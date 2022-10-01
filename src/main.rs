use std::{error::Error, process::Command};

#[derive(Debug, Clone)]
struct Session {
    name: String,
    window_count: u8,
    windows: Option<Vec<Window>>,
}

#[derive(Debug, Clone)]
struct Window {
    id: u8,
    name: String,
    panes: Option<Vec<Pane>>,
}

#[derive(Debug, Clone, Copy)]
struct Pane {
    id: u8,
    size: (u32, u32),
}

fn main() {
    let raw_sessions = get_tmux_info(&["list-sessions"]).unwrap();
    for session in raw_sessions {
        let session = extract_session(&session).unwrap();
    }
}

fn get_tmux_info(args: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
    let mut cmd = Command::new("tmux");
    let t = cmd.args(args).output().unwrap();

    let tt = std::str::from_utf8(&t.stdout)
        .unwrap()
        .split_terminator('\n')
        .map(|v| v.to_string())
        .collect();

    Ok(tt)
}

fn extract_session(input: &str) -> Result<Session, Box<dyn Error>> {
    let input: [&str; 2] = input
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let session_name = input[0]
        .trim_end_matches(|v| !char::is_alphabetic(v))
        .to_string();

    let windows = extract_windows(&session_name).ok();
    Ok(Session {
        name: session_name,
        window_count: input[1].parse()?,
        windows,
    })
}

fn extract_windows(session_name: &str) -> Result<Vec<Window>, Box<dyn Error>> {
    let raw_windows = get_tmux_info(&["list-windows", "-t", session_name]).unwrap();

    let mut windows: Vec<Window> = vec![];
    for window in raw_windows {
        let input: [&str; 2] = window
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let id: u8 = input[0][0..input[0].len() - 1].parse().unwrap();

        //remove * and -  from the window name
        let name = input[1]
            .trim_end_matches(|v| !char::is_alphabetic(v))
            .to_string();

        let panes = extract_panes(session_name, id).ok();
        windows.push(Window { name, id, panes });
    }
    Ok(windows)
}

fn extract_panes(session_name: &str, window_id: u8) -> Result<Vec<Pane>, Box<dyn Error>> {
    let window_arg = format!("{}:{}", session_name, window_id);
    let panes = get_tmux_info(&["list-pane", "-t", &window_arg]).unwrap();
    let mut pane_vec: Vec<Pane> = vec![];

    for pane in panes {
        let input: [&str; 2] = pane
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let id: u8 = input[0][0..input[0].len() - 1].parse().unwrap();

        let name: [u32; 2] = input[1]
            .chars()
            .filter(|v| v.is_alphanumeric())
            .collect::<String>()
            .replace('x', " ")
            .split(' ')
            .map(|v| v.parse::<u32>().unwrap())
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();
        pane_vec.push(Pane {
            id,
            size: (name[0], name[1]),
        });
    }
    Ok(pane_vec)
}

