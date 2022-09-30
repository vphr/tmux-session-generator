use std::{error::Error, fmt::format, process::Command};



#[derive(Debug)]
struct Session {
    name: String,
    window_count: u8,
}

#[derive(Debug)]
struct Window {
    id: u8,
    name: String,
}

#[derive(Debug)]
struct Pane {
    id: u8,
    size: (u32, u32),
}

fn main() {
    let raw_sessions = get_tmux_info(&["list-sessions"]).unwrap();
    for session in raw_sessions {
        let session = extract_session(&session).unwrap();
        let windows = extract_windows(&session).unwrap();
        extract_panes(&session, &windows);
        // println!("{} {} {:?}", session.name, session.window_count, windows);
    }
    // let tt = get_tmux_info( &["list-windows", "-t", "rust"]);
    // let binding = tt.unwrap();
    // let t3 = binding.split('\n').collect::<Vec<_>>();
    // println!("{:#?}", t3);
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
    // println!("{}", session_name);
    Ok(Session {
        name: session_name,
        window_count: input[1].parse()?,
    })
}

fn extract_windows(session: &Session) -> Result<Vec<Window>, Box<dyn Error>> {
    let raw_windows = get_tmux_info(&["list-windows", "-t", &session.name]).unwrap();

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
        windows.push(Window { name, id });
    }
    Ok(windows)
}

fn extract_panes(session: &Session, windows: &[Window]) {
    for window in windows {
        let window_arg = format!("{}:{}", session.name, window.name);
        let panes = get_tmux_info(&["list-pane", "-t", &window_arg]).unwrap();
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
            let t = Pane {
                id,
                size: (name[0], name[1]),
            };
            println!("{:?}", t);
        }
    }
}
