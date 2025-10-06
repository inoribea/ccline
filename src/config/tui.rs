use std::io::{self, BufRead, Write};

#[derive(Clone, Copy, Debug)]
enum ProviderChoice {
    Claude,
    Codex,
    Both,
}

pub fn run_configuration_wizard() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin().lock();

    writeln!(stdout, "\nCCometixLine Configuration Wizard")?;
    writeln!(stdout, "================================\n")?;
    writeln!(stdout, "This guide will print the exact steps needed to wire CCometixLine into Claude Code, Codex CLI, or both.")?;

    let choice = loop {
        writeln!(
            stdout,
            "\nSelect environment:\n  [1] Claude Code\n  [2] Codex CLI\n  [3] Both (side-by-side)"
        )?;
        write!(stdout, "> ")?;
        stdout.flush()?;

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        match line.trim() {
            "1" => break ProviderChoice::Claude,
            "2" => break ProviderChoice::Codex,
            "3" => break ProviderChoice::Both,
            other => {
                writeln!(
                    stdout,
                    "Unknown selection: {}. Please enter 1, 2, or 3.",
                    other
                )?;
            }
        }
    };

    match choice {
        ProviderChoice::Claude => render_claude_instructions(&mut stdout)?,
        ProviderChoice::Codex => render_codex_instructions(&mut stdout)?,
        ProviderChoice::Both => {
            render_claude_instructions(&mut stdout)?;
            writeln!(stdout, "\n----------------------------------------\n")?;
            render_codex_instructions(&mut stdout)?;
        }
    }

    writeln!(stdout, "\nEnvironment Variables")?;
    writeln!(stdout, "----------------------")?;
    writeln!(stdout, "• CLAUDE_CONFIG_DIR  — additional Claude transcript roots (comma separated, appends /projects)")?;
    writeln!(
        stdout,
        "• CODEX_SESSIONS_DIR — additional Codex session roots (comma separated)"
    )?;
    writeln!(
        stdout,
        "• CCLINE_CONFIG_HOME — override where block overrides and update state live"
    )?;
    writeln!(
        stdout,
        "• CCLINE_DISABLE_COST=1 — hide Cost & Burn Rate segments"
    )?;
    writeln!(
        stdout,
        "• CCLINE_SHOW_TIMING=1 — print profiling timings for debugging\n"
    )?;

    prompt_readme_excerpt(&mut stdout, &mut stdin)?;
    Ok(())
}

fn render_claude_instructions<W: Write>(stdout: &mut W) -> io::Result<()> {
    writeln!(stdout, "Claude Code Setup")?;
    writeln!(stdout, "------------------")?;
    writeln!(stdout, "1. Install the binary:")?;
    writeln!(stdout, "   mkdir -p ~/.claude/ccline")?;
    writeln!(stdout, "   install -Dm755 ccline ~/.claude/ccline/ccline\n")?;

    writeln!(stdout, "2. Add to Claude Code settings.json:")?;
    writeln!(
        stdout,
        "   {\n     \"statusLine\": {\n       \"type\": \"command\",\n       \"command\": \"~/.claude/ccline/ccline\",\n       \"padding\": 0\n     }\n   }\n"
    )?;

    writeln!(
        stdout,
        "3. Make sure transcripts exist under ~/.config/claude/projects or ~/.claude/projects.\n"
    )?;
    Ok(())
}

fn render_codex_instructions<W: Write>(stdout: &mut W) -> io::Result<()> {
    writeln!(stdout, "Codex CLI Setup")?;
    writeln!(stdout, "----------------")?;
    writeln!(stdout, "1. Install the binary:")?;
    writeln!(stdout, "   mkdir -p ~/.codex/ccline")?;
    writeln!(stdout, "   install -Dm755 ccline ~/.codex/ccline/ccline\n")?;

    writeln!(stdout, "2. Add to ~/.codex/config.toml:")?;
    writeln!(
        stdout,
        "   [status_line]\n   type = \"command\"\n   command = \"~/.codex/ccline/ccline\"\n   padding = 0\n"
    )?;

    writeln!(stdout, "3. Codex stores sessions in ~/.codex/sessions/YYYY/MM/DD (override with CODEX_SESSIONS_DIR).\n")?;
    Ok(())
}

fn prompt_readme_excerpt<W: Write, R: BufRead>(stdout: &mut W, stdin: &mut R) -> io::Result<()> {
    writeln!(
        stdout,
        "Would you like to preview the README instructions now? [y/N]"
    )?;
    write!(stdout, "> ")?;
    stdout.flush()?;

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    if !matches!(line.trim().to_lowercase().as_str(), "y" | "yes") {
        writeln!(
            stdout,
            "\nTip: Full instructions live in README.md and README.zh.md.\n"
        )?;
        return Ok(());
    }

    loop {
        writeln!(
            stdout,
            "Show which document? [1] README (English)  [2] README.zh (中文)  [3] Back"
        )?;
        write!(stdout, "> ")?;
        stdout.flush()?;

        line.clear();
        stdin.read_line(&mut line)?;
        match line.trim() {
            "1" => {
                print_excerpt(stdout, include_str!("../../README.md"))?;
                break;
            }
            "2" => {
                print_excerpt(stdout, include_str!("../../README.zh.md"))?;
                break;
            }
            "3" => {
                writeln!(stdout, "Returning to shell...\n")?;
                break;
            }
            other => {
                writeln!(
                    stdout,
                    "Unknown selection: {}. Please enter 1, 2, or 3.",
                    other
                )?;
            }
        }
    }

    Ok(())
}

fn print_excerpt<W: Write>(stdout: &mut W, doc: &str) -> io::Result<()> {
    let preview: String = doc
        .lines()
        .take(32)
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    writeln!(
        stdout,
        "\n---- README excerpt ----\n{}\n------------------------\n",
        preview
    )
}
