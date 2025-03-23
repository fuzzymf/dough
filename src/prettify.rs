extern crate lazy_static;
use crate::utils::{
    calculate_length_of_line, calculate_length_of_longest_line, check_if_text_is_right_aligned,
    store_colors, strip_ansi_codes,
};

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::{collections::HashMap, str};

use colored::*;
use markdown::mdast::{self};
use regex::Regex;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use unicode_width::UnicodeWidthStr;

use lazy_static::lazy_static;

lazy_static! {
    /// Style map is used to store the styles associated with a particular markdown element
    /// The styles are stored as a HashMap with the key being the name of the markdown element
    /// and the value being the style associated with it.
    /// The styles are stored as strings and are converted to the appropriate type when needed.
    /// The styles are stored in the global STYLES variable, which is a Mutex<HashMap<String, String>>
    /// This also stores the upper and lower bounds of the content, which is used for vertical alignment
    static ref STYLES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());


    /// This is used to store the colors associated with each line of the content
    /// Using a static variable to store the colors ensures that the the colors are cached and the code does not recompute the colors
    static ref PS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref TS: ThemeSet = ThemeSet::load_defaults();

    /// This is used to store the codes in the file
    /// The codes are stored in sequence of their appearance in the file
    /// The codes are stored in the global CODES variable, which is a BtreeMap<String, String>
    ///     where the key is the index of the order of appearance of the code and the value is a vetor of language and code
    static ref CODES: Mutex<BTreeMap<usize, (String, String)>> = Mutex::new(BTreeMap::new());


}

/// This function is used to join the children of a particular mdast node
/// The join_fn is used to decorate the text before joining it

fn join_children_with(
    join_fn: fn(String) -> String,
    depth: usize,
    children: Vec<mdast::Node>,
) -> String {
    let mut result = String::default();
    for child in children {
        if let Some(text) = visit_md_node(child, depth) {
            let decorated_text = join_fn(text);
            result.push_str(&decorated_text);
        }
    }
    return result;
}

/// This function is used to join the children of a particular mdast node

fn join_children(children: Vec<mdast::Node>, depth: usize) -> String {
    return join_children_with(|x| x, depth, children);
}

