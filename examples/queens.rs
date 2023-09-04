use clap::Parser;
use xcc::Matrix;

#[derive(Parser)]
struct Cmdline {
    /// Do not print solutions, just the count.
    #[clap(short, long)]
    no_print: bool,

    size: usize,
}

/// Solves the N-Queens problem: place N queens on an NxN chessboard
/// so that no queen can attack any other queen.  Queens can attack
/// horizontally, vertically, or diagonally.

pub fn main() {
    let cmdline = Cmdline::parse();
    let n = cmdline.size;

    let mut matrix = build_matrix(n);
    let start_time = std::time::Instant::now();
    let mut count = 0;
    for solution in matrix.solve_all() {
        if !cmdline.no_print {
            let placements = solution.meanings(&matrix);
            let mut grid = vec![vec!['.'; n]; n];
            for &Queen { row, col } in placements {
                grid[row][col] = 'Q';
            }
            #[allow(clippy::needless_range_loop)]
            for row in 0..n {
                for col in 0..n {
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

/// The meaning of an option is the position of a queen.
pub struct Queen {
    row: usize,
    col: usize,
}

pub fn build_matrix(n: usize) -> Matrix<Queen> {
    let mut builder = Matrix::builder();

    // We define primary items Ri and Ci to represent the presence of a queen in
    // row i and column j.  We also define secondary items Di and Ei to
    // represent the presence of a queen on the two diagonals.
    //
    // We do not assign a color to the secondary items, so they will be
    // implicitly assigned a unique color. This ensures that secondary items are
    // used at most once.
    for i in 0..n {
        builder.add_primary_item(format!("R{}", i));
        builder.add_primary_item(format!("C{}", i));
    }

    for i in 0..2 * n - 1 {
        builder.add_secondary_item(format!("D{}", i));
        builder.add_secondary_item(format!("E{}", i));
    }

    // For each square on the board, we add an option that says that its row,
    // column, and two diagonals are occupied.
    for row in 0..n {
        for col in 0..n {
            let d = row + col;
            let e = n - 1 - row + col;

            builder.add_option(
                Queen { row, col },
                [
                    format!("R{}", row),
                    format!("C{}", col),
                    format!("D{}", d),
                    format!("E{}", e),
                ],
            );
        }
    }

    builder.build().expect("Failed to build matrix")
}
