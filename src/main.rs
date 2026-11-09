use std::collections::HashSet;
use std::time::Instant;

const WIDTH: usize = 11;
const HEIGHT: usize = 5;
const NUM_CELLS: usize = WIDTH * HEIGHT; // 55
const NUM_PIECES: usize = 12;

type Coord = (i32, i32);

// Official Kanoodle pieces transcribed directly from the reference image.
// Total beads = 55 (matches the 11x5 board exactly).
const PIECE_SHAPES: [&[Coord]; NUM_PIECES] = [
    // A (Orange) - L-tetromino (4)
    &[(0, 1), (1, 1), (2, 0), (2, 1)],
    // B (Red) - P-pentomino (5)
    &[(0, 1), (1, 0), (1, 1), (2, 0), (2, 1)],
    // C (Dark Blue) - Long L-pentomino (5)
    &[(0, 1), (1, 1), (2, 1), (3, 0), (3, 1)],
    // D (Light Pink) - Y-pentomino (5)
    &[(0, 1), (1, 1), (2, 0), (2, 1), (3, 1)],
    // E (Dark Green) - N-pentomino (5)
    &[(0, 1), (1, 1), (2, 0), (2, 1), (3, 0)],
    // F (White) - V-triomino (3)
    &[(0, 1), (1, 0), (1, 1)],
    // G (Light Blue) - V-pentomino (5)
    &[(0, 2), (1, 2), (2, 0), (2, 1), (2, 2)],
    // H (Bright Pink) - W-pentomino (5)
    &[(0, 2), (1, 1), (1, 2), (2, 0), (2, 1)],
    // I (Yellow) - U-pentomino (5)
    &[(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)],
    // J (Purple) - I-tetromino (4)
    &[(0, 0), (1, 0), (2, 0), (3, 0)],
    // K (Light Green) - O-tetromino (4)
    &[(0, 0), (0, 1), (1, 0), (1, 1)],
    // L (Grey) - X-pentomino (5)
    &[(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
];

const PIECE_LABELS: [char; NUM_PIECES] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
];

#[derive(Clone, Copy)]
struct Placement {
    piece_idx: usize,
    mask: u64,
}

fn normalize(coords: &[Coord]) -> Vec<Coord> {
    let min_r = coords.iter().map(|&(r, _)| r).min().unwrap_or(0);
    let min_c = coords.iter().map(|&(_, c)| c).min().unwrap_or(0);
    let mut normalized: Vec<Coord> = coords.iter().map(|&(r, c)| (r - min_r, c - min_c)).collect();
    normalized.sort();
    normalized
}

fn generate_orientations(base: &[Coord]) -> Vec<Vec<Coord>> {
    let mut unique = HashSet::new();

    for flip in [false, true] {
        for rot in 0..4 {
            let mut transformed: Vec<Coord> = base
                .iter()
                .map(|&(r, c)| {
                    let (mut r, mut c) = if flip { (r, -c) } else { (r, c) };
                    for _ in 0..rot {
                        let tmp = r;
                        r = c;
                        c = -tmp;
                    }
                    (r, c)
                })
                .collect();

            unique.insert(normalize(&transformed));
        }
    }

    unique.into_iter().collect()
}

fn generate_all_placements() -> Vec<Placement> {
    let mut placements = Vec::new();

    for (p_idx, base_coords) in PIECE_SHAPES.iter().enumerate() {
        let orientations = generate_orientations(base_coords);

        for shape in orientations {
            let max_r = shape.iter().map(|&(r, _)| r).max().unwrap();
            let max_c = shape.iter().map(|&(_, c)| c).max().unwrap();

            for offset_r in 0..(HEIGHT as i32 - max_r) {
                for offset_c in 0..(WIDTH as i32 - max_c) {
                    let mut mask = 0u64;
                    for &(r, c) in &shape {
                        let cell_idx = ((offset_r + r) as usize) * WIDTH + ((offset_c + c) as usize);
                        mask |= 1u64 << cell_idx;
                    }
                    placements.push(Placement {
                        piece_idx: p_idx,
                        mask,
                    });
                }
            }
        }
    }

    placements
}

struct Solver {
    cell_placements: Vec<Vec<usize>>,
    placements: Vec<Placement>,
}

impl Solver {
    fn new() -> Self {
        let placements = generate_all_placements();
        let mut cell_placements = vec![Vec::new(); NUM_CELLS];

        for (idx, p) in placements.iter().enumerate() {
            for c in 0..NUM_CELLS {
                if (p.mask & (1u64 << c)) != 0 {
                    cell_placements[c].push(idx);
                }
            }
        }

        Self {
            cell_placements,
            placements,
        }
    }

    fn solve(
        &self,
        board_mask: u64,
        piece_mask: u16,
        current_solution: &mut Vec<usize>,
        solutions: &mut Vec<Vec<usize>>,
        limit: usize,
    ) {
        if solutions.len() >= limit {
            return;
        }

        // All 55 cells covered
        if board_mask == (1u64 << NUM_CELLS) - 1 {
            solutions.push(current_solution.clone());
            return;
        }

        // Select the uncovered cell with the fewest valid candidate placements (MRV)
        let mut best_cell = None;
        let mut min_candidates = usize::MAX;

        for cell in 0..NUM_CELLS {
            if (board_mask & (1u64 << cell)) == 0 {
                let mut candidates = 0;
                for &p_idx in &self.cell_placements[cell] {
                    let p = &self.placements[p_idx];
                    if (piece_mask & (1 << p.piece_idx)) == 0 && (board_mask & p.mask) == 0 {
                        candidates += 1;
                    }
                }

                // Unfillable cell detected; prune entire branch
                if candidates == 0 {
                    return;
                }

                if candidates < min_candidates {
                    min_candidates = candidates;
                    best_cell = Some(cell);
                    if min_candidates == 1 {
                        break;
                    }
                }
            }
        }

        let chosen_cell = match best_cell {
            Some(c) => c,
            None => return,
        };

        for &p_idx in &self.cell_placements[chosen_cell] {
            let p = &self.placements[p_idx];

            if (piece_mask & (1 << p.piece_idx)) == 0 && (board_mask & p.mask) == 0 {
                current_solution.push(p_idx);

                self.solve(
                    board_mask | p.mask,
                    piece_mask | (1 << p.piece_idx),
                    current_solution,
                    solutions,
                    limit,
                );

                current_solution.pop();

                if solutions.len() >= limit {
                    return;
                }
            }
        }
    }

    fn print_board(&self, solution: &[usize]) {
        let mut grid = vec!['.'; NUM_CELLS];

        for &p_idx in solution {
            let p = &self.placements[p_idx];
            let label = PIECE_LABELS[p.piece_idx];
            for cell in 0..NUM_CELLS {
                if (p.mask & (1u64 << cell)) != 0 {
                    grid[cell] = label;
                }
            }
        }

        println!("┌{}┐", "──".repeat(WIDTH));
        for r in 0..HEIGHT {
            print!("│");
            for c in 0..WIDTH {
                print!("{} ", grid[r * WIDTH + c]);
            }
            println!("│");
        }
        println!("└{}┘", "──".repeat(WIDTH));
    }
}

fn main() {
    let start_init = Instant::now();
    let solver = Solver::new();
    println!(
        "Precomputed {} valid placements in {:?}",
        solver.placements.len(),
        start_init.elapsed()
    );

    let mut current_solution = Vec::with_capacity(NUM_PIECES);
    let mut solutions = Vec::new();

    let start_solve = Instant::now();
    solver.solve(0, 0, &mut current_solution, &mut solutions, 1);

    println!("Solved in {:?}", start_solve.elapsed());

    if let Some(first) = solutions.first() {
        println!("\nFirst valid solution found:");
        solver.print_board(first);
    } else {
        println!("No solution found.");
    }
}