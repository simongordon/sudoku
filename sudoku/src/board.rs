
pub type Pos = usize;
//Note: we always use: (Column, Row)
pub type Coord = (usize, usize);
pub type SquareType = i32;

const DEFAULT_BASE_NUM: usize = 3;

// This trait allows us to call certain functions with either the index of an element,
// or its coordinates. Custom row/column/grid functions are also implemented here for
// faster mathematics.
pub trait Position {
    fn into_coord(self, width: usize) -> Coord;
    fn into_pos(self, width: usize) -> Pos;

    fn row_num(self, width: usize) -> usize;
    fn col_num(self, width: usize) -> usize;
    fn grid_num(self, width: usize, base_num: usize) -> usize;
}

impl Position for Pos {
    fn into_coord(self, width: usize) -> Coord {
        let row_num = self.row_num(width);
        let col_num = self.col_num(width);
        (col_num, row_num)
    }
    fn into_pos(self, _width: usize) -> Pos {
        self
    }


    fn row_num(self, width: usize) -> usize {
        self / width
    }

    fn col_num(self, width: usize) -> usize {
        self % width
    }

    fn grid_num(self, width: usize, base_num: usize) -> usize {
        self.into_coord(width).grid_num(width, base_num)
    }
}

impl Position for Coord {
    fn into_coord(self, _width: usize) -> Coord {
        self
    }
    fn into_pos(self, width: usize) -> Pos {
        let (x, y) = self;
        (width * y) + x
    }

    fn row_num(self, _width: usize) -> usize {
        let (_, row_num) = self;
        row_num
    }

    fn col_num(self, _width: usize) -> usize {
        let (col_num, _) = self;
        col_num
    }

    fn grid_num(self, width: usize, base_num: usize) -> usize {
        let (col_num, row_num) = self;
        (base_num * (row_num / base_num)) + (col_num / base_num) + (col_num / width)
    }
}

pub fn get_row_indices(row_num: usize, width: usize) -> Vec<Pos> {
    (0..width)
        .map(|col_num| (col_num, row_num).into_pos(width))
        .collect()
}

pub fn get_col_indices(col_num: usize, width: usize) -> Vec<Pos> {
    (0..width)
        .map(|row_num| (col_num, row_num).into_pos(width))
        .collect()
}

// Not as elegant as the other two functions...
pub fn get_grid_indices(grid_num: usize, width: usize) -> Vec<Pos> {
    let base_num = (width as f64).sqrt() as usize;

    //Convert the grid num into coordinates
    let (grid_col, grid_row) = grid_num.into_coord(base_num);
    let (start_col, start_row) = (grid_col * base_num, grid_row * base_num);
    let (end_col, end_row) = (start_col + base_num, start_row + base_num);

    let mut indices: Vec<Pos> = Vec::new();
    for row in start_row..end_row {
        for col in start_col..end_col {
            indices.push((col, row).into_pos(width));
        }
    }

    indices
}

#[derive(Clone)]
pub struct Board {
    pub squares: Vec<Option<SquareType>>,

    pub num_squares: usize,
    pub side_length: usize,
    pub min_value: SquareType,
    pub max_value: SquareType,
    pub base_num: usize,
}

impl Default for Board {
    fn default() -> Board {
        Board::from_base_num(DEFAULT_BASE_NUM)
    }
}

/// Allow for the comparison of boards
impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        for index in 0..self.squares.len() {
            if self.squares[index] != other.squares[index] {
                return false;
            }
        }
        true
    }
}

impl ToString for Board {
    fn to_string(&self) -> String {
        self.squares
            .iter()
            .map(|o| {
                match *o {
                    Some(val) => val,
                    None => 0,
                }.to_string()
            })
            .collect::<Vec<String>>()
            .join(",")
    }
}

impl Board {
    pub fn from_base_num(base_num: usize) -> Board {
        let squared = base_num * base_num;
        Board::from_size(squared * squared)
    }

    pub fn from_size(num_squares: Pos) -> Board {
        let side_length = (num_squares as f64).sqrt() as Pos;
        let max_value = (side_length) as SquareType;
        //TODO: Need to use from_base_num instead of this function anyways
        let base_num = (max_value as f64).sqrt() as usize;
        Board {
            //Initialises NUM_SQUARES many of "None" values
            squares: vec![None; num_squares],
            side_length,
            num_squares,
            min_value: 1,
            max_value,
            base_num,
        }
    }

