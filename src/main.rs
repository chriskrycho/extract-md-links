use pulldown_cmark::{Event, Parser, Tag};

enum Link {
    Partial {
        url: String,
        title: Option<String>,
    },
    Complete {
        url: String,
        title: Option<String>,
        text: String,
    },
}

fn main() -> Result<(), Box<std::error::Error + 'static>> {
    let mut args = std::env::args();
    let path = args.nth(1).expect("No argument supplied!");
    let contents = std::fs::read_to_string(&path)?;
    Parser::new(&contents)
        .fold(
            (false, vec![]),
            |(in_link, mut ls): (bool, Vec<Link>), event| match event {
                Event::Start(Tag::Link(url, title)) => {
                    let url: String = url.into();

                    let title = if title.len() > 0 {
                        Some(title.into())
                    } else {
                        None
                    };

                    ls.push(Link::Partial { url, title });
                    (true, ls)
                }
                Event::Text(s) | Event::InlineHtml(s) => {
                    let new_text = String::from(s);
                    if in_link {
                        let link = match ls.pop().expect(
                            "We should not be able to `pop` without a previously-set `vec` element",
                        ) {
                            Link::Partial { url, title } => Link::Complete {
                                url,
                                title,
                                text: new_text,
                            },
                            Link::Complete { url, title, text } => Link::Complete {
                                url,
                                title,
                                text: text + &new_text,
                            },
                        };

                        ls.push(link);

                        (true, ls)
                    } else {
                        (false, ls)
                    }
                }
                _ => (false, ls),
            },
        )
        .1
        .iter()
        .for_each(|link| match link {
            Link::Complete {
                url,
                text,
                title: Some(title_text),
            } => println!(r#"[{}]: {} "{}""#, text, url, title_text),
            Link::Complete {
                url,
                text,
                title: None,
            } => println!("[{}]: {}", text, url),
            Link::Partial {
                url,
                title: Some(title_text),
            } => eprintln!(r#"BAD LINK: {} "{}""#, url, title_text),
            Link::Partial { url, title: None } => eprintln!("BAD LINK: {}", url),
        });

    Ok(())
}
