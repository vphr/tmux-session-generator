use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::environment::get_info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub window_count: u8,
    pub windows: Option<Vec<Window>>,
}

impl Session {
    pub fn extract_session(input: &str) -> Result<Self> {
        let (first, second) = parse_raw_string_tuple(input)?;

        let name = first
            .trim_end_matches(|v| !char::is_alphanumeric(v))
            .to_string();
        let (second, _) = second
            .trim()
            .split_once(" ")
            .ok_or(anyhow!("could not parse window count"))?;
        let window_count = second.parse::<u8>()?;

        let windows = Window::extract_windows(&name).ok();
        // dbg!(&windows);
        Ok(Session {
            name,
            window_count,
            windows,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub id: u8,
    pub name: String,
    pub layout: String,
    pub pane_count: usize,
    pub panes: Option<Vec<Pane>>,
}

impl Window {
    pub fn extract_windows(session_name: &str) -> Result<Vec<Window>> {
        let raw_windows = get_info(&["list-windows", "-t", session_name]).unwrap();

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
            let pane_count = get_info(&["list-pane", "-t", &window_arg]).unwrap().len();

            let (second, _) = second
                .trim()
                .split_once(" ")
                .ok_or(anyhow!("could not parse window count"))?;

            //remove * and -  from the window name
            let name = second
                .trim()
                .trim_end_matches(|v| !char::is_alphanumeric(v))
                .to_string();

            let panes = Pane::extract_panes(session_name, id).ok();

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pane {
    pub id: u8,
}

impl Pane {
    pub fn extract_panes(session_name: &str, window_id: u8) -> Result<Vec<Pane>> {
        let window_arg = format!("{}:{}", session_name, window_id);
        let panes = get_info(&["list-pane", "-t", &window_arg]).unwrap();
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
}

fn parse_raw_string_tuple(input: &str) -> Result<(&str, &str)> {
    input
        .split_once(":")
        .ok_or(anyhow!("Could not parse raw string into tuple"))

    // let input_vec = input.split_whitespace().take(2).collect::<Vec<_>>();
    //
    // let first = input_vec.first().expect("first element if unaccessable");
    // let second = input_vec.get(1).expect("second element if unaccessable");

    // Ok((first, second))
}

// TODO: Find a better way to do this
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
