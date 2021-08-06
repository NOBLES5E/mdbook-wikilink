use mdbook::{
    book::{Book, Chapter},
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
    BookItem,
};
use regex::{Captures, Regex};
use std::{collections::HashMap, io};

static RE_EXPR: &str = r"\[\[(?P<link>[^\]\|]+)(?:\|(?P<title>[^\]]+))?\]\]";

pub fn handle_preprocessing(pre: impl Preprocessor) -> Result<(), Error> {
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

fn normalize_string(s: &str) -> String {
    s.replace(" ", "%20")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

pub struct WikiLinks;

impl Preprocessor for WikiLinks {
    fn name(&self) -> &str {
        "wikilink-preprocessor"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let re = Regex::new(RE_EXPR).unwrap();

        let chapters = book
            .iter()
            .filter_map(chapter)
            .filter_map(|it| {
                let path = it.path.as_ref()?.with_extension("");
                let path = path.to_str().unwrap();

                let key = normalize_string(path);

                Some((key, it.name.clone()))
            })
            .collect::<HashMap<_, _>>();

        book.for_each_mut(|it| {
            if let BookItem::Chapter(chapter) = it {
                chapter.content = re
                    .replace_all(&chapter.content, |it: &Captures| -> String {
                        let link_internals = normalize_string(it.get(1).unwrap().as_str());
                        let file = link_internals.to_string();

                        let link = it
                            .get(2)
                            .map(|it| it.as_str().trim().to_string())
                            .unwrap_or_else(|| {
                                if let Some(name) = chapters.get(&file) {
                                    format!("[{}](</{}.md>)", name, &file)
                                } else {
                                    link_internals
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

#[cfg(test)]
mod tests {
    use crate::{normalize_string, RE_EXPR};
    use regex::Regex;

    #[test]
    fn normalize_string_symbols() {
        let cases = [
            ("/Folder/My File <>.md", "/Folder/My%20File%20&lt;&gt;.md"),
            (
                "/👩‍🌾 Gardening Tips/🪴 Sowing<Your>Garden/🎯  Create Custom Dashboards.md", 
                "/👩‍🌾%20Gardening%20Tips/🪴%20Sowing&lt;Your&gt;Garden/🎯%20%20Create%20Custom%20Dashboards.md"
            ),
        ];

        for (case, expected) in cases {
            assert_eq!(normalize_string(case), expected)
        }
    }

    #[test]
    fn extract_link_regex() {
        let re = Regex::new(RE_EXPR).unwrap();
        let cases = [
            ("[[Link]]", "Link"),
            ("[[🪴 Sowing<Your>Garden]]", "🪴 Sowing<Your>Garden"),
            (
                "[[/Templates/🪴 Sowing<Your>Garden]]",
                "/Templates/🪴 Sowing<Your>Garden",
            ),
        ];

        for (case, expected) in cases {
            let got = re.captures(case).unwrap().name("link").unwrap().as_str();
            assert_eq!(got.trim(), expected);
        }
    }

    #[test]
    fn extract_title_regex() {
        let re = Regex::new(RE_EXPR).unwrap();
        let cases = [
            ("[[Link | My New Link]]", "My New Link"),
            ("[[🪴 Sowing<Your>Garden | 🪴 Emoji Link]]", "🪴 Emoji Link"),
            ("[[🪴 Sowing<Your>Garden | 🪴/Emoji/Link]]", "🪴/Emoji/Link"),
        ];

        for (case, expected) in cases {
            let got = re.captures(case).unwrap().name("title").unwrap().as_str();
            assert_eq!(got.trim(), expected)
        }
    }
}
