use std::{collections::HashMap, io::{self, stdout, Error, ErrorKind}, path::PathBuf};
//Searches a path for duplicate files
use clap::Parser;
use console_engine::{pixel, rect_style::BorderStyle, screen::Screen};
use crossterm::{event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Rect}, style::{Color, Style, Stylize}, widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState}, Frame, Terminal};
use todooer::Line;
use lazy_static::lazy_static;

#[derive(Parser)]
#[command(name = "todooer")]
#[command(about = "Todo Reports in your terminal", long_about = None)]
//add extended help
#[clap(
    version = "1.0",
    author = "r4z4",
    about = "Retrieve TODO or FIXME like comments from code files to view",
    after_help = "Example: todooer report --path . --pattern .txt"
)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    //create count with path and pattern defaults for both
    Report {
        #[clap(long, default_value = ".")]
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
        // #[clap(long, default_value = "TODO")]
        // word: String,
    },
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn key_row(key: String) -> Row<'static> {
    Row::new(vec![key]).style(Style::new().on_light_green())
}

fn build_rows(data: HashMap<String, Vec<Line>>) -> Vec<Row<'static>>{
    let mut all_rows = Vec::<Row>::new();
    for (k,v) in data {
        let key_row = key_row(k);
        all_rows.push(key_row);
        let mut rows = v.iter().map(|line| {
            Row::new(vec![
                Cell::from(line.line_text.to_string()).style(Style::default().fg(Color::White)),
                Cell::from(line.priority.to_string()).style(Style::default().fg(Color::Green)),
                Cell::from(line.line_num.to_string()).style(Style::default().fg(Color::Yellow)),
                Cell::from(line.row_index.to_string()).style(Style::default().fg(Color::Red)),
            ])
        }).collect::<Vec<Row>>();
        all_rows.append(&mut rows)
    }
    all_rows
}