    /// Create a board based on a string
    pub fn from_string(s: &str) -> Board {
        //TODO: Should probably make this a result
        //Strip whitespace
        let s: String = s.split_whitespace().collect::<String>().to_lowercase();

        let bytes: Vec<&str> = if s.contains(",") {
            s.split(',').collect::<Vec<&str>>()
        } else {
            //Otherwise, split all chars
            s.split("").collect::<Vec<&str>>()
        }.into_iter()
            .filter(|x| x.len() > 0)
            .collect();
        let mut game_board = Board::from_size(bytes.len());

        for i in 0..game_board.squares.len() {
            game_board.squares[i] = match bytes[i].parse::<SquareType>() {
                Ok(num) => {
                    if num >= game_board.min_value && num <= game_board.max_value {
                        Some(num)
                    } else {
                        None
                    }
                }
                Err(_) => {
                    //Not a valid number - TODO: Maybe continue?
                    None
                }
            };
        }
        game_board
    }


    /// Get the row num of a square within this board
    pub fn get_row_num<T: Position>(&self, pt: T) -> Pos {
        pt.row_num(self.side_length)
    }

    /// Get the col num of a square within this board
    pub fn get_col_num<T: Position>(&self, pt: T) -> Pos {
        pt.col_num(self.side_length)
    }

    /// Get the grid num of a square within this board
    pub fn get_grid_num<T: Position>(&self, pt: T) -> Pos {
        pt.grid_num(self.side_length, self.base_num)
    }

    /// Try to get a value
    pub fn get_val<T: Position>(&self, pos: T) -> Result<Option<SquareType>, &'static str> {
        let index = pos.into_pos(self.side_length);
        if index < self.num_squares {
            Ok(self.squares[index])
        } else {
            Err("Index out of range!")
        }
    }

    /// Try to set a value
    pub fn set_val<T: Position>(
        &mut self,
        pos: T,
        val: Option<SquareType>,
    ) -> Result<(), &'static str> {
        let index = pos.into_pos(self.side_length);
        self.squares[index] = {
            if index < self.num_squares {
                match val {
                    Some(val) if val > self.max_value || val < self.min_value => {
                        return Err("Invalid value!");
                    }
                    _ => val,
                }
            } else {
                return Err("Index out of range!");
            }
        };
        Ok(())
    }

    /// Get all indicies of squares within a column
    pub fn get_col_indices(&self, col_num: usize) -> Vec<Pos> {
        get_col_indices(col_num, self.side_length)
    }

    /// Gets all values in this column
    pub fn get_col_values(&self, col_num: usize) -> Vec<Option<SquareType>> {
        let width = self.side_length;
        (0..width)
            .map(|row_num| self.squares[(col_num, row_num).into_pos(width)])
            .collect()
    }


    /// Get all indicies of squares within a row
    pub fn get_row_indices(&self, row_num: usize) -> Vec<Pos> {
        get_row_indices(row_num, self.side_length)
    }

    /// Gets all values in this row
    pub fn get_row_values(&self, row_num: usize) -> Vec<Option<SquareType>> {
        let width = self.side_length;
        (0..width)
            .map(|col_num| self.squares[(col_num, row_num).into_pos(width)])
            .collect()
    }

    /// Get all indicies of squares within a grid
    pub fn get_grid_indices(&self, grid_num: usize) -> Vec<Pos> {
        get_grid_indices(grid_num, self.side_length)
    }

    /// Gets all values in this grid
    pub fn get_grid_values(&self, grid_num: usize) -> Vec<Option<SquareType>> {
        self.get_grid_indices(grid_num)
            .iter()
            .map(|pos| self.get_val(*pos).unwrap())
            .collect()
    }

    /// Set all values within a row
    pub fn set_row(&mut self, row_num: usize, vals: Vec<Option<SquareType>>) {
        let row_inds = self.get_row_indices(row_num);
        for (i, ind) in row_inds.iter().enumerate() {
            let val = vals[i];
            self.set_val(*ind, val).unwrap();
        }
    }

    /// Set all values within a column
    pub fn set_col(&mut self, col_num: usize, vals: Vec<Option<SquareType>>) {
        let col_inds = self.get_col_indices(col_num);
        for (i, ind) in col_inds.iter().enumerate() {
            let val = vals[i];
            self.set_val(*ind, val).unwrap();
        }
    }

    /// Set all values within a grid
    pub fn set_grid(&mut self, grid_num: usize, vals: Vec<Option<SquareType>>) {
        let grid_inds = self.get_grid_indices(grid_num);
        for (i, ind) in grid_inds.iter().enumerate() {
            let val = vals[i];
            self.set_val(*ind, val).unwrap();
        }
    }

    /// Number of squares with values
    pub fn num_filled(&self) -> usize {
        self.squares.iter().filter(|o| o.is_some()).count()
    }

    /// Number of squares without values
    pub fn num_remaining(&self) -> usize {
        self.squares.iter().filter(|o| o.is_none()).count()
    }
}


