const CHARACTERS: [char; 83] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', '#', '$', '%', '*', '+', ',', '-', '.', ':', ';', '=', '?', '@', '[',
    ']', '^', '_', '{', '|', '}', '~',
];

pub fn encode83(value: u32, length: usize, destination: &mut String) {
    let mut divisor = 1;

    for _ in 0..(length - 1) {
        divisor *= 83;
    }

    for _ in 0..length {
        let digit = (value / divisor) % 83;
        divisor /= 83;
        destination.push(CHARACTERS[digit as usize]);
    }
}
