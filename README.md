# Sudoku solver

Sudoku solver written in Rust. Uses logic as much as possible, and only relies on brute-force techniques when absolutely necessary. Made to get to grips with Rust, especially its borrow checker.

## Usage

Build and/or run with Cargo using the usual commands. The application reads from a text file which contains a puzzle on each line. Puzzles are represented by 81 numbers ranging from 1 to 9, or a dot ('.') for an empty cell. An example puzzle would be:

`..3.7..4...6..23.1.89.........1.7.8.517.....6...4.....271..9..5.95..........2....`

The puzzles in their solved form are optionally output to a file. After solving all puzzles, several statistics are shown in the command line. The application requires one or two arguments, the first of which should be the input file and the second is optionally the output file:

```
sudoku-solver <input-file> <output-file>
```
