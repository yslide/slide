//! Module `utils` provides test utilities for the slide language server.

use tower_lsp::lsp_types::*;

pub fn range_of(subtext: &str, text: &str) -> Range {
    let span_start = text
        .match_indices(subtext)
        .next()
        .expect("Subtext not found.")
        .0;
    let span = (span_start, span_start + subtext.chars().count());
    crate::shims::to_range(&span.into(), text)
}

pub struct DecorationResult {
    pub decorations: Vec<(Range, Option<String>)>,
    pub cursor: Option<Position>,
    pub text: String,
}

/// Given a text like
///   aaa := 1 + 2
///   ~~~
///   1 + 2 + ¦aaa
///            ~~~@[comment two]
/// Extracts the cursor position ¦,
/// all ^^^@[comment] sequences in the pairs (Range, Option<comment>) sequentially,
/// and returns the text without any decorations.
///
/// The returned cursor position always marks the character to right of its original location,
/// for example in "ab¦cde",
/// the hover position would be over "c".
///
/// Decorations can have the following features:
///   ^^^                       - covers just a span
///   ^^^@[<some comment>]      - covers a span with the comment "<some comment>"
///   ^^^@[line one<|>line two] - replaces <|> with newlines;
///                               i.e. this is equivalent to "line one\nline two"
pub fn process_decorations(text: &str) -> DecorationResult {
    let mut lines = text
        .split_terminator('\n')
        .map(String::from)
        .collect::<Vec<_>>();
    let re_decorator = regex::Regex::new(r"(?P<span>~+)(@\[(?P<comment>.*?)\])?").unwrap();
    let replacements = &[("<|>", "\n")];

    let mut cleaned_lines = vec![];
    let mut decorations = vec![];
    let mut cursor = None;

    for i in 0..lines.len() {
        let line = lines[i].clone();
        // First, check for cursors.
        let cursor_splits = line.split('¦').collect::<Vec<_>>();
        if cursor_splits.len() > 1 {
            if cursor_splits.len() > 2 || cursor.is_some() {
                panic!(r#"Cannot have more than one hover cursor, saw "{}""#, line);
            } else {
                let line = cleaned_lines.len();
                let column = cursor_splits[0].len();
                cursor = Some(Position::new(line as u64, column as u64));
                // Since we found a cursor and are about to remove it,
                // we need to re-adjust the next line if it has any decorations after the position
                // of the cursor.
                // For example in
                //   1 + 2 + ¦aaa
                //            ~~~@[comment two]
                // we need to shift the second line back one space after the column where ¦ is on
                // this current line.
                if let Some(next_line) = lines.get(i + 1) {
                    if re_decorator.is_match(next_line) && column + 1 < next_line.len() {
                        let (l, r) = next_line.split_at(column + 1);
                        lines[i + 1] = [&l[0..l.len() - 1], r].join("");
                    }
                }
                cleaned_lines.push(cursor_splits.join(""));
                continue;
            }
        }

        // Next, check for decorations.
        let local_decorations: Vec<_> = re_decorator
            .captures_iter(&line)
            .map(|decor| {
                let span = decor.name("span").unwrap();
                let line = cleaned_lines.len() - 1;
                let range = Range::new(
                    Position::new(line as u64, span.start() as u64),
                    Position::new(line as u64, span.end() as u64),
                );
                let comment = decor.name("comment").map(|c| {
                    replacements
                        .iter()
                        .fold(c.as_str().to_string(), |s, &(replace, with)| {
                            s.split(replace).collect::<Vec<_>>().join(with)
                        })
                });

                (range, comment)
            })
            .collect();

        if local_decorations.is_empty() {
            // No decorations, this is just a normal line.
            cleaned_lines.push(line.to_string());
        }
        decorations.extend(local_decorations.into_iter());
    }

    DecorationResult {
        decorations,
        cursor,
        text: cleaned_lines.join("\n"),
    }
}

#[cfg(test)]
mod test {
    // Testing test utils is... not the best, but it certainly makes writing other tests easier.
    // Even if it means we have some complicated utils.

    use tower_lsp::lsp_types::*;

    #[test]
    fn process_decorations() {
        let text = r"
        aaa := 1 + 2
        ~~~
        1 + 2 + ¦aaa
                 ~~~@[comment two]
        ";
        let processed_text = r"
        aaa := 1 + 2
        1 + 2 + aaa
        ";

        let super::DecorationResult {
            decorations,
            cursor,
            text,
        } = super::process_decorations(text);

        assert_eq!(text, processed_text);
        assert_eq!(cursor, Some(Position::new(2, 16)));
        assert_eq!(
            decorations,
            vec![
                (Range::new(Position::new(1, 8), Position::new(1, 11)), None),
                (
                    Range::new(Position::new(2, 16), Position::new(2, 19)),
                    Some("comment two".to_string())
                )
            ]
        );
    }
}
