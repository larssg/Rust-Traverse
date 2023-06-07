use crate::app::App;
use crate::ui::pane::selected_pane_content;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::Backend;
use ratatui::layout::Alignment;
use ratatui::widgets::{Clear, Paragraph};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Terminal,
    text::Spans,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use std::fs::File;
use std::io::BufRead;
use std::io::{self, BufReader};
use std::time::Duration;

use super::pane::get_pwd;
use super::run_app::run_app;

pub fn init() -> Result<()> {
    enable_raw_mode()?;

    let stdout = io::stdout();
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture,)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;

    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("{}", e);
    }

    Ok(())
}

pub fn render<B: Backend>(f: &mut Frame<B>, app: &mut App, input: &mut String) {
    let cur_dir = app.cur_dir.clone();
    let cur_du = app.cur_du.clone();

    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
        .split(chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(45),
            Constraint::Percentage(10),
        ])
        .split(chunks[1]);

    let bottom_chunks = bottom_chunks(f);

    render_contents(f, app, &left_chunks);
    render_files(f, app, &[right_chunks[0]]);
    render_dirs(f, app, &[right_chunks[1]]);
    render_details(f, app, &bottom_chunks, cur_dir, cur_du);
    render_input(f, app, size, input);
    render_navigator(f, app, size, input);
    render_fzf(f, app, size);
    render_bookmark(f, app, size);
}

fn bottom_chunks<B: Backend>(f: &mut Frame<B>) -> Vec<Rect> {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(94), Constraint::Percentage(6)].as_ref())
        .split(size);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(chunks[1]);

    (bottom_chunks).to_vec()
}

fn render_files<B: Backend>(f: &mut Frame<B>, app: &mut App, chunks: &[Rect]) {
    let files_block = Block::default()
        .borders(Borders::ALL)
        .title("Files")
        .title_alignment(Alignment::Center);
    f.render_widget(files_block, chunks[0]);

    let files = app
        .files
        .items
        .iter()
        .map(|i| ListItem::new(i.0.clone()))
        .collect::<Vec<ListItem>>();

    let items = List::new(files)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Files")
                .title_alignment(Alignment::Center),
        )
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );

    if app.files.items.len() == 0 {
        let empty = vec![ListItem::new("No files in this directory")];
        let empty_list = List::new(empty)
            .block(Block::default().borders(Borders::ALL).title("Files"))
            .highlight_symbol("> ")
            .highlight_style(
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_stateful_widget(empty_list, chunks[0], &mut app.files.state);
        return;
    }
    // else {
    //     // add to config file to hide hidden files
    //     let temp = app.files.items.clone();
    //     for file in temp {
    //         if file.0.starts_with(".") {
    //             let index = app.files.items.iter().position(|x| x.0 == file.0).unwrap();
    //             app.files.items.remove(index);
    //         }
    //     }
    // }

    f.render_stateful_widget(items, chunks[0], &mut app.files.state);

    if app.files.state.selected().is_some() {
        let files_block = Block::default()
            .borders(Borders::ALL)
            .title("Files")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::LightBlue));
        f.render_widget(files_block, chunks[0]);
    } else {
        let files_block = Block::default()
            .borders(Borders::ALL)
            .title("Files")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::White));
        f.render_widget(files_block, chunks[0]);
    }
}

fn render_dirs<B: Backend>(f: &mut Frame<B>, app: &mut App, chunks: &[Rect]) {
    app.cur_dir = get_pwd();

    let dirs_block = Block::default()
        .borders(Borders::ALL)
        .title("Directories")
        .title_alignment(Alignment::Center);
    f.render_widget(dirs_block, chunks[0]);

    let dirs = app
        .dirs
        .items
        .iter()
        .map(|i| ListItem::new(i.0.clone()))
        .collect::<Vec<ListItem>>();

    let items = List::new(dirs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Directories")
                .title_alignment(Alignment::Center),
        )
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(items, chunks[0], &mut app.dirs.state);

    if app.dirs.state.selected().is_some() {
        let dirs_block = Block::default()
            .borders(Borders::ALL)
            .title("Directories")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::LightBlue));
        f.render_widget(dirs_block, chunks[0]);
    } else {
        let dirs_block = Block::default()
            .borders(Borders::ALL)
            .title("Directories")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::White));
        f.render_widget(dirs_block, chunks[0]);
    }
}

fn render_details<B: Backend>(
    f: &mut Frame<B>,
    app: &mut App,
    chunks: &[Rect],
    cur_dir: String,
    cur_du: String,
) {
    let details_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    let selected_file = match app.files.state.selected() {
        Some(i) => match app.files.items.get(i) {
            Some(item) => &item.0,
            None => "",
        },
        None => "",
    };

    let selected_dir = match app.dirs.state.selected() {
        Some(i) => &app.dirs.items[i].0,
        None => "",
    };

    let selected_item = if !selected_file.is_empty() {
        selected_pane_content(&selected_file.to_string())
    } else if !selected_dir.is_empty() {
        selected_pane_content(&selected_dir.to_string())
    } else {
        vec![ListItem::new(Spans::from("No file selected"))]
    };

    let items = List::new(selected_item).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightYellow))
            .title("Details")
            .title_alignment(Alignment::Left),
    );
    f.render_widget(items, details_chunks[0]);

    let pwd_paragraph = Paragraph::new(cur_dir)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightYellow))
                .title_alignment(Alignment::Center)
                .title("Current Directory"),
        )
        .alignment(Alignment::Center);
    f.render_widget(pwd_paragraph, details_chunks[1]);

    let du_paragraph = Paragraph::new(cur_du)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightYellow))
                .title("Disk Usage")
                .title_alignment(Alignment::Right),
        )
        .alignment(Alignment::Right);
    f.render_widget(du_paragraph, details_chunks[2]);
}