/// Recursively visit the mdast tree and return a string
/// The string is decorated with the appropriate styles
/// The styles are fetched from the global STYLES variable
fn visit_md_node(node: mdast::Node, depth: usize) -> Option<String> {
    let style_map = STYLES.lock().unwrap();
    let styles = style_map.clone();
    drop(style_map);

    match node {
        mdast::Node::Root(root) => {
            let mut result = String::default();
            result.push_str(&join_children(root.children, depth));
            result.push('\n');
            Some(result)
        }

        mdast::Node::Paragraph(paragraph) => {
            let text_start = &join_children(paragraph.children.clone(), depth);
            let mut result = String::from("\n");

            // Regex is used to match the strikethrough text
            // This strikethrough text is a child of the paragraph node

            let re = Regex::new(r"~~(.*?)~~").unwrap();
            if re.is_match(text_start) {
                for cap in re.captures_iter(text_start) {
                    let matched_text = &cap[1];
                    let strikethrough_text = matched_text
                        .chars()
                        .map(|c| format!("{}{}", c, '\u{0336}'))
                        .collect::<String>();
                    let text_to_replace = format!("~~{}~~", matched_text);
                    let replaced_text = text_start.replace(&text_to_replace, &strikethrough_text);
                    result.push_str(&replaced_text);
                }
            } else {
                // the depth is used to calculate the indentation
                // Used in nested lists/ blockquotes

                let item_text = " ".white().on_black().to_string().repeat(depth);
                result.push_str(&item_text);
                result.push_str(text_start);
            }

            result.push('\n');
            Some(result)
        }

        mdast::Node::Text(text) => {
            let mut result = String::default();

            // Used to match the strikethrough text
            let re = Regex::new(r"~~(.*?)~~").unwrap();
            if re.is_match(&text.value) {
                for cap in re.captures_iter(&text.value) {
                    let matched_text = &cap[1];
                    let strikethrough_text = matched_text
                        .chars()
                        .map(|c| format!("{}{}", c, '\u{0336}'))
                        .collect::<String>();
                    let text_to_replace = format!("~~{}~~", matched_text);
                    let replaced_text = text.value.replace(&text_to_replace, &strikethrough_text);
                    result.push_str(&replaced_text);
                }
            } else {
                result.push_str(&text.value);
            }
            Some(result)
        }

        mdast::Node::Heading(heading) => {
            let level = heading.depth;
            let mut result = String::from("\n");

            let color: &str;
            let mut item_text = String::new();

            match level {
                1 => {
                    color = styles.get("h1").map(|s| s.as_str()).unwrap_or("red");
                    item_text.push_str(
                        &format!("█ {}", join_children(heading.children, depth))
                            .color(color)
                            .to_string(),
                    );
                    result.push_str(&item_text);
                }
                2 => {
                    color = styles.get("h2").map(|s| s.as_str()).unwrap_or("yellow");
                    item_text.push_str(
                        &format!("██ {}", join_children(heading.children, depth))
                            .color(color)
                            .to_string(),
                    );
                    result.push_str(&item_text);
                }
                3 => {
                    color = styles.get("h3").map(|s| s.as_str()).unwrap_or("green");
                    item_text.push_str(
                        &format!("███ {}", join_children(heading.children, depth))
                            .color(color)
                            .to_string(),
                    );
                    result.push_str(&item_text);
                }
                4 => {
                    color = styles.get("h4").map(|s| s.as_str()).unwrap_or("blue");
                    item_text.push_str(
                        &format!("████ {}", join_children(heading.children, depth))
                            .color(color)
                            .to_string(),
                    );
                    result.push_str(&item_text);
                }
                5 => {
                    color = styles.get("h5").map(|s| s.as_str()).unwrap_or("magenta");
                    item_text.push_str(
                        &format!("█████ {}", join_children(heading.children, depth))
                            .color(color)
                            .to_string(),
                    );
                    result.push_str(&item_text);
                }

                6 => {
                    color = styles.get("h6").map(|s| s.as_str()).unwrap_or("cyan");
                    item_text.push_str(
                        &format!("██████ {}", join_children(heading.children, depth))
                            .color(color)
                            .to_string(),
                    );
                    result.push_str(&item_text);
                }
                _ => result.push_str(&join_children(heading.children, depth)),
            }
            result.push('\n');
            Some(result)
        }

        mdast::Node::InlineCode(inline_code) => {
            let text = inline_code.value;

            let mut result = String::from("`").replace("`", "");

            let color: &str = styles
                .get("inline_code")
                .map(|s| s.as_str())
                .unwrap_or("red");

            result.push_str("\x1b[0m");
            result.push_str(&text.color(color).to_string());
            result.push_str("`".replace("`", "").as_str());

            Some(result)
        }

        mdast::Node::Code(code) => {
            let language = code.lang.unwrap_or("plaintext".to_string());

            // Store the codes in the file in the global CODES variable
            // The codes are stored in the order of their appearance in the file
            // The specifics of the syntax highlighting are stored in the global STYLES variable from the style.yml file

            let mut codes = CODES.lock().unwrap();

            let last_index = codes.len();
            codes.insert(last_index + 1, (language.clone(), code.value.to_string()));
            drop(codes);

            let syntax_theme = styles
                .get("syntax_theme")
                .map(|s| s.as_str())
                .unwrap_or("base16-ocean.dark")
                .to_string();
            let syntax_highlighting = styles
                .get("syntax_highlighting")
                .map(|s| s.as_str())
                .unwrap_or("true");

            let include_background_color: bool = match styles
                .get("syntax_bg")
                .map(|s| s.as_str())
                .unwrap_or("false")
            {
                "true" | "True" => true,
                _ => false,
            };

            let mut result = String::from("```\n").replace("```", "");
            if syntax_highlighting == "true" {
                let mut highlighted_code = syntax_highlighter(
                    &language,
                    code.value.to_string(),
                    syntax_theme,
                    include_background_color,
                );

                highlighted_code = highlighted_code
                    .lines()
                    .map(|line| format!("{}", line))
                    .collect::<Vec<String>>()
                    .join("\n");
                result.push_str(&highlighted_code.to_string());
            } else {
                // A tab is replaced by 4 spaces to ensure uniform indentation across different terminals and different widths
                let escaped = code.value.replace("\t", "    ");
                result.push_str(&&escaped.to_string());
            }
            result.push_str("\n```\n".replace("```", "").as_str());
            Some(result)
        }

        mdast::Node::Emphasis(emphasis) => Some(join_children_with(
            |s| s.italic().to_string(),
            depth,
            emphasis.children,
        )),

        mdast::Node::Strong(strong) => Some(join_children_with(
            |s| s.bold().to_string(),
            depth,
            strong.children,
        )),

        mdast::Node::Link(link) => {
            let color_url = styles
                .get("link_url")
                .map(|s| s.as_str())
                .unwrap_or("green");
            let color_text = styles
                .get("link_text")
                .map(|s| s.as_str())
                .unwrap_or("blue");

            let mut result = String::from("[");
            result = result.replace("[", "");

            result.push_str(
                &join_children(link.children, depth)
                    .color(color_text)
                    .to_string(),
            );

            result.push_str(" - ");
            result.push_str(&link.url.color(color_url).to_string());

            Some(result)
        }

        mdast::Node::ThematicBreak(_) => Some("\n---\n".to_string()),

        mdast::Node::Blockquote(blockquote) => {
            let default_blockquote_color = "black on white".to_string();

            let color = styles
                .get("blockquote")
                .map(|s| s.as_str())
                .unwrap_or(&default_blockquote_color);

            let colors: Vec<&str> = color.split(" on ").collect();
            let foreground_color = colors[0];
            let background_color = colors[1];

            let mut result = String::default();
            result.push_str(
                &join_children(blockquote.children, depth + 1)
                    .color(foreground_color)
                    .on_color(background_color)
                    .to_string(),
            );
            result.push('\n');

            Some(result)
        }

        mdast::Node::List(list) => {
            let bullet_color: &str = match list.ordered {
                true => styles
                    .get("ordered_list_bullet")
                    .map(|s| s.as_str())
                    .unwrap_or("green"),
                false => styles
                    .get("unordered_list_bullet")
                    .map(|s| s.as_str())
                    .unwrap_or("green"),
            };

            let text_color: &str = match list.ordered {
                true => styles
                    .get("ordered_list")
                    .map(|s| s.as_str())
                    .unwrap_or("blue"),
                false => styles
                    .get("unordered_list")
                    .map(|s| s.as_str())
                    .unwrap_or("blue"),
            };

            let mut result = String::default();
            let mut item_number = list.start.unwrap_or(1);
            result.push_str("\n");

            for item in list.children {
                let mut item_text = "  ".repeat((depth) as usize);
                if list.ordered {
                    item_text.push_str(
                        &format!(" {}. ", item_number)
                            .color(bullet_color)
                            .to_string(),
                    );
                } else {
                    // depth is used to calculate the indentation

                    let sep = match depth {
                        0 => " • ",
                        1 => " · ",
                        2 => " * ",
                        3 => " - ",
                        _ => " • ",
                    };
                    item_text.push_str(sep.to_string().color(bullet_color).to_string().as_str());
                }

                if let mdast::Node::ListItem(list_item) = item {
                    for child in list_item.children {
                        if let mdast::Node::Paragraph(paragraph) = child {
                            item_text.push_str(&join_children(paragraph.children, depth + 1));
                        } else {
                            item_text.push_str(&join_children(vec![child], depth + 1));
                        }
                    }
                }

                item_text.push('\n');
                result.push_str(&item_text.color(text_color).to_string());
                item_number += 1;
            }

            result.push('\n');

            Some(result)
        }

        mdast::Node::Break(mdast::Break { position: _ }) => Some("\n".to_string()),

        mdast::Node::Delete(delete) => Some(join_children_with(
            |s| s.strikethrough().to_string(),
            depth,
            delete.children,
        )),

        mdast::Node::Definition(definition) => {
            let color = styles
                .get("definition")
                .map(|s| s.as_str())
                .unwrap_or("green");

            let mut result = String::from("[");
            result.push_str(&definition.identifier.color(color).to_string());
            result.push_str("]: ");
            result.push_str(&definition.url.color(color).to_string());
            result.push_str(" ");
            result.push_str(&definition.title?.color(color).to_string());
            Some(result)
        }

        _ => None,
    }
}

