use assert_cmd::crate_name;
use assert_cmd::Command;
use std::env;

#[test]
fn examples() {
    for meta in rew::commands::METAS {
        for example in meta.examples {
            println!("[{}] {}", meta.name, first_line(example.text));

            Command::cargo_bin(crate_name!())
                .unwrap()
                .env("SHELL", "sh") // Examples expect UNIX shell
                .arg(meta.name)
                .args(example.args)
                .write_stdin(join_lines(example.input))
                .assert()
                .success()
                .stdout(join_lines(example.output))
                .stderr("");
        }
    }
}

fn first_line(text: &str) -> &str {
    text.split('\n').next().unwrap_or_default()
}

fn join_lines(lines: &[&str]) -> String {
    format!("{}\n", lines.join("\n"))
}
