use crate::eval::score::*;

//PIECE WEIGHTS
pub const PAWN: Score = S!(126, 208);
pub const KNIGHT: Score = S!(781, 854);
pub const BISHOP: Score = S!(825, 915);
pub const ROOK: Score = S!(1276, 1380);
pub const QUEEN: Score = S!(2538, 2682);

//PSTs
pub const P: [Score; 64] = [
    S!(   0,   0), S!(   0,   0), S!(   0,   0), S!(   0,   0), S!(   0,   0), S!(   0,   0), S!(   0,   0), S!(   0,   0), 
    S!( -31, -31), S!(   8,   8), S!(  -7,  -7), S!( -37, -37), S!( -36, -36), S!( -14, -14), S!(   3,   3), S!( -31, -31), 
    S!( -22, -22), S!(   9,   9), S!(   5,   5), S!( -11, -11), S!( -10, -10), S!(  -2,  -2), S!(   3,   3), S!( -19, -19), 
    S!( -26, -26), S!(   3,   3), S!(  10,  10), S!(   9,   9), S!(   6,   6), S!(   1,   1), S!(   0,   0), S!( -23, -23), 
    S!( -17, -17), S!(  16,  16), S!(  -2,  -2), S!(  15,  15), S!(  14,  14), S!(   0,   0), S!(  15,  15), S!( -13, -13), 
    S!(   7,   7), S!(  29,  29), S!(  21,  21), S!(  44,  44), S!(  40,  40), S!(  31,  31), S!(  44,  44), S!(   7,   7), 
    S!(  78,  78), S!(  83,  83), S!(  86,  86), S!(  73,  73), S!( 102, 102), S!(  82,  82), S!(  85,  85), S!(  90,  90), 
    S!( 100, 100), S!( 100, 100), S!( 100, 100), S!( 100, 100), S!( 105, 105), S!( 100, 100), S!( 100, 100), S!( 100, 100)
];

pub const N: [Score; 64] = [
    S!( -74, -74), S!( -23, -23), S!( -26, -26), S!( -24, -24), S!( -19, -19), S!( -35, -35), S!( -22, -22), S!( -69, -69), 
    S!( -23, -23), S!( -15, -15), S!(   2,   2), S!(   0,   0), S!(   2,   2), S!(   0,   0), S!( -23, -23), S!( -20, -20), 
    S!( -18, -18), S!(  10,  10), S!(  13,  13), S!(  22,  22), S!(  18,  18), S!(  15,  15), S!(  11,  11), S!( -14, -14), 
    S!(  -1,  -1), S!(   5,   5), S!(  31,  31), S!(  21,  21), S!(  22,  22), S!(  35,  35), S!(   2,   2), S!(   0,   0), 
    S!(  24,  24), S!(  24,  24), S!(  45,  45), S!(  37,  37), S!(  33,  33), S!(  41,  41), S!(  25,  25), S!(  17,  17), 
    S!(  10,  10), S!(  67,  67), S!(   1,   1), S!(  74,  74), S!(  73,  73), S!(  27,  27), S!(  62,  62), S!(  -2,  -2), 
    S!(  -3,  -3), S!(  -6,  -6), S!( 100, 100), S!( -36, -36), S!(   4,   4), S!(  62,  62), S!(  -4,  -4), S!( -14, -14), 
    S!( -66, -66), S!( -53, -53), S!( -75, -75), S!( -75, -75), S!( -10, -10), S!( -55, -55), S!( -58, -58), S!( -70, -70)
];

pub const B: [Score; 64] = [
    S!(  -7,  -7), S!(   2,   2), S!( -15, -15), S!( -12, -12), S!( -14, -14), S!( -15, -15), S!( -10, -10), S!( -10, -10), 
    S!(  19,  19), S!(  20,  20), S!(  11,  11), S!(   6,   6), S!(   7,   7), S!(   6,   6), S!(  20,  20), S!(  16,  16), 
    S!(  14,  14), S!(  25,  25), S!(  24,  24), S!(  15,  15), S!(   8,   8), S!(  25,  25), S!(  20,  20), S!(  15,  15), 
    S!(  13,  13), S!(  10,  10), S!(  17,  17), S!(  23,  23), S!(  17,  17), S!(  16,  16), S!(   0,   0), S!(   7,   7), 
    S!(  25,  25), S!(  17,  17), S!(  20,  20), S!(  34,  34), S!(  26,  26), S!(  25,  25), S!(  15,  15), S!(  10,  10), 
    S!(  -9,  -9), S!(  39,  39), S!( -32, -32), S!(  41,  41), S!(  52,  52), S!( -10, -10), S!(  28,  28), S!( -14, -14), 
    S!( -11, -11), S!(  20,  20), S!(  35,  35), S!( -42, -42), S!( -39, -39), S!(  31,  31), S!(   2,   2), S!( -22, -22), 
    S!( -59, -59), S!( -78, -78), S!( -82, -82), S!( -76, -76), S!( -23, -23), S!(-107,-107), S!( -37, -37), S!( -50, -50)
];

