//walks a filesystem and finds duplicate files
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use walkdir::WalkDir;
// TODO What
static MATCHES: &'static [&str] = &["TODO", "FIXME"];

pub fn get_comment_lines(filepath: &PathBuf) -> std::io::Result<Vec<(usize, String)>> {
    let mut file = BufReader::new(File::open(filepath)?);
    let mut correct_lines = Vec::<(usize, String)>::new();
    for (idx, line) in file.lines().enumerate() {
        let line = line?;
        // dbg!(idx);
        dbg!(&line);
        if line.is_empty() { continue; }
        // here starts_with takes an array of char
        // if !line.starts_with(['#', '\n']) { continue; }
        // if let (found) = line.find("TODO") {dbg!(&found)} else {};
        if !line.contains("TODO") { continue; }
        // TODO: Get Bytes so can get line position too
        correct_lines.push((idx, line));
    }
    // I'm leaving handling the empty case to the caller, but you can do it here too
    Ok(correct_lines)
}

#[derive(Debug)]
pub struct Line {
    pub line_text: String,
    pub line_num: usize,
    pub priority: usize,
    pub filename: String,
    pub row_index: usize,
}

// pub fn make_req() -> Result<String, String> {
//     let body: String = reqwest::blocking::get(url: "https://www.rust-lang.org")?.text()?;
// }

pub fn par_examine_dir(dir: &Path, _pattern: &str) -> Result<HashMap<String, Vec<Line>>, Box<dyn Error>> {
    let pattern = ".rs";
    let acc = std::sync::Mutex::new(Vec::<String>::new());
    let _ = list_files(dir, pattern, &acc);
    dbg!(&acc);
    par_get_comments(acc)
}

pub fn par_get_comments(acc: Mutex<Vec<String>>) -> Result<HashMap<String, Vec<Line>>, Box<dyn Error>> {
    let officials = std::sync::Mutex::new(HashMap::new());
    // let mut guard = acc.lock().unwrap();
    // let val = *guard;
    let val = acc.lock().unwrap().clone();
    dbg!(&val);
    let pb = indicatif::ProgressBar::new(val.len() as u64);
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap();
    pb.set_style(sty);
    val.par_iter().progress_with(pb).for_each(|file| {
        if let Ok(lines) = get_comment_lines(&PathBuf::from(file)) {
            dbg!(&file, &lines);
            // Consumes the iterator, returns an (Optional) String
            let new = lines.iter().map(|line| {
                let (idx, priority) = handle_t(&line.1);
                Line{line_text: line.1.to_owned(), line_num: line.0 + 1, filename: file.to_owned(), priority: priority, row_index: idx}
            }).collect::<Vec<Line>>();
            dbg!(&new);
            officials.lock().unwrap().insert(file.to_owned(), new);
            // Ok(new)
        } else {
            dbg!("No Comment Lines");
        }
    });
    Ok(officials.into_inner().unwrap())
}

pub fn list_files(dir: &Path, pattern: &str, acc: &Mutex<Vec<String>>) {
    // Determine all the dirs we need to scan (find all subdirs)
    // Send each subdir to a separate process to scan the files
    if dir.is_dir() {
        let paths = fs::read_dir(dir).unwrap();
        for entry in paths {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        list_files(&path, pattern, acc);
                    } else if path.is_file() {
                        let full_path = path.into_os_string().into_string();
                        match full_path {
                            Ok(path) => {
                                if path.contains(pattern) {
                                    acc.lock().unwrap().push(path)
                                }
                            },
                            Err(err) => {
                                // CLI apps need to write err to stderr
                                eprintln!("Error: {:?}", err);
                                dbg!("Error: {}", err);
                            },
                        }
                    }
                },
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    dbg!("Error: {}", err);
                },
            }
        }
    }
}

pub fn find_todo(line: &str) -> Result<usize, &str> {
    let idx = line.find("TODO");
    match idx {
        Some(idx) => {
            Ok(idx)
        },
        None => Err("No Todo Found")
    }
}

pub fn handle_t(line: &str) -> (usize, usize) {
    // let mut char_indices = line.char_indices();
    let idx = find_todo(line);
    match idx {
        Ok(idx) => {
            println!("{} is the start", idx);
            let (_old, rem) = line.split_at(idx);
            let priority = get_priority(rem);
            (idx, priority)
        },
        Err(err) => {
            dbg!("Nothing");
            (0, 0)
        }
    }
}

use std::iter::Peekable;

struct SequentialCount<I>
    where I: Iterator
{
    iter: Peekable<I>,
}

impl<I> SequentialCount<I>
    where I: Iterator
{
    fn new(iter: I) -> Self {
        SequentialCount { iter: iter.peekable() }
    }
}

impl<I> Iterator for SequentialCount<I>
    where I: Iterator,
          I::Item: Eq
{
    type Item = (I::Item, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // Check the next value in the inner iterator
        match self.iter.next() {
            // There is a value, so keep it
            Some(head) => {
                // We've seen one value so far
                let mut count = 1;
                // Check to see what the next value is without
                // actually advancing the inner iterator
                while self.iter.peek() == Some(&head) {
                    // It's the same value, so go ahead and consume it
                    self.iter.next();
                    count += 1;
                }
                // The next element doesn't match the current value 
                // complete this iteration 
                Some((head, count))
            }
            // The inner iterator is complete, so we are also complete
            None => None,
        }
    }
}

