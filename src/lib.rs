use std::char;
use std::collections::HashSet;
use std::fmt;
use std::mem;

/// Convert the coordinates of a cell in a sudoku grid to the coordinates
/// of the square it is located in.
fn cell_to_square(coords: (usize, usize)) -> (usize, usize) {
    (coords.0 / 3, coords.1 / 3)
}

/// Convert the index of an element in the range 0..9 to the corresponding
/// coordinates in a 3x3 dimensional grid.
fn index_to_3x3_coords(idx: usize) -> (usize, usize) {
    (idx / 3, idx % 3)
}

#[derive(Debug)]
pub struct Solution {
    cells: [[u8; 9]; 9],
    pub brute_forces: u8,
}

impl fmt::Display for Solution {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (row_idx, row) in self.cells.iter().enumerate() {
            if row_idx % 3 == 0 {
                write!(fmt, "+-------+-------+-------+\n")?;
            }
            for (col_idx, value) in row.iter().enumerate() {
                if col_idx % 3 == 0 {
                    write!(fmt, "| ")?;
                }
                write!(fmt, "{}", value.to_string())?;
                write!(fmt, " ")?;
            }
            write!(fmt, "|\n")?;
        }
        write!(fmt, "+-------+-------+-------+")?;
        Ok(())
    }
}

impl Solution {
    pub fn row_representation(&self) -> String {
        self.cells
            .iter()
            .flat_map(|row| row.map(|cell| char::from_digit(cell.into(), 10).unwrap()))
            .collect()
    }
}

#[derive(Clone, Debug)]
enum Cell {
    Value(u8),
    Candidates(HashSet<u8>),
}

impl Default for Cell {
    fn default() -> Self {
        Self::Candidates([1, 2, 3, 4, 5, 6, 7, 8, 9].iter().cloned().collect())
    }
}

#[derive(Debug)]
enum Group {
    All,
    Row,
    Column,
    Square,
    None,
}

#[derive(Clone, Debug)]
struct Occurrences<T> {
    row: [[T; 9]; 9],
    col: [[T; 9]; 9],
    sqr: [[[T; 9]; 3]; 3],
}

impl Default for Occurrences<u8> {
    fn default() -> Self {
        Self {
            row: [[9; 9]; 9],
            col: [[9; 9]; 9],
            sqr: [[[9; 9]; 3]; 3],
        }
    }
}

impl Default for Occurrences<bool> {
    fn default() -> Self {
        Self {
            row: [[false; 9]; 9],
            col: [[false; 9]; 9],
            sqr: [[[false; 9]; 3]; 3],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Solver {
    cells: [[Cell; 9]; 9],
    value_occurrences: Occurrences<bool>,
    candidate_occurrences: Occurrences<u8>,
    unfilled_cells: u8,
    brute_force_fills: u8,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            cells: Default::default(),
            value_occurrences: Default::default(),
            candidate_occurrences: Default::default(),
            unfilled_cells: 9 * 9,
            brute_force_fills: 0,
        }
    }
}

impl Solver {
    /// Load a puzzle represented by a 81 length vector of values
    /// and dots ('.') for non-filled cells.
    pub fn solve(puzzle: Vec<char>) -> Result<Solution, &'static str> {
        if puzzle.len() != 9 * 9 {
            return Err("invalid puzzle size");
        }

        // Load in values supplied by the puzzle.
        let mut grid: Solver = Default::default();
        for (idx, c) in puzzle.iter().enumerate() {
            if let Some(value) = c.to_digit(10) {
                grid.fill((idx / 9, idx % 9), value as u8)?;
            } else if *c != '.' {
                return Err("invalid character in puzzle");
            }
        }

        // Brute-force any remaining unfilled cells.
        let brute_force = grid.unfilled_cells > 0;
        if brute_force {
            grid = grid.brute_force()?;
        }

        Ok(Solution {
            cells: grid.cells.map(|row| {
                row.map(|cell| match cell {
                    Cell::Value(v) => v,
                    Cell::Candidates(_) => 0,
                })
            }),
            brute_forces: grid.brute_force_fills,
        })
    }

