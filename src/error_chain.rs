use std::collections::HashSet;
use std::error::Error;

pub fn collect_chain(err: &(dyn Error + 'static)) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    let mut cur: Option<&(dyn Error + 'static)> = Some(err);
    while let Some(e) = cur {
        let ptr = (e as *const dyn std::error::Error) as *const () as usize;
        if !seen.insert(ptr) {
            break;
        }
        out.push(e.to_string());
        cur = e.source();
    }
    out
}
