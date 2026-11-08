use std::collections::HashSet;

const WIDTH: usize = 11;
const HEIGHT: usize = 5;
const NUM_CELLS: usize = WIDTH * HEIGHT; // 55
const NUM_PIECES: usize = 12;

type Coord = (i32, i32);

// Canonical piece shapes defined as relative (row, col) coordinates
const PIECE_SHAPES: [&[Coord]; NUM_PIECES] = [
    &[(0, 0), (0, 1), (0, 2), (1, 0)], // L-tromino variant / 4-bead (Piece A)
    &[(0, 0), (0, 1), (0, 2), (1, 1)], // T-tetromino (Piece B)
    &[(0, 0), (0, 1), (1, 0), (1, 1)], // 2x2 Square (Piece C)
    &[(0, 0), (0, 1), (0, 2), (0, 3)], // I-tetromino (Piece D)
    &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 1)], // P-pentomino (Piece E)
    &[(0, 0), (0, 1), (0, 2), (1, 0), (2, 0)], // V-pentomino / Corner (Piece F)
    &[(0, 0), (0, 1), (0, 2), (0, 3), (1, 0)], // L-pentomino (Piece G)
    &[(0, 0), (0, 1), (0, 2), (1, 1), (1, 2)], // Z-pentomino (Piece H)
    &[(0, 0), (0, 1), (0, 2), (1, 1), (2, 1)], // T-pentomino (Piece I)
    &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 2)], // U-pentomino (Piece J)
    &[(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)], // X-pentomino / Cross (Piece K)
    &[(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)], // W-pentomino (Piece L)
];

const PIECE_LABELS: [char; NUM_PIECES] =
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'];

/// A single placement of a piece on the board
#[derive(Clone, Copy)]
struct Placement {
    piece_idx: usize,
    mask: u64, // 55 bits used to represent occupied cells
}

impl {
    /// Normalize coordinates so the minimum row and minimum col are 0
    fn normalize(coords: &[Coord]) -> Vec<Coord> {
        let min_r = coords.iter().map(|&(r, _)| r).min().unwrap_or(0);
        let min_c = coords.iter().map(|&(_, c)| c).min().unwrap_or(0);
        let mut normalized: Vec<Coord> = coords
            .iter()
            .map(|&(r, c)| (r - min_r, c - min_c))
            .collect();
        normalized.sort();
        normalized
    }
}
/// Generate all unique rotations and reflections for a piece shape
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

/// Precompute all valid legal placements across the board
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
                        let row = offset_r + r;
                        let col = offset_c + c;
                        let cell_idx = (row as usize) * WIDTH + (col as usize);
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

/// Pre-index placements covering each cell for O(1) branch lookups
struct Solver {
    cell_placements: Vec<Vec<usize>>, // cell_idx -> list of placement indices
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

    /// Backtracking search (Knuth's Algorithm X on bitsets)
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

        // All 55 cells filled
        if board_mask == (1u64 << NUM_CELLS) - 1 {
            solutions.push(current_solution.clone());
            return;
        }

        // Exact Cover Heuristic: Choose the uncovered cell with minimum valid branches
        let mut best_cell = None;
        let mut min_candidates = usize::MAX;

        for cell in 0..NUM_CELLS {
            if (board_mask & (1u64 << cell)) == 0 {
                let candidates = self.cell_placements[cell]
                    .iter()
                    .filter(|&&p_idx| {
                        let p = &self.placements[p_idx];
                        (piece_mask & (1 << p.piece_idx)) == 0 && (board_mask & p.mask) == 0
                    })
                    .count();

                if candidates == 0 {
                    // Dead end: this cell cannot be covered
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

        // Try placing valid candidate pieces over chosen_cell
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
    let solver = Solver::new();
    println!(
        "Total legal placements precomputed: {}",
        solver.placements.len()
    );

    let mut current_solution = Vec::with_capacity(NUM_PIECES);
    let mut solutions = Vec::new();

    // Find the first solution
    solver.solve(0, 0, &mut current_solution, &mut solutions, 1);

    if let Some(first) = solutions.first() {
        println!("\nSolution found:");
        solver.print_board(first);
    } else {
        println!("No solutions found.");
    }
}
