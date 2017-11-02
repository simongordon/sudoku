use board::*;
use hintmap::HintMap;
use std::thread;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use scoped_threadpool::Pool;

/// Status from running checking/solving algorithms on `Board`
#[derive(Debug, PartialEq)]
pub enum BoardStatus {
    Solved,
    Valid,
    Invalid,
}

impl BoardStatus {
    #[cfg(test)]
    /// Board is Valid or Solved
    pub fn is_valid(&self) -> bool {
        match *self {
            BoardStatus::Valid | BoardStatus::Solved => true,
            BoardStatus::Invalid => false,
        }
    }

    /// Board is Solved
    pub fn is_solved(&self) -> bool {
        match *self {
            BoardStatus::Solved => true,
            _ => false,
        }
    }

    /// Board is Solved or Invalid - no further action needed on it
    pub fn is_finished(&self) -> bool {
        match *self {
            BoardStatus::Solved | BoardStatus::Invalid => true,
            _ => false,
        }
    }
}


/// Helpers
impl Board {
    pub fn check_status(&self) -> BoardStatus {
        let num_groups = self.side_length;

        for group_num in 0..num_groups {
            let row = self.get_row_values(group_num);
            let col = self.get_col_values(group_num);
            let grid = self.get_grid_values(group_num);
            for i in 0..num_groups {
                for j in 0..i + 1 {
                    if (i != j) &&
                        ((row[i] != None && row[i] == row[j]) ||
                            (col[i] != None && col[i] == col[j]) ||
                            (grid[i] != None && grid[i] == grid[j]))
                    {
                        return BoardStatus::Invalid;
                    }
                }
            }
        }

        // If the above tests passed and there are no
        // squares remaining, then the board is solved.
        if self.num_remaining() == 0 {
            return BoardStatus::Solved;
        }

        // If a HintMap has no hints for any square, that means that the board is invalid.
        let hmap = HintMap::from_board(self);
        for index in 0..self.num_squares {
            if self.get_val(index).unwrap() == None {
                let hints = hmap.get_hints(index);
                if hints.len() == 0 {
                    return BoardStatus::Invalid;
                }
            }
        }

        // If all above passes without returning, the board is Valid
        BoardStatus::Valid
    }

    /// Indicates whether the board, in its current state, can be solved.
    /// This is achieved by running the `solve_parallel` over it, so any overhead from that will occur.
    pub fn is_solvable(&self) -> bool {
        self.clone().solve_parallel().is_solved()
    }
}

impl Board {
    /// Solve the board with known techniques.
    pub fn solve_standard(&mut self) -> BoardStatus {
        //Check the status of the board first - saves us from trying to solve an already finished board
        let board_status = self.check_status();
        if board_status.is_finished() {
            return board_status;
        }

        loop {
            let original_remaining = self.num_remaining();
            let hmap = HintMap::from_board(self);

            for index in 0..self.squares.len() {
                if self.squares[index].is_none() {
                    if let Some(new_val) = hmap.find_square_answer(index) {
                        self.squares[index] = Some(new_val);
                    }
                }
            }

            let has_changed = self.num_remaining() != original_remaining;
            let board_status = self.check_status();
            if board_status.is_finished() | !has_changed {
                return board_status;
            }
        }
    }

