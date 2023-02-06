use crate::eval::score::*;
 
//PIECE WEIGHTS
pub const PAWN: Score = S!(  72,   97);
pub const KNIGHT: Score = S!( 327,  323);
pub const BISHOP: Score = S!( 346,  347);
pub const ROOK: Score = S!( 519,  518);
pub const QUEEN: Score = S!(1029, 1028);
 
//EXTRA CALCS
pub const TEMPO: Score = S!(  19,   20);

pub const PAWN_MOBILITY: [Score; 5] = [
    S!(   3,    5), S!(   2,   -2), S!(   0,    1), S!(   2,    0), S!(  -7,   -6), 
];
pub const KNIGHT_MOBILITY: [Score; 9] = [
    S!(  -8,    4), S!(   1,    0), S!(  -1,   -1), S!(   2,    1), S!(   3,    2), S!(  -3,   -2), S!(  -1,    0), S!(  -1,    0), 
    S!(   6,    1), 
];
pub const BISHOP_MOBILITY: [Score; 14] = [
    S!(   0,    0), S!(  -1,    0), S!( -43,  -35), S!( -14,  -14), S!(  -7,   -7), S!(  -6,   -6), S!(   1,    0), S!(   2,    0), 
    S!(  -1,    0), S!(  -1,    0), S!(  11,    6), S!(   1,    1), S!(  14,   -3), S!(  19,   -5), 
];
pub const ROOK_MOBILITY: [Score; 15] = [
    S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(-202, -133), S!( -16,  -15), S!(   1,    0), 
    S!( -10,   -5), S!( -11,   -8), S!( -12,   -9), S!( -10,  -11), S!(  -3,   -3), S!(   4,    4), S!(   2,    2), 
];
pub const QUEEN_MOBILITY: [Score; 28] = [
    S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), 
    S!(   0,    0), S!(   0,    0), S!(  25,   17), S!( -10,  -10), S!(  -4,   -4), S!(   1,    0), S!(  -4,   -4), S!(  -1,   -1), 
    S!(  -2,   -2), S!(   1,    0), S!(  -1,    0), S!(  -5,   -4), S!(  -1,    0), S!(   3,    2), S!(  -1,    0), S!(  -1,    0), 
    S!(  -1,    0), S!(  15,   13), S!(   0,    0), S!(   0,    0), 
];
pub const KING_MOBILITY: [Score; 9] = [
    S!(   4,    4), S!(  16,   -7), S!(  13,  -15), S!(   5,   -7), S!(  -6,    3), S!(  -3,   -4), S!(  -5,    2), S!(  -2,    3), 
    S!(  -1,    2), 
];
pub const VIRTUAL_MOBILITY: [Score; 28] = [
    S!(  -1,    0), S!(  10,   10), S!(  11,   12), S!(  11,    8), S!(   8,    5), S!(  11,    5), S!(   3,    4), S!(   2,   -2), 
    S!(  -1,   -2), S!(   1,   -1), S!(  -4,    1), S!(   2,    1), S!( -20,    3), S!( -15,    9), S!(   1,   -1), S!(   2,    1), 
    S!(  -2,    3), S!(   4,    3), S!(  -1,    2), S!(  -1,    3), S!(   0,    1), S!(  -2,   -2), S!(  -7,   -8), S!(  -5,   -5), 
    S!( -15,  -27), S!( -20,  -20), S!(   0,    1), S!( -34,  -34), 
];

pub const BISHOP_PAIR_BONUS: Score = S!(  -1,    0);
pub const PASSED_PAWN_BONUS: Score = S!(  11,   22);
pub const PAWN_ISLAND_PENALTY: Score = S!(   1,    0);
pub const PAWN_ISOLATION_PENALTY: Score = S!( -12,  -12);
pub const ROOK_OPEN_FILE_BONUS: Score = S!(  23,   23);
pub const ROOK_SEMI_FILE_BONUS: Score = S!(  13,   12);
pub const PAWN_SHIELD_PENALTY: Score = S!(  -3,    2);
 
