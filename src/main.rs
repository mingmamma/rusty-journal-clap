fn main() {
    // print!("hello");
    if let Err(err) = rusty_journal_clap::run() {
        eprint!("{err}");
    }
}
