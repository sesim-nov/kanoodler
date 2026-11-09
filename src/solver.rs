use crate::board::*;
use std::collections::HashSet;


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
            let transformed: Vec<Coord> = base
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

pub struct Solver {
    cell_placements: Vec<Vec<usize>>,
    placements: Vec<Placement>,
}

impl Solver {
    pub fn new() -> Self {
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

    /// Parses a 2D ASCII grid representation of the initial board state.
    /// Returns (initial_board_mask, initial_piece_mask, initial_solution_indices)
    pub fn parse_initial_state(&self, grid: &[&str]) -> Result<(u64, u16, Vec<usize>), String> {
        if grid.len() != HEIGHT {
            return Err(format!("Grid must have {} rows", HEIGHT));
        }

        let mut piece_masks = std::collections::HashMap::new();

        // 1. Build a target bitmask for each character found on the grid
        for (r, row) in grid.iter().enumerate() {
            let chars: Vec<char> = row.chars().filter(|c| !c.is_whitespace()).collect();
            if chars.len() != WIDTH {
                return Err(format!("Row {} must have {} columns (ignoring spaces)", r, WIDTH));
            }

            for (c, &ch) in chars.iter().enumerate() {
                if ch != '.' {
                    let cell_idx = r * WIDTH + c;
                    let mask = piece_masks.entry(ch).or_insert(0u64);
                    *mask |= 1u64 << cell_idx;
                }
            }
        }

        let mut board_mask = 0u64;
        let mut piece_mask = 0u16;
        let mut initial_solution = Vec::new();

        // 2. Match each constructed mask to a valid precomputed Placement
        for (ch, target_mask) in piece_masks {
            let p_idx = PIECE_LABELS
                .iter()
                .position(|&l| l == ch)
                .ok_or_else(|| format!("Invalid piece label on board: {}", ch))?;

            let matching_placement_idx = self
                .placements
                .iter()
                .position(|p| p.piece_idx == p_idx && p.mask == target_mask)
                .ok_or_else(|| format!("Invalid geometry or placement for piece '{}'", ch))?;

            board_mask |= target_mask;
            piece_mask |= 1 << p_idx;
            initial_solution.push(matching_placement_idx);
        }

        Ok((board_mask, piece_mask, initial_solution))
    }

    pub fn solve(
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

    pub fn print_board(&self, solution: &[usize]) {
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
    
    pub fn get_placement_count(&self) -> usize {
        self.placements.len()
    }
}