extern crate rand;
use board::*;
use rand::Rng;


impl Board {
    pub fn rand_vals(&self) -> Vec<Option<SquareType>> {
        let num_vals = self.side_length;
        let mut retval = vec![];
        let vals_range = self.min_value..self.max_value + 1;
        let mut vals = vec![];
        for i in vals_range {
            vals.push(i);
        }
        for _ in 0..num_vals {
            let chosen_ind = rand::thread_rng().gen_range(0, vals.len());
            let val = vals.swap_remove(chosen_ind);
            retval.push(Some(val));
        }
        assert_eq!(num_vals, retval.len());
        retval
    }

    pub fn set_row_rand(&mut self, row_num: usize) {
        let rand_vals = self.rand_vals();
        self.set_row(row_num, rand_vals);
    }
    pub fn set_col_rand(&mut self, col_num: usize) {
        let rand_vals = self.rand_vals();
        self.set_col(col_num, rand_vals);
    }
    pub fn set_grid_rand(&mut self, grid_num: usize) {
        let rand_vals = self.rand_vals();
        self.set_grid(grid_num, rand_vals);
    }

    pub fn reduce(&mut self) {
        for _ in 0..self.squares.len() {
            let ind = rand::thread_rng().gen_range(0, self.squares.len());
            let old_val = self.squares[ind];
            if old_val != None {
                self.squares[ind] = None;
                if !self.is_solvable() {
                    self.squares[ind] = old_val;
                }
            }
        }
    }


    pub fn generate_new(base_num: usize) -> Result<Board, &'static str> {
        let mut new_board = Board::from_base_num(base_num);

        // step_by method is currently unstable, need to do this instead
        let mut i = 0;

        // Randomise 3 diagonal grids, as they are not dependent on each other
        while i < new_board.side_length {
            let grid_num = new_board.get_grid_num((i, i));
            new_board.set_grid_rand(grid_num);
            i += base_num;
        }

        let solve_result = new_board.solve_search_parallel();
        if !solve_result {
            return Err("Error generating board");
        }
        new_board.reduce();
        Ok(new_board)
    }

    pub fn _swap_rows(&mut self, row_ind_1: Pos, row_ind_2: Pos) {
        let row1_vals = self.get_row_values(row_ind_1);
        let row2_vals = self.get_row_values(row_ind_2);
        let row1_inds = self.get_row_indices(row_ind_1);
        let row2_inds = self.get_row_indices(row_ind_2);
        for i in 0..self.side_length {
            self.squares[row1_inds[i]] = row2_vals[i];
            self.squares[row2_inds[i]] = row1_vals[i];
        }
    }
    pub fn _swap_cols(&mut self, col_ind_1: Pos, col_ind_2: Pos) {
        let col1_vals = self.get_col_values(col_ind_1);
        let col2_vals = self.get_col_values(col_ind_2);
        let col1_inds = self.get_col_indices(col_ind_1);
        let col2_inds = self.get_col_indices(col_ind_2);
        for i in 0..self.side_length {
            self.squares[col1_inds[i]] = col2_vals[i];
            self.squares[col2_inds[i]] = col1_vals[i];
        }
    }

    pub fn _mirror(&mut self) {
        let num_groups = self.side_length;
        let mut cols = vec![];
        for i in 0..num_groups {
            cols.push(self.get_col_values(i));
        }
        for i in 0..num_groups {
            let thingo = cols.pop().unwrap();
            self.set_col(i, thingo);
        }
    }

    //Rotates anti-clockwise
    pub fn _rotate(&mut self) {
        self._mirror();
        let cloned = self.clone();
        for pos in 0..self.squares.len() {
            let (col, row) = pos.into_coord(self.side_length);
            self.set_val((row, col), cloned.get_val((col, row)).unwrap())
                .unwrap();
        }
    }
}


#[cfg(test)]
pub const SWAPPED_ROW_STR: &str = concat!(
    "9,6,5,3,2,7,1,4,8,",
    "8,2,7,1,5,4,3,9,6,",
    "3,4,1,6,8,9,7,5,2,",
    "5,9,3,4,6,8,2,7,1,",
    "4,7,2,5,1,3,6,8,9,",
    "6,1,8,9,7,2,4,3,5,",
    "7,8,6,2,3,5,9,1,4,",
    "1,5,4,7,9,6,8,2,3,",
    "2,3,9,8,4,1,5,6,7"
);

#[cfg(test)]
pub const SWAPPED_COL_STR: &str = concat!(
    "2,8,7,1,5,4,3,9,6,",
    "6,9,5,3,2,7,1,4,8,",
    "4,3,1,6,8,9,7,5,2,",
    "9,5,3,4,6,8,2,7,1,",
    "7,4,2,5,1,3,6,8,9,",
    "1,6,8,9,7,2,4,3,5,",
    "8,7,6,2,3,5,9,1,4,",
    "5,1,4,7,9,6,8,2,3,",
    "3,2,9,8,4,1,5,6,7"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_generated_no_blanks() {
        let board = Board::generate_new(3).expect("Error generating board");
        for i in 0..NUM_SQUARES {
            assert_ne!(None, board.squares[i]);
        }
    }

    #[test]
    fn test_swap_rows() {
        let mut board = Board::from_string(SOLVED_PUZZLE_STR);
        assert_eq!(SOLVED_PUZZLE_STR, board.to_string());
        assert!(board.check_status().is_valid());
        board._swap_rows(0, 1);
        assert_ne!(SOLVED_PUZZLE_STR, board.to_string());
        assert_eq!(SWAPPED_ROW_STR, board.to_string());
    }

    #[test]
    fn test_swap_cols() {
        let mut board = Board::from_string(SOLVED_PUZZLE_STR);
        assert_eq!(SOLVED_PUZZLE_STR, board.to_string());
        assert!(board.check_status().is_valid());
        board._swap_cols(0, 1);
        assert_ne!(SOLVED_PUZZLE_STR, board.to_string());
        assert_eq!(SWAPPED_COL_STR, board.to_string());
    }
}
