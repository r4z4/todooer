use std::path::PathBuf;
//Searches a path for duplicate files
use clap::Parser;
use todooer::Line;

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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Report { path, pattern }) => {
            //count files matching a pattern
            println!("Counting files in {} matching {}", path, pattern);
            let path = PathBuf::from(path);


            // let mutex = todooer::examine_dir(&path, &pattern);
            // let data = mutex.lock().unwrap();
            // println!("{:?}", data);

            let _ = todooer::par_examine_dir(&path, &pattern);
        }

        None => {
            println!("No command given");
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_file() {
        let path = PathBuf::from("./src/main.rs");
        let lines = todooer::walk_file_for_lines(&path).unwrap();
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
        let (idx, priority) = todooer::handle_t(line1);
        assert_eq!(priority, 6);

        let line = "some other T comments TODOOOOOOOO";
        let (idx, priority) = todooer::handle_t(line);
        assert_eq!(priority, 8);
    }

    #[test]
    fn test_get_priority() {
        let line_6 = "TODOOOOOO";
        let res = todooer::get_priority(line_6);
        assert_eq!(res, 6);

        let line_17 = "TODOOOOOOOOOOOOOOOOO";
        let res = todooer::get_priority(line_17);
        assert_eq!(res, 17);

        let line_17 = "TODOOOOOOOOOOOOOOOOO_OOOOOOOOOOO";
        let res = todooer::get_priority(line_17);
        assert_eq!(res, 17);

        let line = "TODO";
        let res = todooer::get_priority(line);
        assert_eq!(res, 1);
    }
}
