use crate::app::app::App;
use ratatui::backend::Backend;
use ratatui::widgets::Paragraph;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

pub fn render_contents<B: Backend>(f: &mut Frame<B>, app: &mut App, chunks: &[Rect]) {
    let contents_block = Block::default().borders(Borders::ALL).title("Contents");
    f.render_widget(contents_block, chunks[0]);

    let selected_file = match app.files.state.selected() {
        Some(i) => match app.files.items.get(i) {
            Some(item) => &item.0,
            None => "",
        },
        None => "",
    };

    let mut content = String::new();
    let max_lines = chunks[0].height as usize - 2;

    if !selected_file.is_empty() {
        let metadata = match std::fs::metadata(selected_file) {
            Ok(metadata) => metadata,
            Err(err) => {
                println!("Error getting metadata for file: {}", err);
                return;
            }
        };

        if !metadata.is_file() {
            println!("Not a regular file: {}", selected_file);
            return;
        }

        let file = match File::open(selected_file) {
            Ok(file) => file,
            Err(err) => {
                println!("Error opening file: {}", err);
                return;
            }
        };

        let reader = BufReader::new(file);
        // TODO: check for reading binary files
        for (num, line) in reader.lines().enumerate() {
            if num >= max_lines {
                break;
            }

            match line {
                Ok(line) => {
                    content.push_str(&line);
                    content.push('\n');
                }
                Err(err) => {
                    println!("Error reading line: {}", err);
                    return;
                }
            }
        }
    }

    let items = List::new(vec![ListItem::new(content)])
        .block(Block::default().borders(Borders::ALL).title("Preview"));

    f.render_stateful_widget(items, chunks[0], &mut app.files.state);

    if selected_file.is_empty() {
        let placeholder = Paragraph::new("No file selected")
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("Contents"));
        f.render_widget(placeholder, chunks[0]);
    }
}
