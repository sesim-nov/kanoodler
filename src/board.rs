// This file contains board state information and constants. 
pub const WIDTH: usize = 11;
pub const HEIGHT: usize = 5;
pub const NUM_CELLS: usize = WIDTH * HEIGHT; // 55
pub const NUM_PIECES: usize = 12;

pub type Coord = (i32, i32);

// Official Kanoodle pieces transcribed directly from the reference image.
// Total beads = 55 (matches the 11x5 board exactly).
pub const PIECE_SHAPES: [&[Coord]; NUM_PIECES] = [
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

pub const PIECE_LABELS: [char; NUM_PIECES] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
];