#[cfg(test)]
pub const BASE_NUM: Pos = 3;
#[cfg(test)]
pub const GROUP_NUM: Pos = BASE_NUM * BASE_NUM;
#[cfg(test)]
pub const NUM_SQUARES: Pos = GROUP_NUM * GROUP_NUM;


#[cfg(test)]
pub const SOLVED_PUZZLE_STR: &str = concat!(
    "8,2,7,1,5,4,3,9,6,",
    "9,6,5,3,2,7,1,4,8,",
    "3,4,1,6,8,9,7,5,2,",
    "5,9,3,4,6,8,2,7,1,",
    "4,7,2,5,1,3,6,8,9,",
    "6,1,8,9,7,2,4,3,5,",
    "7,8,6,2,3,5,9,1,4,",
    "1,5,4,7,9,6,8,2,3,",
    "2,3,9,8,4,1,5,6,7"
);


pub const UNSOLVED_PUZZLE_STR: &str = concat!(
    "8,2,7,1,0,4,0,0,6,",
    "0,6,5,3,2,7,1,4,8,",
    "0,4,1,6,0,0,7,5,0,",
    "5,0,3,4,6,0,0,7,1,",
    "0,7,2,5,1,0,6,0,9,",
    "6,1,8,0,0,0,4,0,5,",
    "7,0,6,0,3,5,0,0,0,",
    "0,5,4,7,0,6,0,2,3,",
    "2,3,0,8,4,1,5,0,7"
);

#[cfg(test)]
pub const _UNSOLVED_16: &str = concat!(
    "4,11,0,0,13,0,0,2,3,9,0,0,5,0,0,0,",
    "3,0,0,9,0,14,0,4,11,15,0,0,0,0,0,0,",
    "16,0,0,0,7,0,6,1,0,4,0,13,0,3,0,0,",
    "8,0,0,15,0,9,0,11,0,7,0,2,0,0,0,16,",
    "15,12,0,0,0,5,2,0,14,1,0,0,11,4,7,0,",
    "0,0,0,13,0,0,3,0,4,2,7,0,0,0,16,10,",
    "14,0,9,0,0,6,0,7,15,10,0,16,3,0,5,0,",
    "7,0,0,3,0,0,9,16,5,0,11,6,0,0,8,0,",
    "0,0,16,0,14,1,13,0,0,0,0,0,0,0,15,0,",
    "0,0,7,5,11,16,0,0,0,0,0,10,2,0,14,0,",
    "1,0,8,0,10,4,0,0,0,0,0,15,6,0,0,5,",
    "6,9,4,14,0,0,0,0,0,0,12,11,7,0,0,3,",
    "11,13,15,1,2,0,0,0,0,6,8,12,16,0,0,0,",
    "0,0,0,6,0,10,0,0,9,11,4,3,0,0,13,0,",
    "0,4,12,0,0,3,0,13,0,14,5,0,10,15,0,1,",
    "0,0,3,0,0,0,14,0,0,13,0,0,4,0,0,2"
);

#[cfg(test)]
pub const INVALID_PUZZLE_STR: &str = concat!(
    "8,2,7,1,5,4,3,9,6,",
    "0,6,5,3,2,7,1,4,8,",
    "3,4,1,6,8,9,7,5,2,",
    "5,9,3,4,6,8,2,7,1,",
    "4,7,2,5,1,3,4,8,9,",
    "6,1,8,9,7,2,4,3,5,",
    "7,8,6,2,3,5,9,1,4,",
    "1,5,4,7,9,6,8,2,3,",
    "2,3,9,8,4,1,5,6,7"
);