/// This function is used to draw a margin around the content based on the flag set in the style map
/// The flag is set to true by default

pub fn draw_box(content: &str, line_color_map: &HashMap<usize, String>) -> String {
    let lines: Vec<&str> = content.split('\n').collect();
    let lines_clone = lines.clone();

    // Calculate the length of the longest line
    let max_length = lines_clone
        .iter()
        .map(|s| {
            let s = strip_ansi_codes(s).replace("̶", "");
            UnicodeWidthStr::width(s.as_str())
        })
        .max()
        .unwrap_or(0);

    // Create a horizontal border based on the length of the longest line
    let horizontal_border: String = "─".repeat(max_length + 4); // 2 for box corners and sides
    let mut boxed_content = format!("┌{}┐\n", strip_ansi_codes(&horizontal_border)); // top border

    for (i, line) in lines.iter().enumerate() {
        let original_color = match line_color_map.get(&i) {
            Some(_color) => "\x1B[0m",
            None => "\x1B[0m",
        };
        // Remove the strikethrough character from the line
        // These characters add extra length to the line

        let mut free_line = line.replace("̶", "");
        free_line = free_line.replace('\t', " ");

        // Calculate the number of spaces to be added to the end of the line based on the line free of strikethrough characters
        let padding_length =
            max_length - UnicodeWidthStr::width(strip_ansi_codes(&free_line).as_str());
        let padding = " ".repeat(padding_length);

        let formatted_line = String::from(*line);

        boxed_content.push_str(&format!(
            "│  {}{}{}{}{}  │\n",
            "\x1B[0m", original_color, formatted_line, "\x1B[0m", padding
        )); // content with side borders
    }

    boxed_content.push_str(&format!("└{}┘\n", strip_ansi_codes(&horizontal_border))); // bottom border

    boxed_content
}

