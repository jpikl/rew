use assert_cmd::crate_name;
use assert_cmd::Command;

#[test]
fn examples() {
    for meta in rew::commands::METAS {
        for example in meta.examples {
            println!("[{}] {}", meta.name, example.name);

            Command::cargo_bin(crate_name!())
                .unwrap()
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

fn join_lines(lines: &[&str]) -> String {
    format!("{}\n", lines.join("\n"))
}
