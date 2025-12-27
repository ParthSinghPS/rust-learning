use clap::Parser;
use maud::{DOCTYPE , Markup , html};
use pulldown_cmark::{Options , Parser as MarkdownParser , html};
use std::{fs , path::PathBuf};

#[derive(Parser , Debug)]
struct Args {
    #[arg(long , short)]
    input: String,

    #[arg(long , short)]
    output: Option<PathBuf>,
}

fn render_html_page (content:&str) -> Markup {
    html!{
        (DOCTYPE)
        html {
            head {
                meta charset = "utf-8";
                title { "Markdown to Html input" }
            }
        body {
            (maud::PreEscaped(content.to_string()))
        }
        }
    }
}


fn main() {
    let args = Args::parse();
    let markdown_input = fs::read_to_string(&args.input).expect("Failed to read the input");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = MarkdownParser::new_ext(&markdown_input , options);

    let mut html_output = String::new();
    html::push_html(&mut html_output , parser);

    let full_html_output = render_html_page(&html_output).into_string();

    match &args.output {
        Some(path) => fs::write(path , full_html_output).expect("Failed to write in the file"),
        None => println!("Path not provided"),
    }

}
