use std::{error::Error, process::Command, vec};

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

#[derive(Debug, Clone)]
struct Pane {
    id: u8,
    size: (u32, u32),
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

fn extract_windows(session_name: &str) -> Result<Vec<Window>, Box<dyn Error>> {
    let raw_windows = get_tmux_info(&["list-windows", "-t", session_name]).unwrap();

    let mut windows: Vec<Window> = vec![];
    for window in raw_windows {
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
        windows.push(Window { name, id, panes });
    }
    Ok(windows)
}

fn extract_panes(session_name: &str, window_id: u8) -> Result<Vec<Pane>, Box<dyn Error>> {
    let window_arg = format!("{}:{}", session_name, window_id);
    let panes = get_tmux_info(&["list-pane", "-t", &window_arg]).unwrap();
    let mut pane_vec: Vec<Pane> = vec![];

    for pane in panes {
        let (first, second) = parse_raw_string_tuple(&pane)?;
        let id: u8 = first
            .chars()
            .filter(|v| v.is_numeric())
            .collect::<String>()
            .parse()?;

        let name = second
            .chars()
            .filter(|v| v.is_alphanumeric())
            .collect::<String>()
            .replace('x', " ")
            .split(' ')
            .map(|v| v.parse::<u32>().unwrap())
            .collect::<Vec<u32>>();
        pane_vec.push(Pane {
            id,
            size: (name[0], name[1]),
        });
    }
    Ok(pane_vec)
}
