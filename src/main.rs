#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use regex::Regex;
use std::collections::VecDeque;

enum Source<I>
where
    I: Iterator,
{
    Direct(std::iter::Enumerate<I>),
    Buffered(VecDeque<(usize, I::Item)>),
}

struct ISlice<I>
where
    I: Iterator,
    I::Item: Clone,
{
    source: Source<I>,
    start: i32,
    end: Option<i32>,
}

impl<I> ISlice<I>
where
    I: Iterator,
    I::Item: Clone,
{
    fn new(iter: I, slice: Slice, iterable_len: Option<usize>) -> Self {
        let need_len = slice.start.map_or(false, |x| x < 0) || slice.end.map_or(false, |x| x < 0);
        let mut iter = iter;

        let (normalized_start, normalized_end, source) = if need_len && iterable_len.is_none() {
            // Buffer the entire iterator to determine its length
            let buffer: Vec<(usize, I::Item)> = iter.enumerate().collect();
            let length = buffer.len();
            let (normalized_start, normalized_end) = normalize_indices(
                slice.start.unwrap_or(0),
                slice.end,
                Some(length),
            );
            (
                normalized_start,
                normalized_end,
                Source::Buffered(VecDeque::from(buffer)),
            )
        } else {
            // Length is known or no negative indices
            let (normalized_start, normalized_end) = normalize_indices(
                slice.start.unwrap_or(0),
                slice.end,
                iterable_len,
            );

            let source = match (normalized_start, normalized_end) {
                (start, Some(end)) if start < 0 || end < 0 => {
                    // Buffer the iterator to determine length
                    let buffer: Vec<(usize, I::Item)> = iter.enumerate().collect();
                    Source::Buffered(VecDeque::from(buffer))
                }
                _ => Source::Direct(iter.enumerate()),
            };

            (normalized_start, normalized_end, source)
        };

        ISlice {
            source,
            start: normalized_start,
            end: normalized_end,
        }
    }
}

impl<I> Iterator for ISlice<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.source {
            Source::Direct(iter) => {
                while let Some((i, item)) = iter.next() {
                    if (i as i32) >= self.start
                        && (self.end.is_none() || (i as i32) < self.end.unwrap())
                    {
                        return Some(item);
                    }
                }
                None
            }
            Source::Buffered(buf) => {
                while let Some((i, item)) = buf.pop_front() {
                    if (i as i32) >= self.start
                        && (self.end.is_none() || (i as i32) < self.end.unwrap())
                    {
                        return Some(item.clone());
                    }
                }
                None
            }
        }
    }
}

fn islice<I>(
    iter: I,
    slice: Slice,
    iterable_len: Option<usize>,
) -> ISlice<I::IntoIter>
where
    I: IntoIterator,
    I::Item: Clone,
{
    ISlice::new(iter.into_iter(), slice, iterable_len)
}

fn normalize_indices(
    start: i32,
    end: Option<i32>,
    len: Option<usize>,
) -> (i32, Option<i32>) {
    match len {
        Some(length) => {
            let normalized_start = if start < 0 {
                length as i32 + start
            } else {
                start
            };

            let normalized_end = end.map(|e| {
                if e < 0 {
                    length as i32 + e
                } else {
                    e
                }
            });

            (normalized_start, normalized_end)
        }
        None => (start, end),
    }
}

#[derive(Debug, Clone)]
struct Slice {
    start: Option<i32>,
    end: Option<i32>,
}

fn slice_from_spec(slice_spec: &str) -> Slice {
    if slice_spec.contains(':') {
        let parts: Vec<&str> = slice_spec.split(':').collect();
        let (beg, end) = (parts[0], parts[1]);
        let start = if beg.is_empty() {
            None
        } else {
            Some(beg.parse().unwrap())
        };
        let end = if end.is_empty() {
            None
        } else {
            Some(end.parse().unwrap())
        };
        Slice { start, end }
    } else {
        let pos: i32 = slice_spec.parse().unwrap();
        if pos == -1 {
            Slice {
                start: Some(pos),
                end: None,
            }
        } else {
            Slice {
                start: Some(pos),
                end: Some(pos + 1),
            }
        }
    }
}