    /// An alternative to `solve_standard()`. Applies similar solving techniques, but repetetively focuses on
    /// squares with the minimum number of hints first, in order to avoid more expensive computations.
    pub fn solve_ordered(&mut self) -> BoardStatus {
        let board_status = self.check_status();
        if board_status.is_finished() {
            return board_status;
        }

        'outer: loop {
            let mut start_again = false;
            let hmap = HintMap::from_board(self);
            let ordered = hmap.get_ordered();

            //Store the computed values in order to save time
            let mut rows: HashMap<usize, Vec<usize>> = HashMap::new();
            let mut cols: HashMap<usize, Vec<usize>> = HashMap::new();
            let mut grids: HashMap<usize, Vec<usize>> = HashMap::new();

            for (ind, hints) in ordered {
                if hints.len() == 1 {
                    self.squares[ind] = Some(hints[0]);
                    start_again = true; //This is instead of continue
                } else if start_again {
                    // start_again is called by the If branch above this one, but is regarding the actual for loop
                    continue 'outer;
                } else if hints.len() > 1 {
                    {
                        let mut hints = hints.clone();
                        let col_num = self.get_col_num(ind);

                        // If column indices already not in cache, add to cache
                        if !cols.contains_key(&col_num) {
                            cols.insert(
                                col_num,
                                self.get_col_indices(col_num)
                                    .iter()
                                    .map(|s| *s)
                                    .filter(|s| self.squares[*s] == None)
                                    .collect(),
                            );
                        }

                        let col_ind = cols.get(&col_num).unwrap();
                        for other_ind in col_ind {
                            let other_ind = *other_ind;
                            if ind != other_ind {
                                let other_hints = hmap.get_hints(other_ind);
                                for oth_hint in other_hints {
                                    if let Ok(hint_index) = hints.binary_search(&oth_hint) {
                                        hints.remove(hint_index);
                                    }
                                }
                            }
                        }
                        if hints.len() == 1 {
                            self.squares[ind] = Some(hints[0]);
                            continue 'outer;
                        }
                    }
                    //Do all these in separate blocks, as we don't want to mask the original "hints" variable
                    {
                        let mut hints = hints.clone();
                        let row_num = self.get_row_num(ind);
                        if !rows.contains_key(&row_num) {
                            rows.insert(
                                row_num,
                                self.get_row_indices(row_num)
                                    .iter()
                                    .map(|s| *s)
                                    .filter(|s| self.squares[*s] == None)
                                    .collect(),
                            );
                        }

                        let row_ind = rows.get(&row_num).unwrap();
                        for other_ind in row_ind {
                            let other_ind = *other_ind;
                            if ind != other_ind {
                                let other_hints = hmap.get_hints(other_ind);
                                for oth_hint in other_hints {
                                    if let Ok(hint_index) = hints.binary_search(&oth_hint) {
                                        hints.remove(hint_index);
                                    }
                                }
                            }
                        }
                        if hints.len() == 1 {
                            self.squares[ind] = Some(hints[0]);
                            continue 'outer;
                        }
                    }
                    {
                        let mut hints = hints.clone();

                        let grid_num = self.get_grid_num(ind);
                        if !grids.contains_key(&grid_num) {
                            grids.insert(
                                grid_num,
                                self.get_grid_indices(grid_num)
                                    .iter()
                                    .map(|s| *s)
                                    .filter(|s| self.squares[*s] == None)
                                    .collect(),
                            );
                        }

                        let grid_ind = grids.get(&grid_num).unwrap();
                        for other_ind in grid_ind {
                            let other_ind = *other_ind;
                            if ind != other_ind {
                                let other_hints = hmap.get_hints(other_ind);
                                for oth_hint in other_hints {
                                    if let Ok(hint_index) = hints.binary_search(&oth_hint) {
                                        hints.remove(hint_index);
                                    }
                                }
                            }
                        }
                        if hints.len() == 1 {
                            self.squares[ind] = Some(hints[0]);
                            continue 'outer;
                        }
                    }
                }
            } // End of For Loop

            return self.check_status();
        } // End of Outer Loop
    }


    /// The parallel solving algorithm. This is different from `solve_standard()` and `solve_ordered`
    /// as every blank square is evaluated for solving techniques in each iteration.
    pub fn solve_parallel(&mut self) -> BoardStatus {
        // Check the Board Status - if invalid or solved, don't bother performing this algorithm
        let board_status = self.check_status();
        if board_status.is_finished() {
            return board_status;
        }

        // Create a thread pool with the board's base_num as the number of workers
        let mut pool = Pool::new(self.base_num as u32);

        'outer: loop {
            let num_before = self.num_filled();
            let hmap = Arc::new(HintMap::from_board(self));
            pool.scoped(|scoped| {
                        let mut index = 0; // Manually build a loop index, since the ForEach loop below works on mutable references from an iterator
                        for val in &mut self.squares {
                            let hmap = hmap.clone(); // Copy the Arc
                            scoped.execute(move || {
                                if val.is_some() {
                                    return;
                                }
                                if let Some(new_val) = hmap.find_square_answer(index) {
                                    *val = Some(new_val);
                                    return;
                                }
                            });
                            index += 1;
                        }
                    });

            let has_changed = num_before < self.num_filled();
            let status = self.check_status();
            if status == BoardStatus::Valid && has_changed {
                continue 'outer;
            } else {
                return status;
            }
        } //End Outer loop
    }
}