/// This function is used to align the content vertically based on the flag set in the style map
/// The flag is set to true by default
pub fn align_vertical(
    mut prettified: String,
    style_map: &HashMap<String, String>,
    height: u16,
    upper_bound: &mut u32,
    lower_bound: &mut u32,
) -> String {
    let blank_lines;

    if style_map.get("vertical_alignment").unwrap() == "false" {
        blank_lines = 0;
    } else {
        if height > prettified.lines().count() as u16 {
            // If height is greater than the number of lines, add blank lines at the beginning and end
            // The number of blank lines is calculated by subtracting the number of lines from the height
            blank_lines = (height - prettified.lines().count() as u16) as u32 / 2;
        } else {
            blank_lines = 0;
        }
    }
    let mut new_prettified = String::new();
    // Add blank lines at the end and beginning
    if blank_lines > 2 {
        for _ in 0..blank_lines - 2 {
            new_prettified.push('\n');
            prettified.push('\n');
        }
    }
    new_prettified.push('\n');
    new_prettified.push_str(&prettified);
    prettified = new_prettified;

    // The upper and lower bounds are updated to reflect the changes
    *upper_bound += blank_lines;
    *lower_bound += blank_lines;

    return prettified;
}

/// This function is used to align the content horizontally based on the flag set in the style map
/// The flag is set to true by default
///
pub fn align_horizontal(
    prettified: String,
    style_map: &HashMap<String, String>,
    width: u16,
    line_color_map: HashMap<usize, String>,
    right_alignment: bool,
) -> String {
    let blank_chars;
    let spaces = if right_alignment { false } else { true };
    let longest_line = calculate_length_of_longest_line(&prettified, spaces);

    if style_map.get("horizontal_alignment").unwrap() == "false" {
        blank_chars = 0;
    } else {
        if width > longest_line as u16 {
            // If width is greater than the length of the longest line, add blank characters at the beginning
            // The number of blank characters is calculated by subtracting the length of the longest line from the width
            blank_chars = (width - longest_line as u16) as usize / 2;
        } else {
            blank_chars = 0;
        }
    }

    let mut new_prettified = String::new();

    if blank_chars > 0 {
        // for each line, add blank_chars spaces at the beginning
        let reset_colored_line_ref: &str = "\x1B[0m";
        for (i, line) in prettified.lines().enumerate() {
            let original_color = match line_color_map.get(&i) {
                Some(color) => color,
                None => reset_colored_line_ref,
            };
            let new_line = format!(
                "{}{}{}{}",
                " ".repeat(blank_chars),
                original_color,
                line,
                "\x1B[0m"
            );
            new_prettified.push_str(&new_line);
            new_prettified.push('\n'); // Add newline after each line
        }
        return new_prettified; // Return the modified string
    }

    return prettified; // Return the original string if no alignment needed
}

