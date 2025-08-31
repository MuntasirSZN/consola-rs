use std::{collections::HashSet, error::Error};

pub fn collect_chain(err: &(dyn Error + 'static)) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    let mut cur: Option<&(dyn Error + 'static)> = Some(err);
    while let Some(e) = cur {
        let ptr = (e as *const dyn Error) as *const () as usize;
        if !seen.insert(ptr) {
            break;
        }
        out.push(e.to_string());
        cur = e.source();
    }
    out
}

pub fn format_chain_lines(chain: &[String], max_depth: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for (i, msg) in chain.iter().enumerate() {
        if i >= max_depth {
            break;
        }
        if i == 0 {
            lines.push(msg.to_string());
        } else {
            lines.push(format!("Caused by: {msg}"));
        }
    }
    lines
}
