use board::*;
use std::collections::HashMap;


pub struct HintMap {
    pub hints: HashMap<usize, Vec<SquareType>>,
    pub side_length: usize,
}


impl HintMap {
    pub fn from_board(board: &Board) -> HintMap {
        let mut hints: HashMap<usize, Vec<SquareType>> = HashMap::new();
        let num_groups = board.side_length;

        //Store the computed values in order to save time
        let mut rows = HashMap::new();
        let mut cols = HashMap::new();
        let mut grids = HashMap::new();

        let vals: Vec<SquareType> = (board.min_value..board.max_value + 1).collect();

        'outer: for (pt, val) in board.squares.iter().enumerate() {
            if val.is_some() {
                continue 'outer;
            }
            //Create a vector full of all values
            let mut vals = vals.clone();

            let row_num = board.get_row_num(pt);
            let col_num = board.get_col_num(pt);
            let grid_num = board.get_grid_num(pt);

            if !rows.contains_key(&row_num) {
                rows.insert(row_num, board.get_row_values(row_num));
            }

            if !cols.contains_key(&col_num) {
                cols.insert(col_num, board.get_col_values(col_num));
            }

            if !grids.contains_key(&grid_num) {
                grids.insert(grid_num, board.get_grid_values(grid_num));
            }

            let row_vals = rows.get(&row_num).unwrap();
            let col_vals = cols.get(&col_num).unwrap();
            let grid_vals = grids.get(&grid_num).unwrap();

            'forloop: for i in 0..num_groups {
                if vals.len() == 0 {
                    break 'forloop;
                }
                if let Some(val) = row_vals[i] {
                    vals.retain(|&o| o != val);
                }
                if let Some(val) = col_vals[i] {
                    vals.retain(|&o| o != val);
                }
                if let Some(val) = grid_vals[i] {
                    vals.retain(|&o| o != val);
                }
            }
            hints.insert(pt, vals);
        }

        HintMap {
            hints,
            side_length: board.side_length,
        }
    }


    pub fn get_hints<T: Position>(&self, pt: T) -> Vec<SquareType> {
        let pt = pt.into_pos(self.side_length); //Make sure is index, not coords
        if let Some(hints) = self.hints.get(&pt) {
            hints.clone()
        } else {
            vec![]
        }
    }

    pub fn get_ordered(&self) -> Vec<(Pos, Vec<SquareType>)> {
        let mut retval: Vec<(Pos, Vec<SquareType>)> = self.hints
            .iter()
            .map(|(key, value)| (*key, value.clone()))
            .filter(|&(_, ref value)| value.len() > 0)
            .collect();
        retval.sort_by(|&(_, ref a1), &(_, ref a2)| a1.len().cmp(&a2.len()));

        retval
    }

    pub fn find_square_answer(&self, square_index: Pos) -> Option<SquareType> {
        let square_hints = self.hints.get(&square_index);

        if square_hints.is_none() {
            return None;
        }

        let square_hints = square_hints.unwrap();

        // This would also indicate that the board is invalid
        if square_hints.len() == 0 {
            return None;
        }

        // There is a single hint - this is the only possible value
        if square_hints.len() == 1 {
            return Some(square_hints[0]);
        }

        let side_length = self.side_length;

        // Solve Hidden Singles

        // Row
        let mut hints = square_hints.clone();
        let row_num = square_index.row_num(side_length);
        let row_indices = get_row_indices(row_num, side_length);
        for other_index in row_indices {
            if square_index != other_index {
                let other_hints = self.get_hints(other_index);
                for other_hint in other_hints {
                    if let Ok(hint_index) = hints.binary_search(&other_hint) {
                        hints.remove(hint_index);
                    }
                }
            }
        }
        if hints.len() == 1 {
            return Some(hints[0]);
        }

        // Column
        let mut hints = square_hints.clone();
        let col_num = square_index.col_num(side_length);
        let col_indices = get_col_indices(col_num, side_length);
        for other_index in col_indices {
            if square_index != other_index {
                let other_hints = self.get_hints(other_index);
                for other_hint in other_hints {
                    if let Ok(hint_index) = hints.binary_search(&other_hint) {
                        hints.remove(hint_index);
                    }
                }
            }
        }
        if hints.len() == 1 {
            return Some(hints[0]);
        }

        // Grid
        let base_num = (side_length as f64).sqrt() as usize;
        let mut hints = square_hints.clone();
        let grid_num = square_index.grid_num(side_length, base_num);
        let grid_indices = get_grid_indices(grid_num, side_length);
        for other_index in grid_indices {
            if square_index != other_index {
                let other_hints = self.get_hints(other_index);
                for other_hint in other_hints {
                    if let Ok(hint_index) = hints.binary_search(&other_hint) {
                        hints.remove(hint_index);
                    }
                }
            }
        }
        if hints.len() == 1 {
            return Some(hints[0]);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use board::*;
    use super::*;

    #[test]
    fn test_hint_map() {
        let unsolved = Board::from_string(UNSOLVED_PUZZLE_STR);
        let hmap = HintMap::from_board(&unsolved);
        assert_eq!(vec![9], hmap.get_hints(9));
    }
}


#[cfg(all(feature = "bench", test))]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_hint_map_09_unsolved(b: &mut Bencher) {
        let _unsolved = Board::from_string(UNSOLVED_PUZZLE_STR);
        b.iter(|| { let _hmap = HintMap::from_board(&_unsolved); });
    }


    #[bench]
    fn bench_hint_map_09_blank(b: &mut Bencher) {
        let _unsolved = Board::from_base_num(3);
        b.iter(|| { let _hmap = HintMap::from_board(&_unsolved); });
    }
}
