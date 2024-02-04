extern crate lazy_static;
use crate::utils::{calculate_length_of_longest_line, store_colors, strip_ansi_codes};

use std::sync::Mutex;
use std::{collections::HashMap, str};

use colored::*;
use markdown::mdast::{self};
use regex::Regex;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use lazy_static::lazy_static;

lazy_static! {
    /// Style map is used to store the styles associated with a particular markdown element
    /// The styles are stored as a HashMap with the key being the name of the markdown element
    /// and the value being the style associated with it.
    /// The styles are stored as strings and are converted to the appropriate type when needed.
    /// The styles are stored in the global STYLES variable, which is a Mutex<HashMap<String, String>>
    /// This also stores the upper and lower bounds of the content, which is used for vertical alignment
    static ref STYLES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());

    static ref PS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref TS: ThemeSet = ThemeSet::load_defaults();
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
///
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
                // currently, blockquotes are indented by 2 spaces

                let item_text = " ".white().on_black().to_string().repeat(depth);
                result.push_str(&item_text);
                result.push_str(text_start);
            }

            result.push('\n');
            Some(result)
        }

        mdast::Node::Text(text) => {
            let mut result = String::default();
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
                .unwrap_or("green");

            result.push_str(&text.color(color).to_string());
            result.push_str("`".replace("`", "").as_str());

            Some(result)
        }

        mdast::Node::Code(code) => {
            let language = code.lang.unwrap_or("plaintext".to_string());
            let color: &str = styles.get("code").map(|s| s.as_str()).unwrap_or("white");
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
                .unwrap_or("true")
            {
                "true" => true,
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
                result.push_str(&highlighted_code.color(color).to_string());
            } else {
                result.push_str(&code.value.color(color).to_string());
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

            if link.url.to_string().contains("http") {
                result.push_str(" :(");
                result.push_str(&link.url.color(color_url).to_string());
                result.push(')');
            }

            Some(result)
        }

        mdast::Node::ThematicBreak(_) => Some("\n---\n".to_string()),

        mdast::Node::BlockQuote(blockquote) => {
            let mut result = String::default();
            result.push_str(
                &join_children(blockquote.children, depth + 1)
                    .on_white()
                    .black()
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
            let leading_spaces = strip_ansi_codes(s)
                .chars()
                .take_while(|c| *c == ' ')
                .count();

            let s = strip_ansi_codes(s).replace("̶", "");
            s.chars().count() + leading_spaces
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
        let padding_length = max_length - strip_ansi_codes(&free_line).chars().count();
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
    if let Some(terminal_style) = style_map.get("terminal") {
        let mut new_prettified = String::new();
        if terminal_style == "warp" {
            // If terminal style is warp, add blank lines at the end and beginning
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
        } else {
            // In all other cases, add blank lines at the beginning
            if blank_lines > 2 {
                for _ in 0..blank_lines - 2 {
                    new_prettified.push('\n');
                }
            } else {
                new_prettified.push('\n');
            }
            new_prettified.push_str(&prettified);
            new_prettified.push('\n');
            prettified = new_prettified;
        }
    }

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
) -> String {
    let blank_chars;
    let longest_line = calculate_length_of_longest_line(&prettified);

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

pub fn align_custom(prettified: String) -> String {
    let longest_line = calculate_length_of_longest_line(&prettified);

    let mut new_prettified = String::new();

    for line in prettified.lines() {
        let mut aligned_line = line.to_string();
        let re = regex::Regex::new(r"\$\[([clr])\]\$").unwrap();
        if let Some(captures) = re.captures(&aligned_line) {
            let alignment = captures.get(1).unwrap().as_str();

            let new_line = aligned_line.replace(&captures[0], "");

            let line_length = strip_ansi_codes(&new_line).len();

            match alignment {
                "c" => {
                    let spaces = (longest_line - line_length) / 2;
                    let mut new_line = format!("{}{}", " ".repeat(spaces), line);
                    new_line = new_line.replace(&captures[0], "");
                    aligned_line = new_line;
                }
                "r" => {
                    let spaces = longest_line - line_length;
                    let mut new_line = format!("{}{}", " ".repeat(spaces), line);
                    new_line = new_line.replace(&captures[0], "");
                    aligned_line = new_line;
                }
                _ => {
                    // Do nothing for "l" alignment
                    aligned_line = aligned_line.replace(&captures[0], "");
                }
            }
        }

        // handle rendering thematic breaks

        if line == "---" || line == "***" || line == "___" {
            let mut new_line = String::from(line.replace("---", ""));
            for _ in 0..longest_line {
                new_line.push_str("-");
            }
            new_prettified.push_str(&new_line);
        } else {
            new_prettified.push_str(&aligned_line);
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

pub fn align_content(mut prettified: String, style_map: &HashMap<String, String>) -> String {
    let (_width, height) = termion::terminal_size().unwrap();

    let mut upper_bound = prettified.lines().count() as u32;
    let mut lower_bound = 0;

    let mut content_lines: Vec<String> = prettified.lines().map(|s| s.to_string()).collect();

    let mut line_color_map = store_colors(&content_lines);

    prettified = align_custom(prettified);

    if style_map.get("box").unwrap() == "true" {
        upper_bound += 4;
        prettified = draw_box(&prettified, &line_color_map);
    }

    if style_map.get("horizontal_alignment").unwrap() == "true" {
        content_lines = prettified.lines().map(|s| s.to_string()).collect();
        line_color_map = store_colors(&content_lines);

        prettified = align_horizontal(prettified, style_map, _width, line_color_map);
    }

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

/// This function is used to prettify the markdown text
/// The markdown text is parsed using the markdown crate
/// The parsed mdast tree is then visited and converted to a string
/// The string is then decorated with the appropriate styles
/// The styles are fetched from the global STYLES variable

pub fn prettify(md_text: &str, style_map: &HashMap<String, String>) -> Result<String, String> {
    let map = style_map.clone();
    let mut global_styles = STYLES.lock().unwrap();
    *global_styles = map;
    drop(global_styles);

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
        Err(err) => return Err(format!("Could not prettify markdown, error: {}", err)),
        Ok(node) => {
            let result = visit_md_node(node, 0);
            if let Some(text) = result {
                prettified.push_str(&text);
            }
        }
    }

    return Ok(align_content(prettified, style_map));
}
