#[path = "utils.rs"]
mod utils;

use indoc::indoc;
use predicates::prelude::*;
use std::env;
use std::path::Path;
use utils::rew;

mod no_pattern {
    use super::*;

    #[test]
    fn no_values() {
        rew().assert().success().stdout("").stderr("");
    }

    #[test]
    fn stdin_values() {
        rew()
            .write_stdin("a\nb")
            .assert()
            .success()
            .stdout("a\nb\n")
            .stderr("");
    }

    #[test]
    fn stdin_values_disabled() {
        rew()
            .arg("--no-stdin")
            .write_stdin("a\nb")
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

mod some_pattern {
    use super::*;

    #[test]
    fn no_values() {
        rew().arg("_{}_").assert().success().stdout("").stderr("");
    }

    #[test]
    fn stdin_values() {
        rew()
            .arg("_{}_")
            .write_stdin("a\n\0b")
            .assert()
            .success()
            .stdout("_a_\n_\0b_\n")
            .stderr("");
    }

    #[test]
    fn stdin_values_disabled() {
        rew()
            .arg("_{}_")
            .arg("--no-stdin")
            .write_stdin("a\n\0b")
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    #[test]
    fn args_values() {
        rew()
            .arg("_{}_")
            .arg("a")
            .arg("b")
            .assert()
            .success()
            .stdout("_a_\n_b_\n")
            .stderr("");
    }

    #[test]
    fn args_values_over_stdin() {
        rew()
            .arg("_{}_")
            .arg("a")
            .arg("b")
            .write_stdin("c\nd")
            .assert()
            .success()
            .stdout("_a_\n_b_\n")
            .stderr("");
    }
}

mod input_terminator {
    use super::*;

    #[test]
    fn null() {
        rew()
            .arg("--read-nul")
            .arg("_{}_")
            .write_stdin("a\n\0b")
            .assert()
            .success()
            .stdout("_a\n_\n_b_\n")
            .stderr("");
    }

    #[test]
    fn raw() {
        rew()
            .arg("--read-raw")
            .arg("_{}_")
            .write_stdin("a\n\0b")
            .assert()
            .success()
            .stdout("_a\n\0b_\n")
            .stderr("");
    }

    #[test]
    fn custom() {
        rew()
            .arg("--read=;")
            .arg("_{}_")
            .write_stdin("a;b")
            .assert()
            .success()
            .stdout("_a_\n_b_\n")
            .stderr("");
    }

    #[test]
    fn custom_no_end() {
        rew()
            .arg("--read=;")
            .arg("--read-end")
            .arg("_{}_")
            .write_stdin("a;b")
            .assert()
            .success()
            .stdout("_a_\n")
            .stderr("");
    }
}

mod output_terminator {
    use super::*;

    #[test]
    fn null() {
        rew()
            .arg("--print-nul")
            .arg("_{}_")
            .write_stdin("a\n\0b")
            .assert()
            .success()
            .stdout("_a_\0_\0b_\0")
            .stderr("");
    }

    #[test]
    fn raw() {
        rew()
            .arg("--print-raw")
            .arg("_{}_")
            .write_stdin("a\n\0b")
            .assert()
            .success()
            .stdout("_a__\0b_")
            .stderr("");
    }

    #[test]
    fn custom() {
        rew()
            .arg("--print=;")
            .arg("_{}_")
            .write_stdin("a\nb")
            .assert()
            .success()
            .stdout("_a_;_b_;")
            .stderr("");
    }

    #[test]
    fn custom_no_end() {
        rew()
            .arg("--print=;")
            .arg("--no-print-end")
            .arg("_{}_")
            .write_stdin("a\nb")
            .assert()
            .success()
            .stdout("_a_;_b_")
            .stderr("");
    }
}

mod output_mode {
    use super::*;

    #[test]
    fn diff() {
        rew()
            .arg("--diff")
            .arg("_{}_")
            .write_stdin("a\n\0b")
            .assert()
            .success()
            .stdout(indoc! {"
                <a
                >_a_
                <\0b
                >_\0b_
            "})
            .stderr("");
    }

    #[test]
    fn pretty() {
        rew()
            .arg("--pretty")
            .arg("_{}_")
            .write_stdin("a\n\0b")
            .assert()
            .success()
            .stdout(indoc! {"
                a -> _a_
                \0b -> _\0b_
            "})
            .stderr("");
    }
}

mod counter {
    use super::*;

    #[test]
    fn local() {
        rew()
            .arg("--local-counter=2:3")
            .arg("{}.{c}.{C}")
            .write_stdin(indoc! {"
                a/a
                a/b
                b/a
                b/b
            "})
            .assert()
            .success()
            .stdout(indoc! {"
                a/a.2.1
                a/b.5.2
                b/a.2.3
                b/b.5.4
            "})
            .stderr("");
    }

    #[test]
    fn global() {
        rew()
            .arg("--global-counter=2:3")
            .arg("{}.{c}.{C}")
            .write_stdin(indoc! {"
                a/a
                a/b
                b/a
                b/b
            "})
            .assert()
            .success()
            .stdout(indoc! {"
                a/a.1.2
                a/b.2.5
                b/a.1.8
                b/b.2.11
            "})
            .stderr("");
    }
}

mod regex {
    use super::*;

    #[test]
    fn value() {
        rew()
            .arg("--regex=(\\d+)")
            .arg("{$1}")
            .write_stdin("dir_1/file_2")
            .assert()
            .success()
            .stdout("1\n")
            .stderr("");
    }

    #[test]
    fn filename() {
        rew()
            .arg("--regex-filename=(\\d+)")
            .arg("{$1}")
            .write_stdin("dir_1/file_2")
            .assert()
            .success()
            .stdout("2\n")
            .stderr("");
    }
}

mod errors {
    use super::*;

    #[test]
    fn non_utf8_input() {
        rew()
            .write_stdin(&[0x66, 0x6f, 0x80, 0x6f][..])
            .assert()
            .failure()
            .code(1)
            .stdout("")
            .stderr("error: Value does not have UTF-8 encoding (offset 2)\n");
    }

    #[test]
    fn parse() {
        rew()
            .arg("{")
            .assert()
            .failure()
            .code(3)
            .stdout("")
            .stderr(indoc! {"
                error: Invalid pattern: No matching '}' after expression start
                
                {
                ^
            "});
    }

    #[test]
    fn eval() {
        rew()
            .arg("{P}")
            .write_stdin(indoc! {"
                non-existent
                Cargo.toml
            "})
            .assert()
            .failure()
            .code(4)
            .stdout("")
            .stderr(predicate::str::starts_with(
                "error: 'Canonical path' evaluation failed for value 'non-existent':",
            ));
    }

    #[test]
    fn eval_at_end() {
        rew()
            .arg("--fail-at-end")
            .arg("{P}")
            .write_stdin(indoc! {"
                non-existent
                Cargo.toml
            "})
            .assert()
            .failure()
            .code(4)
            .stdout(format!(
                "{}\n",
                env::current_dir()
                    .unwrap()
                    .join(Path::new("Cargo.toml"))
                    .to_str()
                    .unwrap()
            ))
            .stderr(predicate::str::starts_with(
                "error: 'Canonical path' evaluation failed for value 'non-existent':",
            ));
    }
}

mod working_dir {
    use super::*;

    #[test]
    fn default() {
        rew()
            .arg("{a}")
            .write_stdin("file")
            .assert()
            .success()
            .stdout(format!(
                "{}\n",
                std::env::current_dir()
                    .unwrap()
                    .join("file")
                    .to_str()
                    .unwrap()
            ));
    }

    #[test]
    fn custom_absolute() {
        #[cfg(unix)]
        let root_dir = "/";
        #[cfg(windows)]
        let root_dir = "C:\\";
        rew()
            .arg(format!("--working-directory={}", root_dir))
            .arg("{a}")
            .write_stdin("file")
            .assert()
            .success()
            .stdout(format!("{}file\n", root_dir,));
    }

    #[test]
    fn custom_relative() {
        rew()
            .arg("--working-directory=dir")
            .arg("{a}")
            .write_stdin("file")
            .assert()
            .success()
            .stdout(format!(
                "{}\n",
                std::env::current_dir()
                    .unwrap()
                    .join("dir")
                    .join("file")
                    .to_str()
                    .unwrap()
            ));
    }
}

mod quote {
    use super::*;

    #[test]
    fn single() {
        rew()
            .arg("--quote")
            .arg("_{}_")
            .write_stdin("a\nb")
            .assert()
            .success()
            .stdout("_'a'_\n_'b'_\n")
            .stderr("");
    }

    #[test]
    fn double() {
        rew()
            .arg("--quote")
            .arg("--quote")
            .arg("_{}_")
            .write_stdin("a\nb")
            .assert()
            .success()
            .stdout("_\"a\"_\n_\"b\"_\n")
            .stderr("");
    }
}

mod separator {
    use super::*;

    #[test]
    fn default() {
        rew()
            .arg("{&1};{&2};{&3}")
            .write_stdin("a\tb c123d")
            .assert()
            .success()
            .stdout("a;b;c123d\n")
            .stderr("");
    }

    #[test]
    fn string() {
        rew()
            .arg("--separator= ")
            .arg("{&1};{&2};{&3}")
            .write_stdin("a\tb c123d")
            .assert()
            .success()
            .stdout("a\tb;c123d;\n")
            .stderr("");
    }

    #[test]
    fn regex() {
        rew()
            .arg("--separator-regex=[0-9]+")
            .arg("{&1};{&2};{&3}")
            .write_stdin("a\tb c123d")
            .assert()
            .success()
            .stdout("a\tb c;d;\n")
            .stderr("");
    }
}

mod help {
    use super::*;

    #[test]
    fn main() {
        rew()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not())
            .stderr("");
    }

    #[test]
    fn pattern() {
        rew()
            .arg("--help-pattern")
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not())
            .stderr("");
    }

    #[test]
    fn filters() {
        rew()
            .arg("--help-filters")
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not())
            .stderr("");
    }
}

#[test]
fn explain() {
    rew()
        .arg("--explain")
        .arg("_")
        .assert()
        .success()
        .stdout(indoc! {"
            _
            ^
            
            Constant '_'
            
        "})
        .stderr("");
}

mod blns {
    use super::*;
    use naughty_strings::BLNS;

    #[test]
    fn pattern() {
        for string in BLNS {
            rew()
                .arg("--")
                .arg(string)
                .assert()
                .code(predicate::in_iter(vec![0, 3]));
        }
    }

    #[test]
    fn args() {
        for string in BLNS {
            rew()
                .arg("--print-raw")
                .arg("--")
                .arg("{}")
                .arg(string)
                .assert()
                .code(predicate::in_iter(vec![0, 4]))
                .stdout(*string);
        }
    }

    #[test]
    fn stdin() {
        for string in BLNS {
            rew()
                .arg("--print-raw")
                .arg("{}")
                .write_stdin(string.as_bytes())
                .assert()
                .code(predicate::in_iter(vec![0, 4]))
                .stdout(*string);
        }
    }
}