fn build_table(data: HashMap<String, Vec<Line>>) -> (Table<'static>, u16) {
    let real_rows = build_rows(data);
    let row_count = real_rows.len() as u16;
    // println!("{}", &row_count);
    // let rows = [Row::new(vec!["Comment", "Priority", "Ln #", "Col #"])];
    let widths = [
        Constraint::Length(50),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ];
    let table = Table::new(real_rows, widths)
        .column_spacing(1)
        .style(Style::new().blue())
        .header(
            Row::new(vec!["Comment", "Priority", "Ln #", "Col #"])
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .footer(Row::new(vec!["Updated on Dec 28"])
            .style(Style::new().on_light_green())
            .top_margin(2),
        )
        .block(Block::default().title("Todo Report Table"))
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");
    (table, row_count)
}

fn ui(frame: &mut Frame, res: HashMap<String, Vec<Line>>) {
    let mut table_state = TableState::default();
    let (table, row_count) = build_table(res);
    // frame.render_widget(
    //     Paragraph::new("Hello World!")
    //         .block(Block::default().title("Greeting").borders(Borders::ALL)),
    //     frame.size(),
    // );
    frame.render_stateful_widget(table, Rect::new(0, 0, 80, row_count), &mut table_state);
}

pub fn validate_path_and_pattern(path: &PathBuf, pattern: &str) -> bool {
    let path_str = path.as_path().to_str().unwrap();
    if !todooer::COMM_SYM.contains_key(pattern) {
        println!("Path specified not a supported file extension")
    }
    true
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Report { path, pattern }) => {
            //count files matching a pattern
            println!("Counting files in {} matching {}", path, pattern);
            let path = PathBuf::from(path);
            let valid = validate_path_and_pattern(&path, &pattern);
            let res = todooer::par_examine_dir(&path, &pattern);
            let unw = res.unwrap();
            enable_raw_mode()?;
            stdout().execute(EnterAlternateScreen)?;
            let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
            
            let mut should_quit = false;
            while !should_quit {
                let data = unw.clone();
                terminal.draw(|frame| {
                    ui(frame, data)
                })?;
                should_quit = handle_events()?;
            }
        
            disable_raw_mode()?;
            stdout().execute(LeaveAlternateScreen)?;
            Ok(())


            // let mut scr = Screen::new(101,31);

            // let mutex = todooer::examine_dir(&path, &pattern);
            // let data = mutex.lock().unwrap();
            // println!("{:?}", data);

            // draw some shapes and prints some text
            // scr.rect_border(0,0, 100,30, BorderStyle::new_light());
            // scr.fill_circle(5,5, 3, pixel::pxl('*'));
            // scr.print(40,5, "Report of Todos");
            // scr.fill_circle(90,5, 3, pixel::pxl('*'));
            // let _ = res.unwrap().iter().map(|(k,v)| {
            //     scr.print(20,1, k);
            //     let mut combo = Vec::<String>::new();
            //     let _ =  v.iter().map(|line| {
            //         combo.push(line.line_text.clone());
            //     });
            //     scr.print(20,4, &combo.join(":"));
            //     // scr.print(11,4, &line.line_text);
            //     // scr.print(12,4, &line.line_num.to_string());
            //     // scr.print(13,4, &line.priority.to_string());
            //     // scr.print(14,4, &line.filename);
            // });
            // for (key, val) in res.unwrap() {
            //     scr.print(20,15, &key);
            //     let mut combo = Vec::<String>::new();
            //     for line in val {
            //         combo.push(line.line_text.clone().trim().to_string());
            //     }
            //     scr.print(2,15, &combo.join("\n"));
            //     // scr.print(11,4, &line.line_text);
            //     // scr.print(12,4, &line.line_num.to_string());
            //     // scr.print(13,4, &line.priority.to_string());
            //     // scr.print(14,4, &line.filename);
            // }
            // // print the screen to the terminal
            // scr.draw();
        }

        None => {
            println!("No command given");
            Err(Error::new(ErrorKind::Other, "oh no!"))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_file() {
        let path = PathBuf::from("./src/main.rs");
        let pattern = ".rs";
        let lines = todooer::walk_file_for_lines(&path, pattern).unwrap();
        assert!(lines.len() > 0)
    }

    // #[test]
    // fn test_examine_dir() {
    //     let pattern = ".rs";
    //     let mutex = todooer::examine_dir(&PathBuf::from("./src"), pattern);
    //     dbg!(&mutex);
    //     assert!(mutex.lock().unwrap().get("Todo").unwrap().len() > 0)
    // }

    // #[test]
    // fn test_parse_line() {
    //     let line = "TODOOOOOO";
    //     let res = todooer::parse_line(line);
    //     if res.is_ok() {
    //         let idx = res.unwrap().0;
    //         assert_eq!(idx, 0)
    //     }
    //     let line2 = "some other comments TODOOOOOO";
    //     let res = todooer::parse_line(line2);
    //     if res.is_ok() {
    //         let idx = res.unwrap().0;
    //         assert_eq!(idx, 20)
    //     }
    //     let line2 = "some other T comments TODOOOOOO";
    //     let res = todooer::parse_line(line2);
    //     if res.is_ok() {
    //         let idx = res.unwrap().0;
    //         assert_eq!(idx, 22)
    //     }
    // }

    #[test]
    fn test_par_examine_dir() {
        // Ensure you have a Todo comment in file for this to pass
        let pattern = ".rs";
        let res = todooer::par_examine_dir(&PathBuf::from("./src"), pattern);
        assert!(res.is_ok());
        assert!(res.unwrap().len() > 0)
    }

    #[test]
    fn test_find_todo() {
        let line1 = "TODOOOOOO";
        let res = todooer::find_todo(line1);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let line2 = "some other comments TODOOOOOO";
        let res = todooer::find_todo(line2);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 20);

        let line = "some other T comments TODOOOOOO";
        let res = todooer::find_todo(line);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 22);
    }

    #[test]
    fn test_handle_t() {
        let line1 = "TODOOOOOO";
        let pattern = ".rs";
        let (idx, priority, proper) = todooer::handle_t(line1, pattern);
        assert_eq!(priority, 6);

        let line = "some other T comments TODOOOOOOOO";
        let (idx, priority, proper) = todooer::handle_t(line, pattern);
        assert_eq!(priority, 8);
    }

    #[test]
    fn test_get_priority_and_proper() {
        let pattern = ".rs";
        let rem_6 = "TODOOOOOO";
        let old = "as long as last 3 good // ";
        let res = todooer::get_priority_and_proper(pattern, old, rem_6);
        assert_eq!(res, (6, true));

        let rem_17 = "TODOOOOOOOOOOOOOOOOO";
        let res = todooer::get_priority_and_proper(pattern, old, rem_17);
        assert_eq!(res, (17, true));

        let rem_17 = "TODOOOOOOOOOOOOOOOOO_OOOOOOOOOOO";
        let res = todooer::get_priority_and_proper(pattern, old, rem_17);
        assert_eq!(res, (17, true));

        let rem = "TODO";
        let res = todooer::get_priority_and_proper(pattern, old, rem);
        assert_eq!(res, (1, true));
    }
}