//PSTs
pub const P: [Score; 64] = [
    S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), 
    S!( -20,   10), S!( -11,   10), S!(  -8,    4), S!( -18,   -7), S!(  -3,    2), S!(  20,    7), S!(  18,   19), S!(  -6,   -5), 
    S!( -19,    3), S!( -11,    3), S!(  -7,   -8), S!( -14,   14), S!(   6,    1), S!(   5,    4), S!(  16,    1), S!(   8,  -12), 
    S!( -17,   12), S!(  -4,    7), S!(  -6,   -3), S!(   9,   10), S!(   7,   -5), S!(  11,   -1), S!(   4,    4), S!(   0,   -4), 
    S!(   5,   28), S!(  16,   15), S!(   7,   16), S!(  13,    0), S!(  42,   -5), S!(  29,    6), S!(  23,   24), S!(  18,   17), 
    S!(  32,   99), S!(  18,  121), S!(  57,   80), S!(  56,   54), S!(  58,   55), S!(  49,   49), S!(  81,   84), S!(  69,   68), 
    S!( 161,  156), S!( 156,  150), S!( 139,  139), S!( 125,  126), S!( 138,  137), S!(  85,   90), S!(  98,  122), S!(  98,  122), 
    S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0)
];
pub const N: [Score; 64] = [
    S!( -84,  -73), S!( -45,  -45), S!( -50,  -14), S!( -39,  -29), S!( -41,  -19), S!( -33,  -28), S!( -47,  -32), S!( -49,  -50), 
    S!( -39,  -40), S!( -33,  -22), S!( -20,  -13), S!( -17,   -8), S!( -19,   -4), S!(   1,    0), S!( -22,  -34), S!( -24,  -26), 
    S!( -41,  -32), S!( -20,    4), S!( -11,    5), S!(   3,    2), S!(  15,   15), S!( -11,   -1), S!(  -3,   -5), S!( -33,  -30), 
    S!( -22,  -23), S!(   1,    0), S!(  15,   15), S!(  10,   22), S!(  15,   27), S!(  14,   16), S!(  19,   10), S!( -12,  -15), 
    S!(  -8,   -9), S!(  -3,    9), S!(  27,   22), S!(  24,   24), S!(  22,   22), S!(  54,   30), S!(  14,   14), S!(  12,    1), 
    S!( -29,  -30), S!(   5,    5), S!(  51,   17), S!(  44,   20), S!(  49,   25), S!(  47,   23), S!(  19,   14), S!( -16,  -19), 
    S!( -28,  -27), S!(   4,   -5), S!(  20,    6), S!(   0,    0), S!(  10,    5), S!(  37,   13), S!(   0,   -5), S!(  -2,  -25), 
    S!( -50,  -50), S!( -40,  -40), S!( -30,  -30), S!(  -9,  -12), S!( -10,  -15), S!( -30,  -30), S!( -38,  -40), S!( -50,  -50)
];
pub const B: [Score; 64] = [
    S!( -22,  -29), S!(  -3,   -7), S!( -12,  -12), S!( -15,  -10), S!(  -1,   -9), S!( -13,  -13), S!(   7,  -16), S!( -17,  -18), 
    S!(   8,  -39), S!(  -5,   -5), S!(  15,   -9), S!(  -8,   -6), S!(   0,    3), S!(   9,   -1), S!(  12,  -11), S!( -11,  -10), 
    S!(   0,  -17), S!(   3,    3), S!(   0,    4), S!(  10,   -4), S!(  10,    7), S!(  -2,    1), S!(  -6,   -3), S!(   0,   -4), 
    S!( -16,  -12), S!(   5,    4), S!(   2,    3), S!(  28,   11), S!(  10,   10), S!(   8,    9), S!(  -1,    0), S!( -10,  -12), 
    S!( -12,  -12), S!(   4,    4), S!(   9,    9), S!(  38,   21), S!(  25,   21), S!(  13,   13), S!(   6,    6), S!(  -2,    0), 
    S!(  -9,   -1), S!(   8,    1), S!(  17,    6), S!(  37,   -6), S!(  25,    8), S!(   5,    5), S!(  15,   15), S!(  29,    5), 
    S!(  -4,   -8), S!(  -1,    0), S!(   1,    0), S!(   0,    0), S!(  20,    5), S!(  -7,   -5), S!(  -1,    0), S!( -28,  -27), 
    S!( -19,  -20), S!( -10,  -10), S!( -10,  -12), S!( -10,  -10), S!( -10,  -10), S!( -10,  -10), S!( -10,  -10), S!( -20,  -20)
];
pub const R: [Score; 64] = [
    S!( -13,  -14), S!( -12,   -3), S!(  -3,   -3), S!(  -9,    0), S!(  -6,   -2), S!( -11,   -5), S!(  -6,   -5), S!(  -4,   -4), 
    S!( -31,   -2), S!( -27,  -16), S!(   1,    0), S!( -19,   -6), S!( -19,   -4), S!( -11,   -7), S!(   3,   -2), S!( -26,  -23), 
    S!(  -6,   -6), S!( -11,    1), S!( -25,    1), S!( -13,  -12), S!( -25,    5), S!( -14,   -9), S!(  -2,   -1), S!( -11,  -22), 
    S!( -40,    8), S!( -16,    4), S!( -24,    9), S!(  -3,    5), S!(   2,   -1), S!(   1,    0), S!(   6,    2), S!(   0,   -3), 
    S!(  -4,   -5), S!(  25,   -1), S!(   4,    9), S!(  11,   10), S!(  -2,   -1), S!(  -1,    0), S!(   6,    4), S!(  24,    0), 
    S!(   1,   -1), S!(  13,   20), S!(  20,   16), S!(  15,   11), S!(  -1,    0), S!(  -1,    0), S!(   0,    0), S!(  11,    6), 
    S!(  10,   10), S!(  10,   10), S!(  10,   10), S!(  10,   10), S!(  20,   15), S!(  29,   18), S!(  14,   14), S!(   5,    5), 
    S!(  21,   18), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0), S!(   0,    0)
];
pub const Q: [Score; 64] = [
    S!(  -8,   -7), S!( -28,  -22), S!( -17,  -16), S!( -10,  -10), S!(  -5,   -5), S!( -10,  -10), S!( -10,  -10), S!( -18,  -21), 
    S!( -28,  -27), S!( -11,  -10), S!(   1,    0), S!(   1,    0), S!(  -4,   -3), S!(   4,   -1), S!( -19,  -15), S!( -10,  -12), 
    S!( -19,  -19), S!(  -8,   -3), S!(  -4,    2), S!(  -8,    6), S!(   1,    6), S!(  -1,    4), S!(   4,    3), S!(  -7,   -5), 
    S!(  -4,   -4), S!( -10,   -1), S!(  -2,    9), S!(   5,    5), S!(   5,    5), S!(   5,    5), S!(  -1,    0), S!(   0,   -1), 
    S!(  -5,   -5), S!(   2,    2), S!(   7,    6), S!(  17,   22), S!(   5,    5), S!(  23,   23), S!(  -1,    0), S!(  24,   21), 
    S!(  -6,   -7), S!(  -3,   -2), S!(   6,    5), S!(   5,    5), S!(   5,    5), S!(  74,   71), S!(   0,    0), S!(   0,   -1), 
    S!( -10,  -10), S!(   1,    0), S!(  -1,    0), S!(  -1,    0), S!(  44,   43), S!(  36,   33), S!(   5,    5), S!(  22,   20), 
    S!( -22,  -17), S!( -20,  -22), S!(  -9,  -10), S!(  -4,   -5), S!(   0,   -2), S!(   0,   -1), S!(   2,   -4), S!(  -2,   -3)
];
pub const K: [Score; 64] = [
    S!(  23,  -61), S!(  34,  -35), S!(  23,  -25), S!( -72,  -17), S!( -10,  -34), S!(  -8,  -32), S!(  37,  -35), S!(  39,  -62), 
    S!(  23,  -28), S!(  27,  -21), S!(  -6,   -6), S!( -18,   -6), S!( -25,   -2), S!(  -7,   -8), S!(  30,  -16), S!(  26,  -35), 
    S!(   6,  -23), S!( -19,   -4), S!( -38,   10), S!( -39,   12), S!( -35,   17), S!( -34,   13), S!( -16,   -7), S!(  -4,  -16), 
    S!( -18,  -24), S!( -27,   -5), S!( -33,   21), S!( -47,   37), S!( -44,   38), S!( -43,   28), S!( -16,    4), S!( -12,  -18), 
    S!(  -5,   -5), S!( -13,   17), S!( -46,   29), S!( -48,   38), S!( -42,   45), S!( -39,   32), S!(  -9,   21), S!( -29,  -30), 
    S!( -16,   -4), S!( -29,    7), S!( -25,   29), S!( -38,   40), S!( -28,   52), S!( -16,   43), S!(   9,   36), S!( -17,  -18), 
    S!( -30,  -30), S!( -18,    1), S!( -29,    2), S!( -41,    8), S!( -12,   39), S!(  -4,   32), S!(  43,   40), S!(  -8,   -7), 
    S!( -72,  -92), S!( -47,  -48), S!( -40,  -31), S!( -35,   -7), S!( -43,  -18), S!( -11,   -4), S!( -39,  -40), S!( -28,  -52)
];