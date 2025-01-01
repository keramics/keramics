/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License. You may
 * obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations
 * under the License.
 */

use std::env;
use std::io;
use std::iter;
use std::process::ExitCode;

use mdbook::book::{Book, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;

struct TablesPreprocessor {}

impl TablesPreprocessor {
    fn new() -> Self {
        Self {}
    }

    fn preprocess_tables(&self, chapter: &mut Chapter) -> Result<String, Error> {
        let mut options: Options = Options::empty();
        options.insert(Options::ENABLE_TABLES);

        let chapter_parser: Parser = Parser::new_ext(&chapter.content, options);

        let mut events: Vec<Event> = Vec::new();
        let mut in_colspan: bool = false;
        let mut in_table: bool = false;
        let mut in_table_head: bool = false;
        let mut table_html: Vec<String> = Vec::new();

        // Note that html::push_html() as used below will convert table body cells to <th> instead of <td>.
        let mut chapter_iter = chapter_parser.into_iter().peekable();

        while let Some(event) = chapter_iter.next() {
            match event {
                Event::End(TagEnd::Table) => {
                    events.push(Event::HardBreak);

                    table_html.push("</table>".to_string());
                    table_html.push("</div>".to_string());

                    let html_string: String = table_html.join("");
                    events.push(Event::Html(CowStr::Boxed(html_string.into())));

                    in_table = false;

                    continue;
                }
                Event::End(TagEnd::TableCell) => {
                    if !in_colspan {
                        if in_table_head {
                            table_html.push("</th>".to_string());
                        } else {
                            table_html.push("</td>".to_string());
                        }
                    }
                    continue;
                }
                Event::End(TagEnd::TableHead) => {
                    table_html.push("</thead>".to_string());

                    in_table_head = false;

                    continue;
                }
                Event::End(TagEnd::TableRow) => {
                    in_colspan = false;
                }
                Event::Start(Tag::Table(_)) => {
                    table_html = Vec::new();
                    table_html.push("<div class=\"table-wrapper\">".to_string());
                    table_html.push("<table>".to_string());

                    in_table = true;

                    continue;
                }
                Event::Start(Tag::TableCell) => {
                    match &chapter_iter.peek() {
                        Some(event) => {
                            match event {
                                Event::InlineHtml(cow_str) => {
                                    let inline_html_string: String = cow_str.clone().into_string();
                                    if inline_html_string.contains("colspan=") {
                                        chapter_iter.next();

                                        in_colspan = true;
                                    }
                                    table_html.push(inline_html_string);
                                },
                                _ => {
                                    if !in_colspan {
                                        if in_table_head {
                                            table_html.push("<th>".to_string());
                                        } else {
                                            table_html.push("<td>".to_string());
                                        }
                                    }
                                },
                            };
                        },
                        None => return Err(Error::msg("Missing next event")),
                    }
                    continue;
                }
                Event::Start(Tag::TableHead) => {
                    table_html.push("<thead>".to_string());

                    in_table_head = true;

                    continue;
                }
                _ => {}
            };
            if in_table {
                let mut html_string: String = String::new();

                html::push_html(&mut html_string, iter::once(event));
                table_html.push(html_string);
            } else {
                events.push(event);
            }
        }
        let mut markdown_string: String = String::with_capacity(chapter.content.len());
        Ok(cmark(events.iter(), &mut markdown_string).map(|_| markdown_string)?)
    }
}

impl Preprocessor for TablesPreprocessor {
    fn name(&self) -> &str {
        "tables"
    }

    fn run(&self, _context: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            let BookItem::Chapter(chapter) = item else {
                return;
            };
            if chapter.is_draft_chapter() {
                return;
            }
            match self.preprocess_tables(chapter) {
                Ok(string) => chapter.content = string,
                // Note eprintln!() needs to be used instead of println!() otherwise the mdbook
                // preprocessor will error.
                Err(error) => eprintln!("failed to process chapter: {:?}", error),
            }
        });
        Ok(book)
    }
}

pub fn handle_preprocessing() -> Result<(), Error> {
    let preprocessor: TablesPreprocessor = TablesPreprocessor::new();

    let (context, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let processed_book = preprocessor.run(&context, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);

    match arguments.next().as_deref() {
        Some("supports") => {
            // Supports all renderers.
            return ExitCode::SUCCESS;
        }
        Some(argument) => {
            eprintln!("Unsupported argument: {}", argument);
            return ExitCode::FAILURE;
        }
        None => {}
    }

    if let Err(error) = handle_preprocessing() {
        eprintln!("{}", error);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
