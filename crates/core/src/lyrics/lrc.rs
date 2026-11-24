use super::Line;

pub fn parse(text: &str) -> Vec<Line> {
    let mut lines: Vec<Line> = text.lines().flat_map(parse_line).collect();
    lines.sort_by_key(|line| line.start_ms);
    lines
}

fn parse_line(line: &str) -> Vec<Line> {
    let mut rest = line.trim();
    let mut starts = Vec::new();
    while let Some((tag, after)) = rest.strip_prefix('[').and_then(|text| text.split_once(']')) {
        let Some(start) = parse_time(tag) else {
            break;
        };
        starts.push(start);
        rest = after.trim_start();
    }
    starts
        .into_iter()
        .map(|start_ms| Line {
            start_ms,
            text: rest.to_owned(),
        })
        .collect()
}

fn parse_time(tag: &str) -> Option<u64> {
    let (minutes, seconds) = tag.split_once(':')?;
    let minutes: u64 = minutes.parse().ok()?;
    let seconds: f64 = seconds.parse().ok()?;
    let is_valid = (0. ..60.).contains(&seconds) && minutes < 600;
    is_valid.then(|| minutes * 60_000 + (seconds * 1000.).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_repeated_times_and_skips_headers() {
        let lines = parse("[ar:Someone]\n[00:10.5][00:02.25] hey \nno time\n[99:99.00]bad");
        let found: Vec<_> = lines
            .iter()
            .map(|line| (line.start_ms, line.text.as_str()))
            .collect();
        assert_eq!(found, [(2_250, "hey"), (10_500, "hey")]);
    }
}
