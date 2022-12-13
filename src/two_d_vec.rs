const CARDINAL_DIRECTIONS: [PointDirection; 4] = [
    PointDirection::Down,
    PointDirection::Left,
    PointDirection::Right,
    PointDirection::Up,
];

#[derive(Debug, Clone, Copy)]
pub struct BoundedPoint {
    pub x: usize,
    pub y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl BoundedPoint {
    pub fn into_iter_direction(self, point_direction: PointDirection) -> BoundedPointIntoIterator {
        BoundedPointIntoIterator {
            point: self,
            direction: point_direction,
        }
    }

    pub fn into_iter_cardinal_adjacent(self) -> CardinalAdjacentIterator {
        CardinalAdjacentIterator {
            point: self,
            index: 0,
        }
    }

    pub fn get_adjacent(self, point_direction: &PointDirection) -> Option<BoundedPoint> {
        match point_direction {
            PointDirection::Up => {
                if self.y > 0 {
                    Some(BoundedPoint {
                        x: self.x,
                        y: self.y - 1,
                        max_x: self.max_x,
                        max_y: self.max_y,
                    })
                } else {
                    None
                }
            }
            PointDirection::Down => {
                if self.y < self.max_y {
                    Some(BoundedPoint {
                        x: self.x,
                        y: self.y + 1,
                        max_x: self.max_x,
                        max_y: self.max_y,
                    })
                } else {
                    None
                }
            }
            PointDirection::Left => {
                if self.x > 0 {
                    Some(BoundedPoint {
                        x: self.x - 1,
                        y: self.y,
                        max_x: self.max_x,
                        max_y: self.max_y,
                    })
                } else {
                    None
                }
            }
            PointDirection::Right => {
                if self.x < self.max_x {
                    Some(BoundedPoint {
                        x: self.x + 1,
                        y: self.y,
                        max_x: self.max_x,
                        max_y: self.max_y,
                    })
                } else {
                    None
                }
            }
        }
    }
}

pub enum PointDirection {
    Up,
    Down,
    Left,
    Right,
}

pub struct BoundedPointIntoIterator {
    point: BoundedPoint,
    direction: PointDirection,
}

impl Iterator for BoundedPointIntoIterator {
    type Item = BoundedPoint;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.point.get_adjacent(&self.direction);
        result.iter().for_each(|point| self.point = *point);
        result
    }
}

pub struct CardinalAdjacentIterator {
    point: BoundedPoint,
    index: usize,
}

impl Iterator for CardinalAdjacentIterator {
    type Item = BoundedPoint;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= CARDINAL_DIRECTIONS.len() {
            return None;
        }
        let mut result = self.point.get_adjacent(&CARDINAL_DIRECTIONS[self.index]);
        self.index += 1;

        result = match result {
            None => self.next(),
            _ => result,
        };
        result
    }
}
