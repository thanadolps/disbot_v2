use std::io::{Read, Write};
use std::process::{ExitStatus, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

// should return  both stdin, stdout
pub fn run_python_code(code: &str) -> Option<std::process::Output> {
    use std::process::{Command, Stdio};

    let mut file = tempfile::NamedTempFile::new_in("./python_dir").unwrap();

    // Add header code to temp file
    let mut header = std::fs::File::open("./python_dir/header.py").unwrap();
    let mut buf =  Vec::new();
    header.read_to_end(&mut buf).unwrap();
    file.write_all(&buf).unwrap();
    // then add user code to temp file
    file.write_all(code.as_bytes()).unwrap();

    // run temp file (python)
    let mut python_process = Command::new("python3")
        .arg(file.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    match python_process.wait_timeout(Duration::from_secs(2)).unwrap() {
        Some(exit_status) => Some(python_process.wait_with_output().unwrap()),
        None => {
            // timeout
            python_process.kill().unwrap();
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world() {
        run_python_code("print(1+1)");
    }
}
