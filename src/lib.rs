use mdbook::{book::Book, errors::Error, preprocess::PreprocessorContext};
use mdbook::{
    book::Chapter,
    preprocess::{CmdPreprocessor, Preprocessor},
    BookItem,
};
use regex::{Captures, Regex};
use std::{collections::HashMap, io};

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

pub struct AutoTitle {
    re: Regex,
}

impl AutoTitle {
    pub fn new() -> AutoTitle {
        AutoTitle {
            re: Regex::new(r"\[\[([^\]\|]+)(?:\|([^\]]+))?\]\]").unwrap(),
        }
    }
}

impl Preprocessor for AutoTitle {
    fn name(&self) -> &str {
        "autotitle-preprocessor"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let root = ".";

        let chapters = book
            .iter()
            .filter_map(chapter)
            .filter_map(|it| {
                let path = it.path.as_ref()?.file_stem().unwrap().to_str().unwrap();
                Some((root.to_owned() + "/" + path, it.name.clone()))
            })
            .collect::<HashMap<_, _>>();

        //dbg!(&chapters);

        book.for_each_mut(|it| {
            if let BookItem::Chapter(chapter) = it {
                chapter.content = self
                    .re
                    .replace_all(&chapter.content, |it: &Captures| {
                        let file = it.get(1).unwrap().as_str().trim();
                        let title = it.get(2).map(|it| it.as_str().trim());

                        let file = root.to_owned() + "/" + file;
                        let title = title.unwrap_or_else(|| chapters.get(&file).unwrap().trim());

                        format!("[{}]({}.md)", title, file)
                    })
                    .to_string();
            }
        });

        Ok(book)
    }
}