/// This function is used to align the content based on the alignment flag set in the markdown text
/// The alignment flag is set using the following syntax:
/// $[clr]$ -> center, left, right alignment respectively
/// This is used for text alignment within the content

pub fn align_custom(
    mut prettified: String,
    highlight_line_num: u32,
    style_map: &HashMap<String, String>,
) -> String {
    // calculate the length of the longest line
    let longest_line = calculate_length_of_longest_line(&prettified, true);

    let mut new_prettified = String::new();

    // update the thematic break lines to match the length of the longest line
    let mut content_lines: Vec<String> = prettified.lines().map(|s| s.to_string()).collect();
    for line in content_lines.iter_mut() {
        if line == "---" || line == "***" || line == "__i_" {
            let mut new_line = String::from(line.replace("---", ""));
            for _ in 0..longest_line {
                new_line.push_str("-");
            }
            *line = new_line;
        }
    }

    if highlight_line_num > 0 {
        let default_highlight_color = "black on white".to_string();
        let highlight_color = style_map
            .get("highlighter")
            .unwrap_or(&default_highlight_color);

        let colors: Vec<&str> = highlight_color.split(" on ").collect();
        let foreground_color = colors[0];
        let background_color = colors[1];

        prettified = content_lines.join("\n");

        let mut lines: Vec<String> = prettified.lines().map(|line| line.to_string()).collect();
        if lines.len() > highlight_line_num as usize {
            let line_num = lines.len() as u32 - highlight_line_num;
            if line_num < lines.len() as u32 {
                let line = lines.get_mut(line_num as usize).unwrap();
                *line = strip_ansi_codes(line).to_string();
                *line = line
                    .color(foreground_color)
                    .on_color(background_color)
                    .to_string();
            }
        }

        prettified = lines.join("\n");
    } else {
        prettified = content_lines.join("\n");
    }

    // the custom alignment is done using the following syntax:
    // $[clr]$ -> center, left, right alignment respectively for a line
    // $[clr]$ -> center, left, right alignment for a block of text
    // $[e]$ -> end block of text

    let mut lines_iter = prettified.lines().peekable();

    while let Some(line) = lines_iter.next() {
        let mut aligned_line = line.to_string();

        // line_re is used to match the alignment flag for a line
        let line_re = regex::Regex::new(r"\$\[([clr])\]\$").unwrap();
        // block_re is used to match the alignment flag for a block of text
        let block_re = regex::Regex::new(r"\$\[([clr])\]").unwrap();
        // end_block_re is used to match the end block of text
        let end_block_re = regex::Regex::new(r"\$\[e\]").unwrap();

        if let Some(captures) = line_re.captures(&aligned_line) {
            let alignment = captures.get(1).unwrap().as_str();
            // replace the alignment flag with an empty string
            let new_line = aligned_line.replace(&captures[0], "");
            let line_length = calculate_length_of_line(&new_line, true);
            match alignment {
                "c" => {
                    let spaces = (longest_line - line_length) / 2;
                    let mut new_line = format!("{}{}{}", "\x1b[0m", " ".repeat(spaces), line);
                    new_line = new_line.replace(&captures[0], "");
                    aligned_line = new_line;
                }
                "r" => {
                    let spaces = longest_line - line_length - 1;
                    let mut new_line = format!("{}{}{}", "\x1b[0m", " ".repeat(spaces), line);
                    new_line = new_line.replace(&captures[0], "");
                    aligned_line = new_line;
                }
                _ => {
                    // Do nothing for "l" alignment
                    aligned_line = aligned_line.replace(&captures[0], "");
                }
            }
            new_prettified.push_str(&aligned_line);
        } else if let Some(captures) = block_re.captures(&aligned_line) {
            let alignment = captures.get(1).unwrap().as_str();

            // iterate and check for the end block of text
            // until the end block of text is found, push the lines into a vector

            let mut block_lines: Vec<&str> = vec![&aligned_line];

            while let Some(&next_line) = lines_iter.peek() {
                if end_block_re.is_match(next_line) {
                    break;
                }
                block_lines.push(lines_iter.next().unwrap());
            }
            lines_iter.next();
            lines_iter.next();

            let mut aligned_block = String::new();

            // align the block of text based on the alignment flag

            for line in block_lines.iter().skip(1) {
                let line_length = calculate_length_of_line(line, true);
                match alignment {
                    "c" => {
                        let spaces = (longest_line - line_length) / 2;
                        let new_line = format!("{}{}{}\n", "\x1b[0m", " ".repeat(spaces), line);
                        aligned_block.push_str(&new_line);
                    }
                    "r" => {
                        let spaces = longest_line - line_length;
                        let new_line = format!("{}{}{}\n", "\x1b[0m", " ".repeat(spaces), line);
                        aligned_block.push_str(&new_line);
                    }
                    _ => {
                        // Do nothing for "l" alignment
                    }
                }
            }
            new_prettified.push_str(format!("{}", aligned_block).as_str());
        } else {
            new_prettified.push_str(format!("{}", aligned_line).as_str());
        }
        new_prettified.push('\n');
    }

    new_prettified
}

