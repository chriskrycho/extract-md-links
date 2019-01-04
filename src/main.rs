use pulldown_cmark::{Event, Parser, Tag};

#[derive(PartialEq, Eq, Hash)]
enum Link {
    Partial { title: Option<String> },
    Complete { title: Option<String>, text: String },
}

fn main() -> Result<(), Box<std::error::Error + 'static>> {
    let mut args = std::env::args();
    let path = args.nth(1).expect("No argument supplied!");
    let contents = std::fs::read_to_string(&path)?;
    Parser::new(&contents)
        .fold(
            (None, std::collections::HashMap::<String, Link>::new()),
            |(current, mut ls), event| match event {
                Event::Start(Tag::Link(url, title)) => {
                    let key = url.to_string();
                    let tracking_key = url.to_string();

                    let title = if title.len() > 0 {
                        Some(title.into())
                    } else {
                        None
                    };

                    ls.insert(key, Link::Partial { title });
                    (Some(tracking_key), ls)
                }
                Event::Text(s) | Event::InlineHtml(s) => {
                    let new_text = String::from(s);
                    if let Some(url) = current {
                        let key: String = url.to_string();
                        let tracking_key = url.to_string();

                        let link = match ls.get(&key).expect(
                            "We should not be able to `pop` without a previously-set `vec` element",
                        ) {
                            Link::Partial { title } => Link::Complete {
                                title: title.to_owned(),
                                text: new_text,
                            },
                            Link::Complete { title, text } => Link::Complete {
                                title: title.to_owned(),
                                text: text.to_owned() + &new_text,
                            },
                        };

                        ls.insert(key, link);
                        (Some(tracking_key), ls)
                    } else {
                        (None, ls)
                    }
                }
                _ => (None, ls),
            },
        )
        .1
        .iter()
        .for_each(|(url, link)| match link {
            Link::Complete {
                text,
                title: Some(title_text),
            } => println!(r#"[{}]: {} "{}""#, text, url, title_text),
            Link::Complete { text, title: None } => println!("[{}]: {}", text, url),
            Link::Partial {
                title: Some(title_text),
            } => eprintln!(r#"BAD LINK: {} "{}""#, url, title_text),
            Link::Partial { title: None } => eprintln!("BAD LINK: {}", url),
        });

    Ok(())
}
