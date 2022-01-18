use anyhow::Result;
use clap::Parser;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::process;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use walkdir::DirEntry;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[clap(
    name = "rust-fizzy",
    about = "fuzzy filtering tool",
    version = "0.0.1",
    author = "1ain2"
)]

struct Cli {
    // #[clap(short, long, default_value_t = String::from("./"))]
    // #[clap(short, long)]
    #[clap(default_value_t = String::from(""))]
    path: String,
}

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}

// TODO: パス全体ではなく指定ディレクトリ以下のパスを対象に検索した方が良さそう？
fn display_entries<W: std::io::Write>(
    entries: &Vec<DirEntry>,
    word: &String,
    stdout: &mut RawTerminal<W>,
    left_space: u16,
) {
    // TODO: 引数から継続して、iter -> Vec -> Iterって処理してるけど、いいか？？
    for (i, entry) in entries
        .into_iter()
        .filter(|e| word == "" || e.path().to_str().unwrap().contains(word)) // OPTIMIZE: ここコピーしてる？
        .enumerate()
    {
        if i == 10 {
            break;
        }
        // TODO: iがusizeなので変換が必要（変換できない場合が想定される）けど、現状の実装でOKか？
        let i: u16 = (i + 3).try_into().unwrap();
        write!(
            *stdout,
            "{}{:?}",
            termion::cursor::Goto(left_space, i),
            entry
        )
        .unwrap();
    }
}

// TODO: 処理を分割する
fn main() -> Result<()> {
    // 引数処理
    let cli = Cli::parse();
    let path = PathBuf::from(cli.path);
    let path = env::current_dir().unwrap().join(path);

    if !path.exists() {
        eprintln!("{:?}: No such file or directory", path);
        process::exit(1);
    };

    // 全ファイルの取得
    let entries = WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .collect();

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    const LEFT_SPACE: u16 = 3;

    write!(
        stdout,
        "{}{}Type to filtering files.{}",
        termion::clear::All,
        termion::cursor::Goto(LEFT_SPACE, 1),
        termion::cursor::Hide
    )
    .unwrap();
    stdout.flush().unwrap();

    write!(stdout, "{}>", termion::cursor::Goto(1, 2),).unwrap();
    stdout.flush().unwrap();

    let mut word: String = String::from("");

    display_entries(&entries, &word, &mut stdout, LEFT_SPACE);
    stdout.flush().unwrap(); // TODO: 関数内に持っていくことができなかった

    // TODO: stdinで日本語に対応できていなそう
    // TODO: wordはString型じゃなくて、専用の構造体を用意した方が良さそう
    for c in stdin.keys() {
        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(LEFT_SPACE, 2),
        )
        .unwrap();

        match c.unwrap() {
            Key::Ctrl('c') => break,
            Key::Char(c) => {
                word.push(c);
                println!("{}", word);
            }
            Key::Backspace => {
                word.pop();
                println!("{}", word);
            }
            // Key::Char(c) => println!("{}", c),
            // Key::Alt(c) => println!("^{}", c),
            // Key::Ctrl(c) => println!("*{}", c),
            // Key::Esc => println!("ESC"),
            // Key::Left => println!("←"),
            // Key::Right => println!("→"),
            // Key::Up => println!("↑"),
            // Key::Down => println!("↓"),
            // Key::Backspace => println!("x"),
            _ => {}
        }
        display_entries(&entries, &word, &mut stdout, LEFT_SPACE);
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();

    Ok(())
}