#[cfg(test)]
pub const EMPTY_PUZZLE_STR: &str = concat!(
    "0,0,0,0,0,0,0,0,0,",
    "0,0,0,0,0,0,0,0,0,",
    "0,0,0,0,0,0,0,0,0,",
    "0,0,0,0,0,0,0,0,0,",
    "0,0,0,0,0,0,0,0,0,",
    "0,0,0,0,0,0,0,0,0,",
    "0,0,0,0,0,0,0,0,0,",
    "0,0,0,0,0,0,0,0,0,",
    "0,0,0,0,0,0,0,0,0"
);

#[cfg(test)]
pub const HARD_PUZZLE_STR: &str = concat!(
    "0,0,0,0,3,0,8,0,9,",
    "0,6,0,9,0,0,5,3,0,",
    "0,0,0,0,0,0,0,6,0,",
    "0,4,0,0,0,1,0,0,0,",
    "5,0,8,0,0,0,4,0,1,",
    "0,0,0,7,0,0,0,2,0,",
    "0,2,0,0,0,0,0,0,0,",
    "0,1,7,0,0,6,0,4,0,",
    "8,0,5,0,7,0,0,0,0"
);

#[cfg(test)]
mod tests {
    use super::*;

    fn point_to_coord(pt: Pos) -> Coord {
        (pt).into_coord(GROUP_NUM)
    }
    fn coord_to_point(crd: Coord) -> Pos {
        (crd).into_pos(GROUP_NUM)
    }

    fn get_col_num(pt: Pos) -> Pos {
        let (col_num, _) = (pt).into_coord(GROUP_NUM);
        col_num
    }
    fn get_row_num(pt: Pos) -> Pos {
        let (_, row_num) = (pt).into_coord(GROUP_NUM);
        row_num
    }

    #[test]
    fn test_point_to_coord() {
        assert_eq!((0, 0), point_to_coord(0));
        assert_eq!((1, 0), point_to_coord(1));
        assert_eq!((8, 0), point_to_coord(8));

        assert_eq!((0, 1), point_to_coord(9));
        assert_eq!((1, 1), point_to_coord(10));

        assert_eq!((8, 8), point_to_coord(80));
    }

    #[test]
    fn test_coord_to_point() {
        assert_eq!(0, coord_to_point((0, 0)));
        assert_eq!(1, coord_to_point((1, 0)));
        assert_eq!(2, coord_to_point((2, 0)));

        assert_eq!(9, coord_to_point((0, 1)));
        assert_eq!(10, coord_to_point((1, 1)));
        assert_eq!(11, coord_to_point((2, 1)));

        assert_eq!(80, coord_to_point((8, 8)));
    }

    #[test]
    fn test_get_row_num() {
        assert_eq!(0, get_row_num(0));
        assert_eq!(0, get_row_num(1));
        assert_eq!(0, get_row_num(2));
        assert_eq!(0, get_row_num(8));

        assert_eq!(1, get_row_num(9));
        assert_eq!(2, get_row_num(18));
        assert_eq!(3, get_row_num(27));

        assert_eq!(7, get_row_num(63));
        assert_eq!(8, get_row_num(72));
        assert_eq!(8, get_row_num(80));
    }

    #[test]
    fn test_get_col_num() {
        assert_eq!(0, get_col_num(0));
        assert_eq!(1, get_col_num(1));
        assert_eq!(2, get_col_num(2));
        assert_eq!(3, get_col_num(3));
        assert_eq!(4, get_col_num(4));
        assert_eq!(5, get_col_num(5));
        assert_eq!(6, get_col_num(6));
        assert_eq!(7, get_col_num(7));
        assert_eq!(8, get_col_num(8));

        assert_eq!(0, get_col_num(9));
        assert_eq!(1, get_col_num(10));

        assert_eq!(8, get_col_num(80));
    }

    #[test]
    fn test_board_length() {
        let new_board = Board::default();
        match new_board.base_num {
            2 => {
                assert_eq!(16, new_board.squares.len());
            }
            3 => {
                assert_eq!(81, new_board.squares.len());
            }
            4 => {
                assert_eq!(256, new_board.squares.len());
            }
            _ => {
                panic!("Bad board length!");
            }
        };
    }

    #[test]
    fn test_board_cleared() {
        let new_board = Board::default();
        for i in 0..new_board.base_num {
            assert_eq!(None, new_board.squares[i]);
            assert_ne!(Some(0), new_board.squares[i]);
        }
    }

