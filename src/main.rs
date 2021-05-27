use wikilink::{handle_preprocessing, Wikilink};

use clap::{App, Arg, SubCommand};
use mdbook::errors::Error;

pub fn make_app() -> App<'static, 'static> {
    App::new("mdbook-wikilink")
        .about("A mdbook preprocessor to support wikilinks")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() -> Result<(), Error> {
    let matches = make_app().get_matches();

    let preprocessor = Wikilink::new();

    if let Some(_sub_args) = matches.subcommand_matches("supports") {
        Ok(())
    } else {
        handle_preprocessing(preprocessor)
    }
}
