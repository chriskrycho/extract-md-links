use std::borrow::Cow;
use std::collections::HashMap;

use pulldown_cmark::{Event, Parser, Tag};

#[derive(PartialEq, Eq, Hash, Debug)]
enum Link {
    Partial { title: Option<String> },
    Complete { title: Option<String>, text: String },
}

type Links = HashMap<String, Link>;

fn main() -> Result<(), Box<std::error::Error + 'static>> {
    let mut args = std::env::args();
    let path = args.nth(1).expect("No argument supplied!");
    let contents = std::fs::read_to_string(&path)?;
    let links = links_from_document(&contents)?;

    println!("");

    print_links(&links);
    print_errs(&links);

    Ok(())
}

fn links_from_document(contents: &str) -> Result<Links, String> {
    let (current, links) = Parser::new(&contents).try_fold(
        (None, HashMap::<String, Link>::new()),
        |(current, mut links), event| match event {
            Event::Start(Tag::Link(url, title)) => insert_or_update_link(&url, &title, &mut links)
                .and(Ok((Some(url.to_string()), links))),
            Event::Text(s) | Event::InlineHtml(s) | Event::Html(s) => {
                let tracking = update_link_text(&s, &current, &mut links);
                Ok((tracking, links))
            }
            Event::End(Tag::Link(_, _)) => Ok((None, links)),
            _ => Ok((current, links)),
        },
    )?;

    match current {
        Some(url) => Err(format!("ERROR -- un-closed link: {:#?}", url)),
        None => Ok(links),
    }
}

fn print_links(links: &Links) {
    links.iter().for_each(|(_, link)| match link {
        Link::Complete { text, .. } => println!("- [{}]", text),
        _ => {}
    });

    println!("");

    links.iter().for_each(|(url, link)| match link {
        Link::Complete {
            text,
            title: Some(title_text),
        } => println!(r#"[{}]: {} "{}""#, text, url, title_text),
        Link::Complete { text, title: None } => println!("[{}]: {}", text, url),
        _ => {}
    });
}

fn print_errs(links: &Links) {
    if links.iter().any(|(_, link)| match link {
        Link::Partial { .. } => true,
        _ => false,
    }) {
        eprintln!("\nErrors:");
        links.iter().for_each(|(url, link)| match link {
            Link::Partial {
                title: Some(title_text),
            } => eprintln!(r#"BAD LINK: {} "{}""#, url, title_text),
            Link::Partial { title: None } => eprintln!("BAD LINK: {}", url),
            _ => {}
        });
    }
}

fn insert_or_update_link(
    url: &std::borrow::Cow<str>,
    title: &std::borrow::Cow<str>,
    links: &mut Links,
) -> Result<(), String> {
    let key = url.to_string();

    let title = if title.len() > 0 {
        Some(title.as_ref().to_owned())
    } else {
        None
    };

    // If the key exists already, we can update it if we have a non-`None`
    // `title`; otherwise, we can simply use whatever we have.
    let link = if links.contains_key(&key) && title.is_some() {
        match &links[&key] {
            Link::Partial {
                title: Some(current_title),
            }
            | Link::Complete {
                title: Some(current_title),
                ..
            } => Err(format!(
                "ERROR: attempted to build same link with different titles: {:#?} and {:#?}",
                current_title,
                title.unwrap()
            )),
            Link::Partial { title: None } => Ok(Link::Partial { title }),
            Link::Complete { title: None, text } => Ok(Link::Complete {
                title,
                text: text.as_str().into(),
            }),
        }
    } else {
        Ok(Link::Partial { title })
    }?;

    links.insert(key, link);

    Ok(())
}

fn update_link_text(
    new_text: &Cow<str>,
    current: &Option<String>,
    links: &mut Links,
) -> Option<String> {
    let new_text = new_text.as_ref().to_owned();
    if let Some(url) = current {
        let key: String = url.to_string();
        let tracking_key = url.to_string();

        // Lookup is safe: we just checked that the key is set.
        let link = match &links[&key] {
            Link::Partial { title } => Link::Complete {
                title: title.to_owned(),
                text: new_text,
            },
            Link::Complete { title, text } => Link::Complete {
                title: title.to_owned(),
                text: text.to_owned() + &new_text,
            },
        };

        links.insert(key, link);
        Some(tracking_key)
    } else {
        None
    }
}
