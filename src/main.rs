use std::io::BufReader;
use std::{fs::File, io::Write};

use regex::Regex;
use xml::reader::{EventReader, XmlEvent};

fn main() -> std::io::Result<()> {
    let file = File::open("/home/toma/Downloads/jawiki-latest-pages-articles.xml")?;
    let file = BufReader::new(file);

    let mut outfile = File::create("raw.txt")?;

    let parser = EventReader::new(file);

    let html_re = Regex::new(r"<[^>]*>").unwrap(); // HTMLタグを除去
    let template_re = Regex::new(r"\{\{[^}]*\}\}").unwrap(); // Wikipediaテンプレートを除去
    let link_re = Regex::new(r"\[\[([^|\]]+)(?:\|[^\]]*)?\]\]").unwrap(); // [[リンク]]のテキスト部分を残す
    let quote_re = Regex::new(r"'''[^']*'''").unwrap(); // '''で囲まれた部分を除去
    let skip_re = Regex::new(r"^[\d\W[:alpha:]]").unwrap();
    let url_re = Regex::new(r"https?://[^\s]+|www\.[^\s]+").unwrap();

    let remove_re = Regex::new(r"[\[\]{}()\/\.\:「」『』（）0-9_\-&]").unwrap();

    let mut num_bytes = 0;
    let mut num_mega_bytes = 0;

    for e in parser.into_iter() {
        match e {
            Ok(XmlEvent::Characters(text)) => {
                let text = html_re.replace_all(&text, "");
                let text = template_re.replace_all(&text, "");
                let text = link_re.replace_all(&text, "$1");
                let text = quote_re.replace_all(&text, "");
                let text = url_re.replace_all(&text, "");

                for line in text
                    .lines()
                    .map(str::trim)
                    .filter(|line| line.chars().count() > 500 && !skip_re.is_match(line))
                {
                    let line = remove_re.replace_all(line, "");
                    num_bytes += line.len();
                    if num_bytes / (1024 * 1024) > num_mega_bytes {
                        num_mega_bytes = num_bytes / (1024 * 1024);
                        println!("{} MB processed", num_mega_bytes);
                    }
                    outfile.write_all(line.as_bytes()).unwrap();
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_removal() {
        let re = Regex::new(r"[\[\]{}\/\.\:]|[「」『』]|（|）").unwrap();

        let test_cases = vec![
            // 特殊文字
            ("ああ[いい]{}う/.:", "ああいいう"),
            // 括弧変換
            ("これは（テスト）です", "これはテストです"),
            // 引用符の削除
            (
                "「これは引用」『これは別の引用』",
                "これは引用これは別の引用",
            ),
        ];

        for (input, expected) in test_cases {
            let cleaned_text = re.replace_all(input, "");
            assert_eq!(cleaned_text, expected);
        }
    }

    #[test]
    fn test_line_re() {
        let link_re = Regex::new(r"\[\[([^|\]]+)(?:\|[^\]]*)?\]\]").unwrap();
        let text = "[[日本プロサッカーリーグ|Jリーグ]]には多くの[[チーム]]が所属しています";
        assert_eq!(
            link_re.replace_all(text, "$1"),
            "日本プロサッカーリーグには多くのチームが所属しています"
        );
    }
}
