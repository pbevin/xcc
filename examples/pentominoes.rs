use std::collections::HashSet;

use clap::Parser;
use xcc::Matrix;

#[derive(Parser)]
struct Cmdline {
    /// Do not print solutions, just the count.
    #[clap(short, long)]
    no_print: bool,
}

/// Solves the pentominoes puzzle: place all 12 pentominoes into a 20x3 grid.
/// Each pentomino is a shape made of 5 connected squares. The 12 pentominoes
/// are traditionally named after letters they resemble: F, I, L, N, P, T, U,
/// V, W, X, Y, and Z.
///
/// https://en.wikipedia.org/wiki/Pentomino

pub fn main() {
    let cmdline = Cmdline::parse();
    let width = 20;
    let height = 3;

    let mut matrix = build_matrix(width, height);
    let start_time = std::time::Instant::now();
    let mut count = 0;
    for solution in matrix.solve_all() {
        if !cmdline.no_print {
            let placements = solution.meanings(&matrix);
            let mut grid = vec![vec!['.'; width]; height];
            for placement in placements {
                for &(row, col) in &placement.cells {
                    grid[row][col] = placement.piece;
                }
            }
            for row in 0..height {
                for col in 0..width {
                    print!("{}", grid[row][col]);
                }
                println!();
            }
            println!();
        }
        count += 1;
    }
    let elapsed = start_time.elapsed();
    println!("Found {} solutions in {:?}", count, elapsed);
}

#[derive(Debug, Clone)]
struct Placement {
    piece: char,
    cells: Vec<(usize, usize)>,
}

fn build_matrix(width: usize, height: usize) -> Matrix<Placement> {
    let mut builder = Matrix::builder();

    // Add primary items for each cell in the grid
    for row in 0..height {
        for col in 0..width {
            builder.add_primary_item(format!("C{}_{}", row, col));
        }
    }

    // Add primary items for each piece (must use each piece exactly once)
    let pieces = ['F', 'I', 'L', 'N', 'P', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
    for &piece in &pieces {
        builder.add_primary_item(piece.to_string());
    }

    // Define the shapes of the pentominoes
    let shapes = vec![
        // F
        vec![(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)],
        // I
        vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
        // L
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (3, 1)],
        // N
        vec![(0, 0), (1, 0), (2, 0), (2, 1), (3, 1)],
        // P
        vec![(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)],
        // T
        vec![(0, 0), (0, 1), (0, 2), (1, 1), (2, 1)],
        // U
        vec![(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)],
        // V
        vec![(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)],
        // W
        vec![(0, 0), (1, 0), (1, 1), (2, 1), (2, 2)],
        // X
        vec![(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
        // Y
        vec![(0, 0), (1, 0), (2, 0), (2, 1), (3, 0)],
        // Z
        vec![(0, 0), (0, 1), (1, 1), (2, 1), (2, 2)],
    ];

    for (&piece, shape) in pieces.iter().zip(shapes.iter()) {
        // Eliminate symmetric solutions by keeping track of
        // options we've already added.
        let mut seen = HashSet::new();

        // Try each possible position and rotation/reflection
        for row in 0..height {
            for col in 0..width {
                // Try each of the 8 possible rotations/reflections
                for transform in 0..8 {
                    let mut transformed = Vec::new();
                    let mut valid = true;

                    // Apply transformation and translation
                    for &(dr, dc) in shape {
                        let (tr, tc) = match transform {
                            0 => (dr, dc),   // Original
                            1 => (-dr, dc),  // Flip horizontally
                            2 => (dr, -dc),  // Flip vertically
                            3 => (-dr, -dc), // Rotate 180°
                            4 => (dc, dr),   // Rotate 90° clockwise
                            5 => (-dc, dr),  // Rotate 90° clockwise + flip horizontal
                            6 => (dc, -dr),  // Rotate 90° counterclockwise
                            7 => (-dc, -dr), // Rotate 90° counterclockwise + flip horizontal
                            _ => unreachable!(),
                        };

                        let new_row = (row as isize + tr) as usize;
                        let new_col = (col as isize + tc) as usize;

                        if new_row >= height || new_col >= width {
                            valid = false;
                            break;
                        }

                        transformed.push((new_row, new_col));
                    }

                    if valid {
                        let mut items = vec![piece.to_string()];
                        for &(r, c) in &transformed {
                            items.push(format!("C{}_{}", r, c));
                        }

                        // Sort the items so that, for example,
                        // (C0_0, C0_1, C0_2, C0_3, C0_4) and
                        // (C0_4, C0_3, C0_2, C0_1, C0_0) are considered
                        // the same.
                        let mut sorted = items.clone();
                        sorted.sort();

                        if seen.insert(sorted) {
                            builder.add_option(
                                Placement {
                                    piece,
                                    cells: transformed,
                                },
                                items,
                            );
                        }
                    }
                }
            }
        }
    }

    builder.build().expect("Failed to build matrix")
}