/// This function is used to align the entire content based on various flags and markdown text
/// The flags are set in the style map  
/// The flags are as follows:
/// 1. box: true/false
/// 2. horizontal_alignment: true/false
/// 3. vertical_alignment: true/false
/// 4. terminal: warp/normal    

pub fn align_content(
    mut prettified: String,
    style_map: &HashMap<String, String>,
    highlight_line_num: u32,
) -> String {
    let (_width, height) = termion::terminal_size().unwrap();

    // Bounds are used for scrolling
    let mut upper_bound = prettified.lines().count() as u32;
    let mut lower_bound = 0;
    let right_aligned = check_if_text_is_right_aligned(&prettified.clone());

    // Custom text alignment, including highlighting
    prettified = align_custom(prettified, highlight_line_num, style_map);

    // draw a margin around the content based on the flag set in the style map
    if style_map.get("box").unwrap() == "true" {
        // A HashMap is used to store the colors for each line
        let content_lines: Vec<String> = prettified.lines().map(|s| s.to_string()).collect();
        let line_color_map = store_colors(&content_lines);
        upper_bound += 2;
        prettified = draw_box(&prettified, &line_color_map);
    }

    // align the content horizontally based on the flag set in the style map
    if style_map.get("horizontal_alignment").unwrap() == "true" {
        let content_lines: Vec<String> = prettified.lines().map(|s| s.to_string()).collect();
        let line_color_map = store_colors(&content_lines);

        prettified = align_horizontal(prettified, style_map, _width, line_color_map, right_aligned);
    }

    // align the content vertically based on the flag set in the style map
    if style_map.get("vertical_alignment").unwrap() == "true" {
        prettified = align_vertical(
            prettified,
            style_map,
            height,
            &mut upper_bound,
            &mut lower_bound,
        );
    }
    prettified.push('\n');

    let mut global_styles = STYLES.lock().unwrap();

    global_styles.insert("upper_bound".to_string(), upper_bound.to_string());
    global_styles.insert("lower_bound".to_string(), lower_bound.to_string());
    drop(global_styles);

    return prettified;
}