pub const R: [Score; 64] = [
    S!( -30, -30), S!( -24, -24), S!( -18, -18), S!(   5,   5), S!(  -2,  -2), S!( -18, -18), S!( -31, -31), S!( -32, -32), 
    S!( -53, -53), S!( -38, -38), S!( -31, -31), S!( -26, -26), S!( -29, -29), S!( -43, -43), S!( -44, -44), S!( -53, -53), 
    S!( -42, -42), S!( -28, -28), S!( -42, -42), S!( -25, -25), S!( -25, -25), S!( -35, -35), S!( -26, -26), S!( -46, -46), 
    S!( -28, -28), S!( -35, -35), S!( -16, -16), S!( -21, -21), S!( -13, -13), S!( -29, -29), S!( -46, -46), S!( -30, -30), 
    S!(   0,   0), S!(   5,   5), S!(  16,  16), S!(  13,  13), S!(  18,  18), S!(  -4,  -4), S!(  -9,  -9), S!(  -6,  -6), 
    S!(  19,  19), S!(  35,  35), S!(  28,  28), S!(  33,  33), S!(  45,  45), S!(  27,  27), S!(  25,  25), S!(  15,  15), 
    S!(  55,  55), S!(  29,  29), S!(  56,  56), S!(  67,  67), S!(  55,  55), S!(  62,  62), S!(  34,  34), S!(  60,  60), 
    S!(  35,  35), S!(  29,  29), S!(  33,  33), S!(   4,   4), S!(  37,  37), S!(  33,  33), S!(  56,  56), S!(  50,  50)
];

pub const Q: [Score; 64] = [
    S!( -39, -39), S!( -30, -30), S!( -31, -31), S!( -13, -13), S!( -31, -31), S!( -36, -36), S!( -34, -34), S!( -42, -42), 
    S!( -36, -36), S!( -18, -18), S!(   0,   0), S!( -19, -19), S!( -15, -15), S!( -15, -15), S!( -21, -21), S!( -38, -38), 
    S!( -30, -30), S!(  -6,  -6), S!( -13, -13), S!( -11, -11), S!( -16, -16), S!( -11, -11), S!( -16, -16), S!( -27, -27), 
    S!( -14, -14), S!( -15, -15), S!(  -2,  -2), S!(  -5,  -5), S!(  -1,  -1), S!( -10, -10), S!( -20, -20), S!( -22, -22), 
    S!(   1,   1), S!( -16, -16), S!(  22,  22), S!(  17,  17), S!(  25,  25), S!(  20,  20), S!( -13, -13), S!(  -6,  -6), 
    S!(  -2,  -2), S!(  43,  43), S!(  32,  32), S!(  60,  60), S!(  72,  72), S!(  63,  63), S!(  43,  43), S!(   2,   2), 
    S!(  14,  14), S!(  32,  32), S!(  60,  60), S!( -10, -10), S!(  20,  20), S!(  76,  76), S!(  57,  57), S!(  24,  24), 
    S!(   6,   6), S!(   1,   1), S!(  -8,  -8), S!(-104,-104), S!(  69,  69), S!(  24,  24), S!(  88,  88), S!(  26,  26)
];

pub const K: [Score; 64] = [
    S!(  17, -50), S!(  30, -30), S!(  -3, -30), S!(  -14, -30), S!(   6, -30), S!(  -1, -30), S!(  40, -30), S!(  18, -50),
    S!(  -4, -30), S!(   3, -30), S!( -14,   0), S!(  -50,   0), S!( -57,   0), S!( -18,   0), S!(  13, -30), S!(   4, -30),
    S!( -47, -30), S!( -42, -10), S!( -43,  20), S!(  -79,  30), S!( -64,  30), S!( -32,  20), S!( -29, -10), S!( -32, -30),
    S!( -55, -30), S!( -43, -10), S!( -52,  30), S!(  -28,  40), S!( -51,  40), S!( -47,  30), S!(  -8, -10), S!( -50, -30),
    S!( -55, -30), S!(  50, -10), S!(  11,  30), S!(   -4,  40), S!( -19,  40), S!(  13,  30), S!(   0, -10), S!( -49, -30),
    S!( -62, -30), S!(  12, -10), S!( -57,  20), S!(   44,  30), S!( -67,  30), S!(  28,  20), S!(  37, -10), S!( -31, -30),
    S!( -32, -30), S!(  10, -20), S!(  55, -10), S!(   56,   0), S!(  56,   0), S!(  55, -10), S!(  10, -20), S!(   3, -30),
    S!(   4, -50), S!(  54, -40), S!(  47, -30), S!(  -99, -20), S!( -99, -20), S!(  60, -30), S!(  83, -40), S!( -62, -50)
];