// All the searching algorithms
impl Board {
    pub fn solve_search(&mut self) -> bool {
        match self.solve_ordered() {
            BoardStatus::Solved => return true,
            BoardStatus::Invalid => return false,
            BoardStatus::Valid => {}
        }
        let mut banned: HashSet<(usize, SquareType)> = HashSet::new();
        'outer: loop {
            //Build a hint map from the current board state
            let hmap = HintMap::from_board(self);

            //Order the hintmap from lowest number of hints to largest
            let ordered = hmap.get_ordered();
            if ordered.len() < 1 {
                return false;
            }

            //Minimum number of hints
            let min_hints = ordered[0].1.len();
            let mut acceptable: Vec<Board> = vec![];
            'inner: for (ind, vals) in ordered {
                //Only let us do ones from the min size
                if vals.len() != min_hints {
                    continue 'inner;
                }

                let mut working: Vec<Board> = vec![];

                'forloop: for val in vals {
                    if banned.contains(&(ind, val)) {
                        continue 'forloop;
                    }
                    let mut cloned = self.clone();
                    cloned.set_val(ind, Some(val)).unwrap();
                    let status = cloned.solve_ordered();
                    match status {
                        BoardStatus::Valid => {
                            working.push(cloned);
                        }
                        BoardStatus::Solved => {
                            *self = cloned;
                            return true;
                        }
                        BoardStatus::Invalid => {
                            banned.insert((ind, val));
                        }
                    }
                }
                if working.len() == 1 {
                    *self = working.pop().unwrap();
                    continue 'outer;
                } else if working.len() > 1 {
                    while let Some(thing) = working.pop() {
                        acceptable.push(thing);
                    }
                } else {
                    return false;
                }
            }

            acceptable.sort_by(|ref a1, ref a2| a2.num_filled().cmp(&a1.num_filled()));

            for working_board in acceptable {
                let mut working_board = working_board;
                if working_board.solve_search() {
                    *self = working_board;
                    return true;
                }
            }

            //If the above for loop has no results
            return false;
        }
    }

    pub fn solve_search_parallel(&mut self) -> bool {
        //TODO: Pass down banned list
        match self.solve_parallel() {
            BoardStatus::Solved => return true,
            BoardStatus::Invalid => return false,
            BoardStatus::Valid => {}
        }
        let mut banned: HashSet<(usize, SquareType)> = HashSet::new();
        'outer: loop {
            //Build a hint map from the current board state
            let hmap = HintMap::from_board(self);

            //Order the hintmap from lowest number of hints to largest
            let ordered = hmap.get_ordered();
            if ordered.len() < 1 {
                return false;
            }

            //Minimum number of hints
            let min_hints = ordered[0].1.len();
            let mut acceptable: Vec<Board> = vec![];
            'inner: for (ind, vals) in ordered {
                //Only let us do ones from the min size
                if vals.len() != min_hints {
                    continue 'inner;
                }

                let mut working: Vec<Board> = vec![];
                let mut handles = vec![];

                'forloop: for val in vals {
                    if banned.contains(&(ind, val)) {
                        continue 'forloop;
                    }
                    let mut cloned = self.clone();
                    handles.push(thread::spawn(
                        move || -> (usize, SquareType, Board, BoardStatus) {
                            cloned.set_val(ind, Some(val)).unwrap();
                            let status = cloned.solve_parallel();
                            (ind, val, cloned, status)
                        },
                    ));
                }
                let mut results = vec![];
                for handle in handles {
                    let result = handle.join().unwrap();
                    results.push(result);
                }
                for result in results {
                    let (ind, val, board, status) = result;
                    match status {
                        BoardStatus::Valid => {
                            working.push(board);
                        }
                        BoardStatus::Solved => {
                            *self = board;
                            return true;
                        }
                        BoardStatus::Invalid => {
                            banned.insert((ind, val));
                        }
                    }
                }

                if working.len() == 1 {
                    *self = working.pop().unwrap();
                    continue 'outer;
                } else if working.len() > 1 {
                    while let Some(working_board) = working.pop() {
                        acceptable.push(working_board);
                    }
                } else {
                    //There's no "working" values for a given square - this means that all possibilities are bad.
                    return false;
                }
            }

            acceptable.sort_by(|ref a1, ref a2| a2.num_filled().cmp(&a1.num_filled()));

            for working_board in acceptable {
                let mut working_board = working_board;
                //TODO: Pass down banned list
                if working_board.solve_search_parallel() {
                    *self = working_board;
                    return true;
                }
            }

            return false;
        }
    }

    pub fn solve_search_dfs(&mut self, max_depth: Option<i32>) -> BoardStatus {
        if let Some(depth) = max_depth {
            if depth < 1 {
                //This doesn't give much of an indication that we're finished recursion, but it's good enough
                return self.check_status();
            }
        }

        let solved_status = self.solve_standard();
        if solved_status.is_finished() {
            return solved_status;
        }

        let hmap = HintMap::from_board(self);

        let mut hint_counts: Vec<(usize, usize)> = hmap.hints
            .iter()
            .map(|(key, value)| (*key, value.len()))
            .collect();

        //Sort each square from smallest number of hints to largest
        hint_counts.sort_by(|a, b| b.1.cmp(&a.1));

        for i in 0..hint_counts.len() {
            let (index, _) = hint_counts[i];
            if self.squares[index] == None {
                for val in hmap.get_hints(index).iter() {
                    let mut cloned = self.clone();
                    cloned.squares[index] = Some(*val);

                    let new_depth = match max_depth {
                        Some(depth) => Some(depth - 1),
                        None => None,
                    };

                    // Recursive call
                    let solved_status = cloned.solve_search_dfs(new_depth);
                    if solved_status.is_finished() {
                        *self = cloned;
                        return solved_status;
                    }
                }
            }
        }

        self.check_status()
    }
}





