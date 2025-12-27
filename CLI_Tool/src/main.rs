use clap::Parser;
use maud::{DOCTYPE , Markup , html};
use pulldown_cmark::{Options , Parser as MarkdownParser , html};
use std::{fs , path::PathBuf};

#[derive(Parser , Debug)]
struct Args {
    #[arg(long , short)]
    input: String

    #[arg(long , short)]
    output: PathBuf
}


fn main() {
    let args = Args::parse();
    let markdown_input = fs::read_to_string(&args.input).expect("Failed to read the input");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = MarkdownParser::new_ext(&markdown_input , options);

}