/// This function is used to syntax highlight the code using the syntect crate
/// The syntax highlighting is done based on the language and theme set in the style map
/// The syntax highlighting is done using the following syntax:
/// ```language
/// code
/// ```
/// The syntax highlighting is done using the following steps:
/// 1. Load the syntaxes and themes
/// 2. Create a highlighter using the syntax and theme
/// 3. Highlight each line
/// 4. Return the highlighted code and store it in a static variable to optimize performance
/// The highlighted code is then used to decorate the content

pub fn syntax_highlighter(language: &str, code_section: String, theme: String, bg: bool) -> String {
    // Load the syntaxes and themes
    let syntax = PS
        .find_syntax_by_extension(language)
        .unwrap_or(PS.find_syntax_plain_text());
    let theme = &TS.themes[&theme];

    // Create a highlighter
    let mut h = HighlightLines::new(syntax, theme);

    // Highlight each line
    let mut highlighted = String::new();
    for line in LinesWithEndings::from(&code_section) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &PS);
        let mut escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], bg);
        escaped = escaped.replace("\t", "    ");
        highlighted.push_str(&escaped);
    }

    highlighted
}

/// This is used to get the upper and lower bounds of the content
/// The upper and lower bounds are used for vertical alignment
/// The upper bound is the number of blank lines at the beginning of the content
/// The lower bound is the number of blank lines at the end of the content
/// The bounds are stored in the global STYLES variable and are used fort scrolling
pub fn get_bounds() -> (u32, u32) {
    let global_styles = STYLES.lock().unwrap();

    let upper_bound = global_styles
        .get("upper_bound")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let lower_bound = global_styles
        .get("lower_bound")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    drop(global_styles);

    return (upper_bound, lower_bound);
}

/// This function is used to get the code from the global CODES variable
/// The index is used to fetch the code from the global CODES variable
/// The code is returned as a tuple of language and code

pub fn get_code(index: usize) -> Result<(String, String), Box<dyn std::error::Error>> {
    let codes = CODES.lock().unwrap();

    if let Some(code) = codes.get(&index) {
        return Ok((code.0.to_string(), code.1.to_string()));
    }

    Err(format!("Code with index {} not found", index).into())
}

/// This function is used to prettify the markdown text
/// The markdown text is parsed using the markdown crate
/// The parsed mdast tree is then visited and converted to a string
/// The string is then decorated with the appropriate styles
/// The styles are fetched from the global STYLES variable

pub fn prettify(
    md_text: &str,
    style_map: &HashMap<String, String>,
    highlight_line_num: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let map = style_map.clone();
    let mut global_styles = STYLES.lock().unwrap();
    *global_styles = map;
    drop(global_styles);

    let mut codes = CODES.lock().unwrap();

    *codes = BTreeMap::new();
    drop(codes);

    let mut lines = md_text.lines();
    // let mut front_matter = Vec::new();

    let first_line = lines.next();

    let md_text = if let Some(line) = first_line {
        // If there are lines left, join them and add a newline at the end
        std::iter::once(line)
            .chain(lines)
            .collect::<Vec<&str>>()
            .join("\n")
            + "\n"
    } else {
        // If there are no lines left, return an empty string
        String::new()
    };

    let parsed = markdown::to_mdast(&md_text, &markdown::ParseOptions::default());
    let mut prettified = String::new();

    match parsed {
        Err(err) => return Err(format!("Error parsing markdown: {}", err).into()),
        Ok(node) => {
            let result = visit_md_node(node, 0);
            if let Some(text) = result {
                prettified.push_str(&text);
            }
        }
    }
    //remove the last line if it is an empty line
    // this to ensure that the content is not padded with an extra line and improve the multiple rendering methods; the extra line is not highlighted or styled
    if prettified.ends_with('\n') {
        prettified.pop();
    }

    return Ok(align_content(prettified, style_map, highlight_line_num));
}