#[cfg(test)]
mod tests {
    use board::*;
    use super::*;

    #[test]
    fn test_solve() {
        let solved = Board::from_string(SOLVED_PUZZLE_STR);
        let mut unsolved = Board::from_string(UNSOLVED_PUZZLE_STR);
        let solve_result = unsolved.solve_standard();
        assert!(solve_result.is_solved());
        for i in 0..NUM_SQUARES {
            assert_eq!(solved.squares[i], unsolved.squares[i]);
        }
    }

    #[test]
    fn test_solve_ordered() {
        let solved = Board::from_string(SOLVED_PUZZLE_STR);
        let mut unsolved = Board::from_string(UNSOLVED_PUZZLE_STR);
        let solve_result = unsolved.solve_ordered();
        assert!(solve_result.is_solved());
        for i in 0..NUM_SQUARES {
            assert_eq!(solved.squares[i], unsolved.squares[i]);
        }
    }

    #[test]
    fn test_solve_no_infinite_recursion() {
        let mut blank_board = Board::default();
        assert_eq!(None, blank_board.get_val((0, 0)).unwrap());
        assert!(blank_board.check_status().is_valid());
        let result = blank_board.solve_standard().is_solved();
        assert!(!result);
        assert_eq!(None, blank_board.get_val((0, 0)).unwrap());
    }

    #[test]
    fn test_invalid_solve() {
        let mut unsolvable = Board::from_string(INVALID_PUZZLE_STR);
        assert!(!unsolvable.solve_standard().is_solved());
    }

