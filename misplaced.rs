#[macro_export]
macro_rules! rebuild_me {
    () => {{
        use misplaced::Shlex;

        let program_source = file!();
        let mut args = std::env::args();
        let Some(program_name) = args.next() else {
            eprintln!("[ERROR] Cannot rebuild self due to lack of program path");
            std::process::exit(1);
        };

        let program_modification_time = std::fs::metadata(&program_name)
            .and_then(|x| x.modified())
            .expect("[ERROR] Cannot get info on self: {}");

        let source_modification_time = std::fs::metadata(&program_source)
            .and_then(|x| x.modified())
            .expect("[ERROR] Cannot get info on one's code: {}");

        if source_modification_time > program_modification_time {
            println!("[INFO] renaming {} -> {}.old", program_name, program_name);
            let old_program_name = program_name.clone() + ".old";
            std::fs::rename(&program_name, old_program_name).expect("[ERROR] Cannot move old self: {}");

            let compiled = std::process::Command::new("rustc")
                .args([program_source, "-o", &program_name])
                .run()
                .unwrap();

            if compiled.success() {
                std::process::exit(std::process::Command::new(program_name)
                    .args(args)
                    .run()
                    .unwrap()
                    .code()
                    .unwrap_or(-1));
            } else {
                std::process::exit(compiled.code().unwrap_or(-1));
            }
        }
    }}
}

pub trait Shlex {
    fn run(&mut self) -> std::io::Result<std::process::ExitStatus>;
}



fn write_quoted<W: std::io::Write>(mut w: W, mut s: &[u8]) -> std::io::Result<()> {
    if s.is_empty() {
        return write!(w, "''");
    }

    let is_safe = s.iter().all(|x| b"@%+=:,./-".contains(x)
                                 || (b'a'..b'z').contains(&x)
                                 || (b'A'..b'Z').contains(&x)
                                 || (b'0'..b'9').contains(&x)
                            );
    if is_safe {
        write!(w, "{}", unsafe { std::str::from_utf8_unchecked(s) })?;
    } else {
        write!(w, "'")?;
        while let Some(quote) = s.iter().position(|&x| x == b'\'') {
            w.write_all(&s[..quote])?;
            s = &s[quote+1..];
            write!(w, "'\"'\"'")?;
        }
        w.write_all(s)?;
        write!(w, "'")?;
    }

    Ok(())
}

impl Shlex for std::process::Command {
    fn run(&mut self) -> std::io::Result<std::process::ExitStatus> {
        use std::io::Write;

        let mut stdout = std::io::stdout();
        write!(stdout, "[CMD] ")?;
        write_quoted(&stdout, self.get_program().as_encoded_bytes())?;
        for arg in self.get_args() {
            write!(stdout, " ")?;
            write_quoted(&stdout, arg.as_encoded_bytes())?;
        }
        writeln!(stdout, "")?;
        self.status()
    }
}

#[test]
fn quoting() {
    // This is not strictly required, we could compare Vec<u8> with byte slice,
    // however we would loose on nicer error messages
    fn quote(s: &[u8]) -> String {
        let mut q: Vec<u8> = vec![];
        write_quoted(&mut q, s).unwrap();
        String::from(unsafe { std::str::from_utf8_unchecked(&q) })
    }

    assert!("hello" == quote(b"hello"));
    assert!("hello/world" == quote(b"hello/world"));
    assert!("'hello world'" == quote(b"hello world"));
    assert!("'hello*world'" == quote(b"hello*world"));
    assert!("'\"hello\" world'" == quote(b"\"hello\" world"));
    assert!("''\"'\"'hello'\"'\"' world'" == quote(b"'hello' world"));
}
