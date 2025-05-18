use async_process::{Command, Stdio};
use std::io::{Read, Write};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("exceeded timelimit of {timeout:?}")]
    Timeout { timeout: Duration },
    #[error("process io error")]
    IO(#[from] std::io::Error),
}

// should return  both stdin, stdout
pub async fn secure_run_python_code(
    code: &str,
    timeout: Duration,
) -> Result<std::process::Output, Error> {
    let mut file = tempfile::NamedTempFile::new_in("./python_dir").unwrap();

    // Add header code to temp file
    let mut header = std::fs::File::open("./python_dir/header.py").unwrap();
    let mut buf = Vec::new();
    header.read_to_end(&mut buf).unwrap();
    file.write_all(&buf).unwrap();
    // then add user code to temp file
    file.write_all(code.as_bytes()).unwrap();

    // run temp file (python)
    let python_process = Command::new("python3")
        .arg(file.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output();

    Ok(tokio::time::timeout(timeout, python_process)
        .await
        .map_err(|_| Error::Timeout { timeout })??)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hello_world() {
        let output = secure_run_python_code("print(1+1)", Duration::from_secs(2))
            .await
            .unwrap();
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, b"2\n");
    }

    #[tokio::test]
    async fn capture_error() {
        let output = secure_run_python_code("print(1/0)", Duration::from_secs(2))
            .await
            .unwrap();
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("ZeroDivisionError"));
    }

    #[tokio::test]
    async fn allow_whitelist_import() {
        let output =
            secure_run_python_code("import math; print(math.sqrt(4))", Duration::from_secs(2))
                .await
                .unwrap();
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, b"2.0\n");
    }

    #[tokio::test]
    async fn prevent_other_import() {
        let output = secure_run_python_code("import os; print(os.listdir)", Duration::from_secs(2))
            .await
            .unwrap();
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("secure_importer"));
    }

    #[tokio::test]
    async fn prevet_other_import_in_exec() {
        let output = secure_run_python_code(
            "exec('import os; print(os.listdir)')",
            Duration::from_secs(2),
        )
        .await
        .unwrap();
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("secure_importer"));
    }

    #[tokio::test]
    async fn prevent_importlib_workaround() {
        let output = secure_run_python_code(
            "os = importlib.__import__('os'); print(os.listdir())",
            Duration::from_secs(2),
        )
        .await
        .unwrap();
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }

    #[tokio::test]
    async fn prevent_importlib_workaround_in_exec() {
        let output = secure_run_python_code(
            "exec('os = importlib.__import__(\"os\")\nprint(os.listdir())')",
            Duration::from_secs(2),
        )
        .await
        .unwrap();
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }

    #[tokio::test]
    async fn prevent_loader_workaround() {
        let output = secure_run_python_code(
            "os = __loader__.load_module('os'); print(os.listdir())",
            Duration::from_secs(2),
        )
        .await
        .unwrap();
        assert!(output.stdout.is_empty());
        dbg!(String::from_utf8_lossy(&output.stderr));
        assert!(!output.stderr.is_empty());
    }

    #[tokio::test]
    async fn prevent_vardict_workaround() {
        let output = secure_run_python_code(
            "os = __builtins__.__dict__['os']; print(os.listdir())",
            Duration::from_secs(2),
        )
        .await
        .unwrap();
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