    #[test]
    fn test_solve_search() {
        let mut board = Board::from_string(HARD_PUZZLE_STR);
        assert!(board.solve_search());
    }

    #[test]
    fn test_solve_search_bad_mini() {
        let puzzle = "4,2,0,0,1,3,0,0,0,0,3,2,0,0,1,4";
        let mut board = Board::from_string(puzzle);
        assert!(!board.solve_search());
    }

    #[test]
    fn test_solve_search_parallel() {
        let mut board = Board::from_string(HARD_PUZZLE_STR);
        assert!(board.solve_search_parallel());
    }

    #[test]
    fn test_solve_search_parallel_bad_mini() {
        let puzzle = "4,2,0,0,1,3,0,0,0,0,3,2,0,0,1,4";
        let mut board = Board::from_string(puzzle);
        assert!(!board.solve_search_parallel());
    }


    #[test]
    fn test_is_valid() {
        let solved = Board::from_string(SOLVED_PUZZLE_STR);
        assert!(solved.check_status().is_valid());
        let unsolved = Board::from_string(UNSOLVED_PUZZLE_STR);
        assert!(unsolved.check_status().is_valid());
        let invalid = Board::from_string(INVALID_PUZZLE_STR);
        assert!(!invalid.check_status().is_valid());
    }



}

#[cfg(all(feature = "bench", test))]
mod bench {
    use super::*;
    use test::Bencher;

    const ll: &str = concat!(
        "0,0,0,7,0,0,0,0,5,0,0,0,0,0,0,0,",
        "0,0,0,1,0,0,0,0,0,0,0,0,5,0,0,0,",
        "0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,",
        "0,0,0,0,15,0,0,0,0,2,0,0,0,0,12,0,",
        "0,0,0,0,0,3,11,0,0,0,0,0,0,0,0,0,",
        "0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,",
        "0,0,0,0,0,0,0,6,0,0,0,12,0,0,0,0,",
        "0,0,0,0,10,0,0,9,0,16,0,6,0,4,0,0,",
        "12,0,0,0,0,0,0,0,0,15,0,0,0,0,0,6,",
        "0,0,16,0,0,0,0,0,0,0,0,0,0,0,0,0,",
        "0,0,0,0,0,1,0,0,0,0,0,8,0,0,0,0,",
        "0,0,0,0,0,0,0,0,0,0,0,0,12,0,0,0,",
        "0,0,0,0,11,13,0,0,0,0,0,0,0,0,16,0,",
        "0,0,0,0,0,0,0,0,0,13,0,0,0,0,0,0,",
        "0,0,0,15,2,0,0,0,0,0,0,0,11,0,0,3,",
        "0,0,13,0,0,0,0,15,0,0,0,0,0,0,0,4"
    );

    // #[ignore]
    #[bench]
    fn bench_16_solve_search_ll(b: &mut Bencher) {
        let mut unsolved = Board::from_string(ll);
        b.iter(|| { unsolved.solve_search(); });
    }

    // #[ignore]
    #[bench]
    fn bench_16_solve_search_ll_parallel(b: &mut Bencher) {
        let mut unsolved = Board::from_string(ll);
        b.iter(|| { unsolved.solve_search_parallel(); });
    }

    #[bench]
    fn bench_09_solve_standard(b: &mut Bencher) {
        let mut unsolved = Board::from_string(UNSOLVED_PUZZLE_STR);
        b.iter(|| { unsolved.solve_standard(); });
    }

    #[bench]
    fn bench_09_solve_ordered(b: &mut Bencher) {
        let mut unsolved = Board::from_string(UNSOLVED_PUZZLE_STR);
        b.iter(|| { unsolved.solve_ordered(); });
    }

    #[bench]
    fn bench_09_solve_parallel(b: &mut Bencher) {
        let mut unsolved = Board::from_string(UNSOLVED_PUZZLE_STR);
        b.iter(|| { unsolved.solve_parallel(); });
    }



