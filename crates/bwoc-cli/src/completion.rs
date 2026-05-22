//! `bwoc completion <shell>` — emit a shell completion script.
//!
//! Powered by `clap_complete`. Pipes the generated script to stdout
//! so users can install it however their shell expects:
//!
//!   bash:        `bwoc completion bash >> ~/.bash_completion`
//!                or `eval "$(bwoc completion bash)"` in .bashrc
//!   zsh:         `bwoc completion zsh > ~/.zfunc/_bwoc`
//!                then `fpath+=~/.zfunc` + `autoload -U compinit && compinit`
//!   fish:        `bwoc completion fish > ~/.config/fish/completions/bwoc.fish`
//!   powershell:  `bwoc completion powershell | Out-String | Invoke-Expression`
//!
//! Tab-complete then surfaces every subcommand, every flag, and every
//! ValueEnum choice (--backend claude|gemini|codex|kimi, --lang en|th, etc.)
//! — fulfilling the "easy to use, options should have suggestions" theme.

use std::io;

use clap::CommandFactory;
use clap_complete::{Shell, generate};

pub struct CompletionArgs {
    pub shell: Shell,
}

pub fn run<C: CommandFactory>(args: CompletionArgs) -> i32 {
    let mut cmd = C::command();
    let bin_name = cmd.get_name().to_string();
    let mut stdout = io::stdout();
    generate(args.shell, &mut cmd, bin_name, &mut stdout);
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    #[command(name = "stub")]
    struct StubCli {
        #[arg(long, value_enum, default_value_t = StubChoice::A)]
        choice: StubChoice,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
    enum StubChoice {
        A,
        B,
    }

    #[test]
    fn generates_bash_completion_without_panicking() {
        let mut cmd = StubCli::command();
        let bin = cmd.get_name().to_string();
        let mut sink = Vec::new();
        generate(Shell::Bash, &mut cmd, bin, &mut sink);
        let s = String::from_utf8(sink).unwrap();
        assert!(!s.is_empty(), "completion script empty");
        assert!(
            s.contains("--choice"),
            "expected `--choice` in bash completion, got: {s:?}"
        );
    }

    #[test]
    fn generates_zsh_completion_without_panicking() {
        let mut cmd = StubCli::command();
        let bin = cmd.get_name().to_string();
        let mut sink = Vec::new();
        generate(Shell::Zsh, &mut cmd, bin, &mut sink);
        assert!(!sink.is_empty());
    }
}