// At his point we have our Todo
pub fn get_priority(line: &str) -> usize {
    // Might be TODO or TODOOOOOO
    // If todo, split on the d to avoid the first o
    let (_one, two) = line.split_at(3);
    let chars = two.chars().peekable();
    dbg!(&chars);
    let mut char_iter = chars.into_iter();

    let count =
        match char_iter.next() {
            // There is a value, so keep it
            Some(head) => {
                // We've seen one value so far
                let mut count = 1;
                // Check to see what the next value is without
                // actually advancing the inner iterator
                while char_iter.peek() == Some(&head) {
                    // It's the same value, so go ahead and consume it
                    char_iter.next();
                    count += 1;
                }
                // The next element doesn't match the current value 
                // complete this iteration 
                return count
            }
            // The inner iterator is complete, so we are also complete
            None => 0,
        };

    count

    // let counts = SequentialCount::new(line.chars());
    // for (char, count) in counts {
    //     if char.to_string() == "O" && count > 1 {
    //         return count
    //     }
    // }
    // 1
}

// pub fn examine_dir(dir: &Path, pattern: &str) -> Mutex<HashMap<String, Vec<Line>>> {
//     let officials = std::sync::Mutex::new(HashMap::new());
//     walk_dir_for_files(dir, pattern,&officials);
//     officials
// }


// pub fn parse_line(line: &str) -> Result<(usize, char), String> {
//     // Might be looking at a 'TODO' or a 'TODOOOOOOOO'
//     let chars: Vec<char> = line.chars().collect();
//     dbg!(&chars);
//     let dev = match chars.clone().into_iter().enumerate().find(|(idx, char)| char.to_string() == "T") {
//         None => {
//             println!("T not found.");
//             Err("Oops".to_string())
//         },
//         Some(dev) => {
//             dbg!(dev.0, dev.1);
//             if chars[dev.0 + 1].to_string() == "O" && chars[dev.0 + 2].to_string() == "D" && chars[dev.0 + 3].to_string() == "O" {
//                 Ok(dev)
//             } else {
//                 println!("False Alarm T");
//                 let split_idx = dev.0 + 1;
//                 // Re examine line to find actual Todo
//                 let (_old, remaining) = line.split_at(split_idx);
//                 let rem_chars: Vec<char> = remaining.chars().collect();
//                 let rem_dev = match rem_chars.clone().into_iter().enumerate().find(|(idx, char)| char.to_string() == "T") {
//                     None => {
//                         println!("T not found.");
//                         Err("Oops".to_string())
//                     },
//                     Some(dev) => {
//                         dbg!(dev.0, dev.1);
//                         if rem_chars[dev.0 + 1].to_string() == "O" && rem_chars[dev.0 + 2].to_string() == "D" && rem_chars[dev.0 + 3].to_string() == "O" {
//                             Ok((dev.0 + split_idx, dev.1))
//                         } else {
//                             Ok((dev.0 + 777, dev.1))
//                         }
//                     }
//                 };
//                 rem_dev
//             }
//         }
//     };
//     dev
// }

// pub fn walk_dir_for_files(dir: &Path, pattern: &str, acc: &Mutex<HashMap<String, Vec<Line>>>) {
    
//     if dir.is_dir() {
//         let paths = fs::read_dir(dir);
//         match paths {
//             Ok(paths) => {
//                 for entry in paths {
//                     match entry {
//                         Ok(entry) => {
//                             let path = entry.path();
//                             if path.is_dir() {
//                                 walk_dir_for_files(&path, pattern, acc);
//                             } else if path.is_file() {
//                                 match path.file_name() {
//                                     Some(filename) => {
//                                         match filename.to_str() {
//                                             Some(string) => {
//                                                 // Only check the files that have extension matching pattern passed in via ARGV
//                                                 if string.contains(pattern) {
//                                                     if let Ok(lines) = get_comment_lines(&path) {
//                                                         // Consumes the iterator, returns an (Optional) String
//                                                         let new = lines.iter().map(|line| {
//                                                             Line{line_text: line.1.to_owned(), line_num: line.0 + 1, filename: string.to_owned()}
//                                                         }).collect::<Vec<Line>>();
//                                                         acc.lock().unwrap().insert("Todo".to_owned(), new);
//                                                         // Ok(new)
//                                                     } else {
//                                                         dbg!("No Comment Lines");
//                                                     }
//                                                 }
//                                             },
//                                             None => {
//                                                 eprintln!("No String Filename");
//                                                 println!("No String Filename")
//                                             }
//                                         }
//                                     },
//                                     None => {
//                                         eprintln!("No Path Filename");
//                                         println!("No Path Filename")
//                                     }                                    
//                                 }
//                             }
//                         },
//                         Err(err) => {
//                             eprintln!("No Entry {}", err)
//                         }
//                     }

//                 // paths.filter_map(|entry| {
//                 //     entry.ok().and_then(|e|
//                 //       e.path().file_name()
//                 //       .and_then(|n| n.to_str().map(|s| String::from(s)))
//                 //     )
//                 //   }).collect::<Vec<String>>();
//                 }
//             }
//             Err(err) => {
//                 println!("Ugh")
//             }
//         }
//     }
// }

pub fn walk_file_for_lines(entry: &PathBuf) -> Result<Vec<Line>, String> {
    if let Ok(lines) = get_comment_lines(&entry) {
        // Consumes the iterator, returns an (Optional) String
        let new = lines.iter().map(|line| {
            let (idx, priority) = handle_t(&line.1);
            Line{line_text: line.1.to_owned(), line_num: line.0, filename: entry.file_name().unwrap().to_str().unwrap().to_owned(), priority: priority, row_index: idx}
        }).collect::<Vec<Line>>();
        Ok(new)
    } else {
        Err("No lines".to_owned())
    }
} 