use std::time::Instant;
use xcc::Matrix;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = match std::env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: sudoku <path to sudoku file>");
            println!("Example: sudoku examples/sudoku17.txt");
            std::process::exit(1);
        }
    };

    // Read the file into a string.
    let input = std::fs::read_to_string(arg)?;

    let mut count = 0;
    let start_time = Instant::now();
    for line in input.lines() {
        if line.starts_with('#') {
            continue;
        }
        count += 1;
        match solve_sudoku(line) {
            Ok(solution) => println!("{}", solution),
            Err(msg) => println!("Error: {}", msg),
        }
    }

    // Report how long it took to solve all the puzzles.  On my machine,
    // this is about 1.5 seconds for 1000 puzzles.
    let elapsed = start_time.elapsed();
    println!("Solved {} puzzles in {:?}", count, elapsed);
    Ok(())
}

/// Solves a Sudoku puzzle in the format
/// `.91.7...25.....7..3.7.4..69.4.3........59..1......42.....9....5....1.8....96..3..`
/// where `.` represents an empty cell.
///
/// Returns a string containing the solution if there is exactly one solution,
/// or an error message if there are no solutions or multiple solutions.
pub fn solve_sudoku(input: &str) -> Result<String, &'static str> {
    let mut matrix = build_matrix(input);
    match matrix.solve_unique() {
        xcc::Unique::None => Err("No solution"),
        xcc::Unique::One(solution) => {
            // From the solution, we can call `meanings` to get a list
            // of Placement objects. These describe which numbers go in
            // which cells.
            let placements = solution.meanings(&matrix);
            let mut grid = ['.'; 81];
            for Placement { row, col, value } in placements {
                grid[row * 9 + col] = char::from_digit(*value, 10).unwrap();
            }
            Ok(grid.iter().collect())
        }
        xcc::Unique::Ambiguous(_, _) => Err("Multiple solutions"),
    }
}

fn build_matrix(input: &str) -> Matrix<Placement> {
    let mut builder = Matrix::builder();

    // Create items describing how the Sudoku grid must be filled.
    // First, each cell must be filled with a number. These constraints
    // look like "F35" (for "row 3, column 5 is filled").
    for row in 0..9 {
        for col in 0..9 {
            builder.add_primary_item(format!("F{}{}", row, col));
        }
    }
    // Next, each row must contain each number exactly once.
    // These constraints look like "R32" (for "row 3 contains a 2").
    for row in 0..9 {
        for value in 1..10 {
            builder.add_primary_item(format!("R{}{}", row, value));
        }
    }

    // Next, each column must contain each number exactly once.
    // These constraints look like "C52" (for "column 5 contains a 2").
    for col in 0..9 {
        for value in 1..10 {
            builder.add_primary_item(format!("C{}{}", col, value));
        }
    }

    // Finally, each 3x3 box must contain each number exactly once.
    // These constraints look like "B12" (for "box 1 contains a 2").
    for box_row in 0..3 {
        for box_col in 0..3 {
            for value in 1..10 {
                builder.add_primary_item(format!("B{}{}", box_row * 3 + box_col, value));
            }
        }
    }

    // To model the specific Sudoku puzzle we're given, we'll go through the input,
    // and for each char, do one of two things:
    //
    // - If the char is a dot, we'll add 9 options, one of each possible digit
    //   for that cell.
    //
    // - If the char is a digit, we'll only add the option for that specific digit.

    let chars = input.chars().collect::<Vec<_>>();
    for (row, cells) in chars.chunks(9).enumerate() {
        for (col, &c) in cells.iter().enumerate() {
            let box_num = row / 3 * 3 + col / 3;
            let values = if c == '.' {
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
            } else {
                let value = c.to_digit(10).expect("invalid digit");
                vec![value]
            };

            for value in values {
                builder.add_option(
                    Placement { row, col, value },
                    [
                        format!("F{}{}", row, col),
                        format!("R{}{}", row, value),
                        format!("C{}{}", col, value),
                        format!("B{}{}", box_num, value),
                    ],
                );
            }
        }
    }

    builder.build()
}

/// A placement of a number in a Sudoku grid.
#[derive(Debug)]
struct Placement {
    row: usize,
    col: usize,
    value: u32,
}
