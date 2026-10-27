use std::io::{self, BufRead, Write};

/// Asks which profile to open, by number. Empty input picks the first.
pub fn pick(names: &[String]) -> io::Result<String> {
    let mut stdout = io::stdout();

    writeln!(stdout, "Profiles:")?;

    for (index, name) in names.iter().enumerate() {
        writeln!(stdout, "  {}) {name}", index + 1)?;
    }

    loop {
        write!(stdout, "Open which profile? [1] ")?;
        stdout.flush()?;

        let mut answer = String::new();

        if io::stdin().lock().read_line(&mut answer)? == 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }

        let answer = answer.trim();

        if answer.is_empty() {
            return Ok(names[0].clone());
        }

        if let Some(name) = answer
            .parse::<usize>()
            .ok()
            .and_then(|number| number.checked_sub(1))
            .and_then(|index| names.get(index))
        {
            return Ok(name.clone());
        }

        writeln!(stdout, "Enter a number from 1 to {}", names.len())?;
    }
}