    #[test]
    fn test_board_from_string() {
        let puzzle_string = String::from(SOLVED_PUZZLE_STR);
        let new_board = Board::from_string(&puzzle_string);

        assert_eq!(Some(8), new_board.squares[0]);
        assert_eq!(Some(2), new_board.squares[1]);
        assert_eq!(Some(1), new_board.squares[3]);
        assert_eq!(Some(9), new_board.squares[9]);
        assert_eq!(Some(2), new_board.squares[13]);
        assert_eq!(Some(7), new_board.squares[80]);

        assert_ne!(Some(0), new_board.squares[0]);
        assert_ne!(Some(80), new_board.squares[80]);
        assert_ne!(None, new_board.squares[0]);
        assert_ne!(None, new_board.squares[80]);
    }

    #[test]
    fn test_board_from_string_blanks() {
        // There is an extra zero in here.
        let puzzle_string = String::from(UNSOLVED_PUZZLE_STR);
        let new_board = Board::from_string(&puzzle_string);

        assert_eq!(Some(8), new_board.squares[0]);
        assert_eq!(Some(2), new_board.squares[1]);

        assert_eq!(None, new_board.squares[9]);
        assert_ne!(Some(0), new_board.squares[9]);

        assert_eq!(Some(7), new_board.squares[80]);

        assert_ne!(Some(0), new_board.squares[0]);
        assert_ne!(Some(80), new_board.squares[80]);
        assert_ne!(None, new_board.squares[0]);
        assert_ne!(None, new_board.squares[80]);
    }

    #[test]
    fn test_board_to_string() {
        let solved_board = Board::from_string(SOLVED_PUZZLE_STR);
        assert_eq!(String::from(SOLVED_PUZZLE_STR), solved_board.to_string());
        let unsolved_board = Board::from_string(UNSOLVED_PUZZLE_STR);
        assert_eq!(
            String::from(UNSOLVED_PUZZLE_STR),
            unsolved_board.to_string()
        );
        let blank_board = Board::from_string(EMPTY_PUZZLE_STR);
        assert_eq!(String::from(EMPTY_PUZZLE_STR), blank_board.to_string());
    }

    #[test]
    fn test_multiple_boards() {
        let puzzle_string_1 = String::from(SOLVED_PUZZLE_STR);
        let board_1 = Board::from_string(&puzzle_string_1);

        let puzzle_string_2 = String::from(
            "807154396965327148341689752593468271472513689618972435786235914154796823239841567",
        );
        let mut board_2 = Board::from_string(&puzzle_string_2);

        assert_eq!(Some(8), board_1.squares[0]);
        assert_eq!(Some(2), board_1.squares[1]);

        assert_eq!(Some(8), board_2.squares[0]);
        assert_eq!(None, board_2.squares[1]);

        assert_ne!(board_1.squares[1], board_2.squares[1]);

        board_2.squares[1] = Some(2);
        assert_eq!(board_1.squares[1], board_2.squares[1]);
    }

    #[test]
    fn test_get_coord() {
        let puzzle_string = String::from(UNSOLVED_PUZZLE_STR);
        let new_board = Board::from_string(&puzzle_string);

        assert_eq!(Some(8), new_board.get_val((0, 0)).unwrap());
        assert_eq!(Some(2), new_board.get_val((1, 0)).unwrap());

        assert_eq!(None, new_board.get_val((0, 1)).unwrap());
        //assert_eq!(None, new_board.squares[9]);

        //assert_eq!(Some(7), new_board.get_coord(8, 8));
    }

    #[test]
    fn test_get_col() {
        let puzzle_string = String::from(SOLVED_PUZZLE_STR);
        let new_board = Board::from_string(&puzzle_string);

        let col = new_board.get_col_values(0);
        assert_eq!(col[0], new_board.squares[0]);
        assert_eq!(col[1], new_board.squares[9]);
        assert_eq!(col[2], new_board.squares[18]);
        assert_eq!(col[3], new_board.squares[27]);
        assert_eq!(col[4], new_board.squares[36]);
        assert_eq!(col[5], new_board.squares[45]);
        assert_eq!(col[6], new_board.squares[54]);
        assert_eq!(col[7], new_board.squares[63]);
        assert_eq!(col[8], new_board.squares[72]);
    }

