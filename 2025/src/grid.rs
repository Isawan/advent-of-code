#[derive(Debug, Clone)]
pub struct Grid<T> {
    letter: Vec<Vec<T>>,
    pub height: i32,
    pub width: i32,
}

impl<T: Copy> Grid<T> {
    pub fn new(letter: Vec<Vec<T>>) -> Self {
        let height = letter.len() as i32;
        let width = letter[0].len() as i32;
        Self {
            letter,
            width,
            height,
        }
    }
    pub fn get(&self, x: i32, y: i32) -> Option<T> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some(self.letter[y as usize][x as usize])
    }
    pub fn set(&mut self, x: i32, y: i32, s: T) -> Option<()> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        self.letter[y as usize][x as usize] = s;
        Some(())
    }
    pub fn all(&self) -> Vec<(i32, i32, T)> {
        let mut v = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                v.push((x, y, self.get(x, y).unwrap()));
            }
        }
        v
    }
}