    /// Fill a value in the grid at specific coordinates.
    fn fill(&mut self, coords: (usize, usize), value: u8) -> Result<(), &'static str> {
        let square = cell_to_square(coords);
        match &mut self.cells[coords.0][coords.1] {
            Cell::Value(v) => {
                if *v != value {
                    return Err("cannot change already filled in cell");
                }
            }
            Cell::Candidates(cs) => {
                if mem::replace(
                    &mut self.value_occurrences.row[coords.0][value as usize - 1],
                    true,
                ) {
                    return Err("fill results in row conflict");
                }
                if mem::replace(
                    &mut self.value_occurrences.col[coords.1][value as usize - 1],
                    true,
                ) {
                    return Err("fill results in column conflict");
                }
                if mem::replace(
                    &mut self.value_occurrences.sqr[square.0][square.1][value as usize - 1],
                    true,
                ) {
                    return Err("fill results in square conflict");
                }

                let former_candidates = mem::take(cs);
                self.cells[coords.0][coords.1] = Cell::Value(value);
                self.unfilled_cells -= 1;

                // Remove candidates of filled in value in the row, column and square.
                for idx in 0..9 {
                    self.remove_candidate((coords.0, idx), value, Group::Row)?;
                    self.remove_candidate((idx, coords.1), value, Group::Column)?;

                    let relative = index_to_3x3_coords(idx);
                    let absolute = (relative.0 + square.0 * 3, relative.1 + square.1 * 3);
                    self.remove_candidate(absolute, value, Group::Square)?;
                }

                // Decrement occurrences as a result of the formerly present candidates
                // being replaced by a value and thus removed from the grid.
                for candidate in former_candidates {
                    self.decrement_occurrences(
                        coords,
                        candidate,
                        if candidate == value {
                            Group::All
                        } else {
                            Group::None
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Remove a candidate from a cell.
    fn remove_candidate(
        &mut self,
        coords: (usize, usize),
        candidate: u8,
        unique_occurence_ignore: Group,
    ) -> Result<(), &'static str> {
        if let Cell::Candidates(cs) = &mut self.cells[coords.0][coords.1] {
            if cs.remove(&candidate) {
                if cs.len() == 1 {
                    let leftover = *cs.iter().next().unwrap();
                    self.fill(coords, leftover)?;
                }
                self.decrement_occurrences(coords, candidate, unique_occurence_ignore)?;
            }
        }
        Ok(())
    }

    /// Decrement occurrence of a value in the row, column and square as a result
    /// of a candidate being removed from a cell.
    fn decrement_occurrences(
        &mut self,
        coords: (usize, usize),
        candidate: u8,
        unique_occurrence_ignore: Group,
    ) -> Result<(), &'static str> {
        let square = cell_to_square(coords);
        let candidate_idx = candidate as usize - 1;

        self.candidate_occurrences.row[coords.0][candidate_idx] -= 1;
        self.candidate_occurrences.col[coords.1][candidate_idx] -= 1;
        self.candidate_occurrences.sqr[square.0][square.1][candidate_idx] -= 1;

        if !matches!(unique_occurrence_ignore, Group::All) {
            if !matches!(unique_occurrence_ignore, Group::Row) {
                if self.candidate_occurrences.row[coords.0][candidate_idx] == 1 {
                    for col in 0..9 {
                        if let Cell::Candidates(cs) = &self.cells[coords.0][col] {
                            if cs.contains(&candidate) {
                                self.fill((coords.0, col), candidate)?;
                            }
                        }
                    }
                }
            }
            if !matches!(unique_occurrence_ignore, Group::Column) {
                if self.candidate_occurrences.col[coords.1][candidate_idx] == 1 {
                    for row in 0..9 {
                        if let Cell::Candidates(cs) = &self.cells[row][coords.1] {
                            if cs.contains(&candidate) {
                                self.fill((row, coords.1), candidate)?;
                            }
                        }
                    }
                }
            }
            if !matches!(unique_occurrence_ignore, Group::Square) {
                if self.candidate_occurrences.sqr[square.0][square.1][candidate_idx] == 1 {
                    for row in 0..3 {
                        for col in 0..3 {
                            let absolute_row = 3 * square.0 + row;
                            let absolute_col = 3 * square.1 + col;
                            if let Cell::Candidates(cs) = &self.cells[absolute_row][absolute_col] {
                                if cs.contains(&candidate) {
                                    self.fill((absolute_row, absolute_col), candidate)?;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Recursively apply brute-force by testing all candidates of the cell
    /// with the least candidates (highest entropy). Returns errors only if
    /// no branch can result in a valid solution.
    fn brute_force(self) -> Result<Self, &'static str> {
        if self.unfilled_cells == 0 {
            return Ok(self);
        }

        let mut highest_entropy: Option<(usize, usize, u8)> = None;
        for row in 0..9 {
            for col in 0..9 {
                if let Cell::Candidates(cs) = &self.cells[row][col] {
                    let current_entropy = (row, col, cs.len() as u8);
                    match highest_entropy {
                        None => highest_entropy = Some(current_entropy),
                        Some(former) => {
                            if current_entropy.2 < former.2 {
                                highest_entropy = Some(current_entropy);
                            }
                        }
                    }
                }
            }
        }

        match highest_entropy {
            None => return Err("no unfilled cell was found"),
            Some(highest_entropy) => {
                let coords = (highest_entropy.0, highest_entropy.1);
                match &self.cells[coords.0][coords.1] {
                    Cell::Value(_) => return Err("unfilled cell already filled in"),
                    Cell::Candidates(cs) => {
                        for candidate in cs {
                            let mut branch = self.clone();
                            if let Ok(_) = branch.fill(coords, *candidate) {
                                branch.brute_force_fills += 1;
                                if let Ok(branch) = branch.brute_force() {
                                    return Ok(branch);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err("all branches exhausted")
    }
}