    #[bench]
    fn bench_16_solve_standard(b: &mut Bencher) {
        let mut unsolved = Board::from_string(_UNSOLVED_16);
        b.iter(|| { unsolved.solve_standard(); });
    }

    #[bench]
    fn bench_16_solve_ordered(b: &mut Bencher) {
        let mut unsolved = Board::from_string(_UNSOLVED_16);
        b.iter(|| { unsolved.solve_ordered(); });
    }

    #[bench]
    fn bench_16_solve_parallel(b: &mut Bencher) {
        let mut unsolved = Board::from_string(_UNSOLVED_16);
        b.iter(|| { unsolved.solve_parallel(); });
    }


    #[bench]
    fn bench_09_blank_solve_standard(b: &mut Bencher) {
        let mut new_board = Board::from_base_num(3);
        b.iter(|| { new_board.solve_standard(); });
    }

    #[bench]
    fn bench_09_blank_solve_ordered(b: &mut Bencher) {
        let mut new_board = Board::from_base_num(3);
        b.iter(|| { new_board.solve_ordered(); });
    }

    #[bench]
    fn bench_09_blank_solve_parallel(b: &mut Bencher) {
        let mut new_board = Board::from_base_num(3);
        b.iter(|| { new_board.solve_parallel(); });
    }

    #[bench]
    fn bench_25_blank_solve_standard(b: &mut Bencher) {
        let mut new_board = Board::from_base_num(5);
        b.iter(|| { new_board.solve_standard(); });
    }

    #[bench]
    fn bench_25_blank_solve_ordered(b: &mut Bencher) {
        let mut new_board = Board::from_base_num(5);
        b.iter(|| { new_board.solve_ordered(); });
    }

    #[bench]
    fn bench_25_blank_solve_parallel(b: &mut Bencher) {
        let mut new_board = Board::from_base_num(5);
        b.iter(|| { new_board.solve_parallel(); });
    }

    #[ignore]
    #[bench]
    fn bench_36_blank_solve_standard(b: &mut Bencher) {
        let mut new_board = Board::from_base_num(6);
        b.iter(|| { new_board.solve_standard(); });
    }





    #[bench]
    fn bench_is_valid_09_unsolved(b: &mut Bencher) {
        let board = Board::from_string(UNSOLVED_PUZZLE_STR);
        b.iter(|| { let _is_valid = board.check_status().is_valid(); });
    }

    #[bench]
    fn bench_is_valid_09_solved(b: &mut Bencher) {
        let board = Board::from_string(SOLVED_PUZZLE_STR);
        b.iter(|| { let _is_valid = board.check_status().is_valid(); });
    }

    #[bench]
    fn bench_is_valid_09_blank(b: &mut Bencher) {
        let board = Board::from_base_num(3);
        b.iter(|| { let _is_valid = board.check_status().is_valid(); });
    }

    #[bench]
    fn bench_09_hard_solve_search(b: &mut Bencher) {
        let mut board = Board::from_string(HARD_PUZZLE_STR);
        b.iter(|| assert!(board.solve_search()));
    }

    #[bench]
    fn bench_09_hard_solve_search_parallel(b: &mut Bencher) {
        let mut board = Board::from_string(HARD_PUZZLE_STR);
        b.iter(|| assert!(board.solve_search_parallel()));
    }

    #[bench]
    fn bench_num_filled_09_blank(b: &mut Bencher) {
        let mut board = Board::from_base_num(3);
        b.iter(|| board.num_filled());
    }

    #[bench]
    fn bench_num_filled_09_unsolved(b: &mut Bencher) {
        let mut board = Board::from_string(UNSOLVED_PUZZLE_STR);
        b.iter(|| board.num_filled());
    }

    #[bench]
    fn bench_num_filled_09_solved(b: &mut Bencher) {
        let mut board = Board::from_string(SOLVED_PUZZLE_STR);
        b.iter(|| board.num_filled());
    }

    #[bench]
    fn bench_num_filled_16_blank(b: &mut Bencher) {
        let mut board = Board::from_base_num(4);
        b.iter(|| board.num_filled());
    }
}