fn render_contents<B: Backend>(f: &mut Frame<B>, app: &mut App, chunks: &[Rect]) {
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
    let mut total_line_count = 0;

    if !selected_file.is_empty() {
        let file = File::open(selected_file).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut line = String::new();

        while buf_reader.read_line(&mut line).unwrap() > 0 {
            total_line_count += 1;

            if total_line_count <= 30 {
                content.push_str(&line);
            }

            line.clear();
        }
    }

    if total_line_count > 30 {
        content.push_str(&format!("\n... {} more lines", total_line_count - 30));
        content.push_str(&format!("\n{} total", total_line_count));
    };

    let items = List::new(vec![ListItem::new(content)])
        .block(Block::default().borders(Borders::ALL).title("Contents"));

    f.render_stateful_widget(items, chunks[0], &mut app.files.state);

    if selected_file.is_empty() {
        let placeholder = Paragraph::new("No file selected")
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("Contents"));
        f.render_widget(placeholder, chunks[0]);
    }
}

fn render_input<B: Backend>(f: &mut Frame<B>, app: &mut App, size: Rect, input: &mut String) {
    if app.show_popup {
        let block = Block::default()
            .title("Name")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        let input_box_width = 30;
        let input_box_height = 3;
        let input_box_x = (size.width - input_box_width) / 4 + 3;
        let input_box_y = (size.height - input_box_height) / 1;

        let area = Rect::new(input_box_x, input_box_y, input_box_width, input_box_height);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let input_box = Paragraph::new(input.clone())
            .style(Style::default())
            .block(
                Block::default()
                    .title("Input")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left);
        f.render_widget(input_box, area);
    }
}

fn render_navigator<B: Backend>(f: &mut Frame<B>, app: &mut App, size: Rect, input: &mut String) {
    if app.show_nav {
        let block = Block::default()
            .title("Navigator")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        let input_box_width = 30;
        let input_box_height = 3;
        let input_box_x = (size.width - input_box_width) / 4 + 3;
        let input_box_y = (size.height - input_box_height) / 1;

        let area = Rect::new(input_box_x, input_box_y, input_box_width, input_box_height);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let input_box = Paragraph::new(input.clone())
            .style(Style::default())
            .block(Block::default().title("Navigator").borders(Borders::ALL))
            .style(
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Left);
        f.render_widget(input_box, area);
    }
}

fn render_fzf<B: Backend>(f: &mut Frame<B>, app: &mut App, size: Rect) {
    if app.show_fzf {
        let block_width = f.size().width / 3;
        let block_height = f.size().height / 3;
        let block_x = (size.width - block_width) / 2;
        let block_y = (size.height - block_height) / 2;

        let area = Rect::new(block_x, block_y, block_width, block_height);

        let results_block = Block::default()
            .style(Style::default().add_modifier(Modifier::BOLD))
            .title("FZF")
            .border_style(
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        f.render_widget(Clear, area);
        f.render_widget(results_block, area);

        let results_text = app
            .fzf_results
            .items
            .iter()
            .map(|i| ListItem::new(i.clone()))
            .collect::<Vec<ListItem>>();

        let results_list = List::new(results_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Results")
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::LightGreen),
            )
            .highlight_symbol("> ");

        let results_list_area =
            Rect::new(block_x + 1, block_y + 1, block_width - 2, block_height - 2);

        f.render_stateful_widget(results_list, results_list_area, &mut app.fzf_results.state);
    }
}

fn render_bookmark<B: Backend>(f: &mut Frame<B>, app: &mut App, size: Rect) {
    if app.show_bookmark {
        let block_width = f.size().width / 3;
        let block_height = f.size().height / 3;
        let block_x = (size.width - block_width) / 2;
        let block_y = (size.height - block_height) / 2;

        let area = Rect::new(block_x, block_y, block_width, block_height);

        let bookmark_block = Block::default()
            .style(Style::default().add_modifier(Modifier::BOLD))
            .title("Bookmarks")
            .border_style(
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        f.render_widget(Clear, area);
        f.render_widget(bookmark_block, area);

        let bookmark_text = app
            .bookmarked_dirs
            .items
            .iter()
            .map(|i| ListItem::new(i.clone()))
            .collect::<Vec<ListItem>>();

        let bookmark_list = List::new(bookmark_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Bookmarks")
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::LightGreen),
            )
            .highlight_symbol("> ");

        let bookmark_list_area =
            Rect::new(block_x + 1, block_y + 1, block_width - 2, block_height - 2);

        f.render_stateful_widget(
            bookmark_list,
            bookmark_list_area,
            &mut app.bookmarked_dirs.state,
        );
    }
}