    #[test]
    fn test_get_row() {
        let puzzle_string = String::from(SOLVED_PUZZLE_STR);
        let new_board = Board::from_string(&puzzle_string);

        let row = new_board.get_row_values(0);
        assert_eq!(row[0], new_board.squares[0]);
        assert_eq!(row[1], new_board.squares[1]);
        assert_eq!(row[2], new_board.squares[2]);
        assert_eq!(row[3], new_board.squares[3]);
        assert_eq!(row[4], new_board.squares[4]);
        assert_eq!(row[5], new_board.squares[5]);
        assert_eq!(row[6], new_board.squares[6]);
        assert_eq!(row[7], new_board.squares[7]);
        assert_eq!(row[8], new_board.squares[8]);

        let row = new_board.get_row_values(8);
        assert_eq!(row[0], new_board.squares[72]);
        assert_eq!(row[8], new_board.squares[80]);
    }

    #[test]
    fn test_get_grid() {
        let puzzle_string = String::from(SOLVED_PUZZLE_STR);
        let new_board = Board::from_string(&puzzle_string);

        let grid = new_board.get_grid_values(0);
        assert_eq!(grid[0], new_board.squares[0]);
        assert_eq!(grid[1], new_board.squares[1]);
        assert_eq!(grid[2], new_board.squares[2]);

        assert_eq!(grid[3], new_board.squares[9]);
        assert_eq!(grid[4], new_board.squares[10]);
        assert_eq!(grid[5], new_board.squares[11]);

        assert_eq!(grid[6], new_board.squares[18]);
        assert_eq!(grid[7], new_board.squares[19]);
        assert_eq!(grid[8], new_board.squares[20]);

        let grid = new_board.get_grid_values(3);
        assert_eq!(grid[0], new_board.squares[27]);
    }

    // #[ignore]
    #[test]
    fn test_indicies_values() {
        let puzzle_string = String::from(SOLVED_PUZZLE_STR);
        let board = Board::from_string(&puzzle_string);
        for i in 0..board.side_length {
            let row_inds = board.get_row_indices(i);
            let row_vals = board.get_row_values(i);
            let col_inds = board.get_col_indices(i);
            let col_vals = board.get_col_values(i);
            let grid_inds = board.get_grid_indices(i);
            let grid_vals = board.get_grid_values(i);
            for j in 0..board.side_length {
                let ind = row_inds[j];
                assert_eq!(row_vals[j], board.squares[ind]);
            }
            for j in 0..board.side_length {
                let ind = col_inds[j];
                assert_eq!(col_vals[j], board.squares[ind]);
            }
            for j in 0..board.side_length {
                let ind = grid_inds[j];
                assert_eq!(grid_vals[j], board.squares[ind]);
            }
        }
    }

    #[test]
    fn test_set_coord() {
        let mut solved_board = Board::from_string(&SOLVED_PUZZLE_STR);
        let mut unsolved_board = Board::from_string(&UNSOLVED_PUZZLE_STR);

        for i in 0..solved_board.num_squares {
            let (x, y) = (i).into_coord(solved_board.side_length);
            let val = solved_board.get_val((x, y)).unwrap();
            solved_board.set_val((x, y), val).unwrap();
            unsolved_board.set_val((x, y), val).unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn test_set_coord_invalid_0() {
        let mut new_board = Board::from_string(&SOLVED_PUZZLE_STR);
        for i in 0..new_board.num_squares {
            new_board.set_val(i, Some(0)).unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn test_set_coord_invalid_10() {
        let mut new_board = Board::from_string(&SOLVED_PUZZLE_STR);
        for i in 0..new_board.num_squares {
            new_board.set_val(i, Some(10)).unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn test_set_coord_invalid_neg1() {
        let mut new_board = Board::from_string(&SOLVED_PUZZLE_STR);
        for i in 0..NUM_SQUARES {
            new_board.set_val(i, Some(10)).unwrap();
        }
    }
}


#[cfg(all(feature = "bench", test))]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_from_string_normal(b: &mut Bencher) {
        b.iter(|| {
            let _new_board = Board::from_string(UNSOLVED_PUZZLE_STR);
        });
    }

    #[bench]
    fn bench_from_string_large(b: &mut Bencher) {
        b.iter(|| { let _new_board = Board::from_string(_UNSOLVED_16); });
    }

    #[bench]
    fn bench_to_string_normal(b: &mut Bencher) {
        let _new_board = Board::from_string(UNSOLVED_PUZZLE_STR);
        b.iter(|| { let _str = _new_board.to_string(); });
    }

    #[bench]
    fn bench_to_string_large(b: &mut Bencher) {
        let _new_board = Board::from_string(_UNSOLVED_16);
        b.iter(|| { let _str = _new_board.to_string(); });
    }
}
