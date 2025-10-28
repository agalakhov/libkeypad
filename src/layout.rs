#[rustfmt::skip]
const LAYOUT: [[[u8; 3]; 4]; 2] = [
    // Left keypad
    [
        [ b'A', b'B', b'C', ],
        [ b'D', b'E', b'F', ],
        [ b'G', b'H', b'I', ],
        [ b'J', b'K', b'L', ],
    ],

    // Right keypad
    [
        [ b'1', b'2', b'3', ],
        [ b'4', b'5', b'6', ],
        [ b'7', b'8', b'9', ],
        [ b'*', b'0', b'#', ],
    ],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol(u8);

impl Symbol {
    #[inline]
    pub fn chr(&self) -> u8 {
        self.0
    }

    #[inline]
    pub fn is_power(&self) -> bool {
        self.chr() == b'J'
    }
}

pub fn translate(pad: usize, row: usize, column: usize) -> Symbol {
    Symbol(LAYOUT[pad][row][column])
}
