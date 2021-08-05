use mdbook::{
    book::{Chapter, Book},
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
    BookItem,
    errors::Error,
};
use regex::{Captures, Regex};
use std::{collections::HashMap, io};
use urlencoding::encode;

pub fn handle_preprocessing(pre: impl Preprocessor) -> Result<(), Error> {
    // let mut input = String::new();
    // io::stdin().read_to_string(&mut input).unwrap();
    // eprintln!("{}", input);
    // let (ctx, book) = CmdPreprocessor::parse_input(Cursor::new(input))?;
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    //serde_json::to_writer_pretty(io::stderr(), &processed_book);
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn chapter(it: &BookItem) -> Option<&Chapter> {
    if let BookItem::Chapter(ch) = it {
        Some(ch)
    } else {
        None
    }
}

pub struct WikiLinks;

impl Preprocessor for WikiLinks {
    fn name(&self) -> &str {
        "wikilink-preprocessor"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let re = Regex::new(r"\[\[([^\]\|]+)(?:\|([^\]]+))?\]\]").unwrap();
        let root = ".";

        let chapters = book
            .iter()
            .filter_map(chapter)
            .filter_map(|it| {
                let path = it.path.as_ref()?.file_stem().unwrap().to_str().unwrap();
                let key = root.to_owned() + "/" + path;
                Some((key, it.clone()))
            })
            .collect::<HashMap<_, _>>();

        //dbg!(&chapters);

        book.for_each_mut(|it| {
            if let BookItem::Chapter(chapter) = it {
                chapter.content = re
                    .replace_all(&chapter.content, |it: &Captures| -> String {
                        let link_internals = it.get(1).unwrap().as_str().trim().to_string();
                        let file = root.to_owned() + "/" + &link_internals;

                        let link = it.get(2)
                            .map(|it| it.as_str().trim().to_string())
                            .unwrap_or_else(|| {
                                if let Some(linked_chapter) = chapters.get(&file) {
                                    let url = pathdiff::diff_paths(
                                        linked_chapter.path.as_ref().unwrap(),
                                        chapter.path.as_ref().unwrap().parent().unwrap(),
                                    ).unwrap().to_string_lossy().to_string();
                                    format!("[{}]({})", linked_chapter.name,
                                            url.split("/").map(encode).collect::<Vec<_>>().join("/"))
                                } else {
                                    panic!("{} not found", link_internals)
                                }
                            });

                        link
                    })
                    .to_string();
            }
        });

        Ok(book)
    }
}