fn filtered_line(line: &str, slice: Slice) -> Option<String> {
    let eol_to_preserve = if line.ends_with('\n') { "\n" } else { "" };

    let fields = split_with_positions(line, r"\s+");

    let total_fields = fields.len() as i32;

    let start_idx = match slice.start {
        Some(i) if i < 0 => (total_fields + i) as usize,
        Some(i) => i as usize,
        None => 0,
    };

    let end_idx = match slice.end {
        Some(i) if i < 0 => (total_fields + i) as usize,
        Some(i) => i as usize,
        None => fields.len(),
    };

    // Check for invalid indices
    if start_idx >= fields.len() || start_idx >= end_idx {
        return None;
    }

    let sliced_fields = &fields[start_idx..end_idx];
    if sliced_fields.is_empty() {
        return None;
    }

    let start_pos = sliced_fields[0].start;
    let end_pos = sliced_fields[sliced_fields.len() - 1].end;

    let mut result = " ".repeat(start_pos);
    result.push_str(&line[start_pos..end_pos]);
    result.push_str(eol_to_preserve);

    Some(result)
}

#[derive(Debug)]
struct SplitResult<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

fn split_with_positions<'a>(
    str_to_split: &'a str,
    pattern: &str,
) -> Vec<SplitResult<'a>> {
    let regex = Regex::new(pattern).unwrap();
    let mut result = Vec::new();
    let mut last_end = 0;

    for capture in regex.find_iter(str_to_split) {
        let start = capture.start();
        if start > last_end {
            result.push(SplitResult {
                text: &str_to_split[last_end..start],
                start: last_end,
                end: start,
            });
        }
        last_end = capture.end();
    }

    if last_end < str_to_split.len() {
        result.push(SplitResult {
            text: &str_to_split[last_end..],
            start: last_end,
            end: str_to_split.len(),
        });
    }

    result
}

fn pick<I>(
    lines: I,
    row_spec: &str,
    col_spec: &str,
    line_count: Option<usize>,
) -> impl Iterator<Item = String>
where
    I: IntoIterator<Item = String>,
{
    let row_slice = slice_from_spec(row_spec);
    let col_slice = slice_from_spec(col_spec);

    islice(lines, row_slice, line_count)
        .filter_map(move |line| filtered_line(&line, col_slice.clone()))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut input_file = None;
    let mut row = None;
    let mut col = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                return;
            }
            "-f" | "--file" => {
                if i + 1 < args.len() {
                    input_file = Some(args[i + 1].clone());
                    i += 2;
                    continue;
                } else {
                    eprintln!("Error: -f/--file requires a value");
                    std::process::exit(2);
                }
            }
            _ => {
                if row.is_none() {
                    row = Some(args[i].clone());
                } else if col.is_none() {
                    col = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    let (row_spec, col_spec) = match (row, col) {
        (Some(r), Some(c)) => (r, c),
        _ => {
            print_usage();
            std::process::exit(1);
        }
    };

    // Process input
    if let Some(path) = input_file {
        let line_count = count_lines(&path);
        if let Ok(lines) = read_lines(&path) {
            process_lines(lines, &row_spec, &col_spec, line_count);
        }
    } else {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().map(|l| l.unwrap());
        process_lines(lines, &row_spec, &col_spec, None);
    }
}

fn print_usage() {
    println!(
        r#"
Usage: getitem [-h] [-f FILE] row_spec col_spec

Filter stdin and print specific rows and columns, specifying
them in Python's slicing syntax, separating columns by whitespace.

If passed FILE, it will read the file twice but not buffer.

For example:
cat myfile | ./getitem :5 0     # Print the column 0 of the first 5 rows.
cat myfile | ./getitem 0 :      # Print the first row, all of it.
cat myfile | ./getitem -10 0:2  # Print the first 2 columns of the last 10 rows.
cat myfile | ./getitem -2:-1 :  # Prints all fields of the second to last row.
"#
    );
}

fn read_lines<P>(filename: P) -> io::Result<impl Iterator<Item = String>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file)
        .lines()
        .map(|l| l.expect("Could not read line")))
}

fn count_lines<P>(filename: P) -> Option<usize>
where
    P: AsRef<Path>,
{
    fs::read_to_string(filename)
        .ok()
        .map(|contents| contents.lines().count())
}

fn process_lines<I>(
    lines: I,
    row_spec: &str,
    col_spec: &str,
    line_count: Option<usize>,
) where
    I: IntoIterator<Item = String>,
{
    for line in pick(lines, row_spec, col_spec, line_count) {
        print!("{}", line);
    }
}
