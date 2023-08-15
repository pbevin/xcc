use criterion::BenchmarkId;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::time::Duration;
use xcc::ColoredItem;
use xcc::Matrix;

pub fn sudoku_matrix(c: &mut Criterion) {
    c.bench_function("build_sudoku_matrix", |b| {
        let (items, options) = init();
        b.iter(|| {
            let _ = build_sudoku_matrix(items.len(), &options);
        });
    });
}
pub fn add_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_option");
    group.measurement_time(Duration::from_secs(10));
    for size in [10, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(2 * size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            // create 2n items, with n of them having a color
            let max = n;
            let mut matrix = Matrix::<()>::new(n, n);
            let items2 = (0..max)
                .map(ColoredItem::new)
                .chain((max..2 * max).map(|i| ColoredItem::with_color(i, i)))
                .collect::<Vec<_>>();
            b.iter(|| matrix.add_option(black_box(&items2), ()));
        });
    }
}

criterion_group!(benches, add_option);
criterion_main!(benches);

type Items = Vec<usize>;
type Options = Vec<(usize, Vec<ColoredItem>)>;

/// Creates a matrix from items and options.
pub fn build_sudoku_matrix(
    num_primary_items: usize,
    options: &[(usize, Vec<ColoredItem>)],
) -> Matrix<usize> {
    let num_secondary_items = 0;
    let mut matrix = Matrix::new(num_primary_items, num_secondary_items);

    for (placement, items) in options.iter() {
        matrix.add_option(items, *placement);
    }
    matrix
}

/// Creates items and options for benchmarking.  The resulting XCC problem is
/// isomorphic to Sudoku without any clues.  This uses the low-level API for
/// benchmarking; see examples/sudoku.rs for a more realistic Sudoku solver.
pub fn init() -> (Items, Options) {
    let c = |t: usize, row: usize, column: usize| ColoredItem::new(t * 81 + row * 9 + column);
    let items = (0..324).collect();

    let mut options = Vec::new();
    let mut count = 0;
    for row in 0..9 {
        for col in 0..9 {
            let box_num = row / 3 * 3 + col / 3;
            for digit in 0..=8 {
                let items = vec![
                    c(0, row, col),
                    c(1, row, digit),
                    c(2, col, digit),
                    c(3, box_num, digit),
                ];
                assert!(items.iter().all(|i| i.item() < 324));
                options.push((count, items));
                count += 1;
            }
        }
    }

    (items, options)
}
