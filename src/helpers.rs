use crate::score::ScoreTy;
use chess_move_gen::{
    legal_moves, Board, Move, MoveVec, Side, Square, BISHOP, KING, KING_SIDE, KNIGHT, PAWN, QUEEN,
    QUEEN_SIDE, ROOK,
};

#[inline(always)]
pub fn game_over(board: Board) -> bool {
    match board.status() {
        BoardStatus::Ongoing => false,
        BoardStatus::Stalemate | BoardStatus::Checkmate => true,
    }
}

#[inline(always)]
pub fn color_to_num(color: Side) -> ScoreTy {
    match color.0 {
        // W
        0 => 1,
        // B
        1 => -1,
    }
}

pub const N_INF: ScoreTy = ScoreTy::MIN + 1;
pub const P_INF: ScoreTy = ScoreTy::MAX;

pub fn from_san(board: &Board, move_text: &str) -> Move {
    // Castles first...
    if move_text == "O-O" {
        return Move::new_castle(KING_SIDE);
    } else if move_text == "O-O-O" {
        return Move::new_castle(QUEEN_SIDE);
    }

    // forms of SAN moves
    // a4 (Pawn moves to a4)
    // exd4 (Pawn on e file takes on d4)
    // xd4 (Illegal, source file must be specified)
    // 1xd4 (Illegal, source file (not rank) must be specified)
    // Nc3 (Knight (or any piece) on *some square* to c3
    // Nb1c3 (Knight (or any piece) on b1 to c3
    // Nbc3 (Knight on b file to c3)
    // N1c3 (Knight on first rank to c3)
    // Nb1xc3 (Knight on b1 takes on c3)
    // Nbxc3 (Knight on b file takes on c3)
    // N1xc3 (Knight on first rank takes on c3)
    // Nc3+ (Knight moves to c3 with check)
    // Nc3# (Knight moves to c3 with checkmate)

    // Because I'm dumb, I'm wondering if a hash table of all possible moves would be stupid.
    // There are only 186624 possible moves in SAN notation.
    //
    // Would this even be faster?  Somehow I doubt it because caching, but maybe, I dunno...
    // This could take the form of a:
    // struct CheckOrCheckmate {
    //      Neither,
    //      Check,
    //      CheckMate,
    // }
    // struct FromSan {
    //      piece: Piece,
    //      source: Vec<Square>, // possible source squares
    //      // OR
    //      source_rank: Option<Rank>,
    //      source_file: Option<File>,
    //      dest: Square,
    //      takes: bool,
    //      check: CheckOrCheckmate
    // }
    //
    // This could be kept internally as well, and never tell the user about such an abomination
    //
    // I estimate this table would take around 2 MiB, but I had to approximate some things.  It
    // may be less

    // This can be described with the following format
    // [Optional Piece Specifier] ("" | "N" | "B" | "R" | "Q" | "K")
    // [Optional Source Specifier] ( "" | "a-h" | "1-8" | ("a-h" + "1-8"))
    // [Optional Takes Specifier] ("" | "x")
    // [Full Destination Square] ("a-h" + "0-8")
    // [Optional Promotion Specifier] ("" | "N" | "B" | "R" | "Q")
    // [Optional Check(mate) Specifier] ("" | "+" | "#")
    // [Optional En Passant Specifier] ("" | " e.p.")

    let mut cur_index: usize = 0;
    let moving_piece = match &move_text[cur_index..(cur_index + 1)] {
        "N" => {
            cur_index += 1;
            KNIGHT
        }
        "B" => {
            cur_index += 1;
            BISHOP
        }
        "Q" => {
            cur_index += 1;
            QUEEN
        }
        "R" => {
            cur_index += 1;
            ROOK
        }
        "K" => {
            cur_index += 1;
            KING
        }
        _ => PAWN,
    };

    let file = move_text[cur_index..(cur_index + 1)]
        .chars()
        .next()
        .unwrap();
    let mut source_file = if file >= 'a' && file <= 'h' {
        cur_index += 1;
        Some((file as usize) - ('a' as usize))
    } else {
        None
    };

    let rank = move_text[cur_index..(cur_index + 1)]
        .chars()
        .next()
        .unwrap();
    let mut source_rank = if rank >= '1' && rank <= '9' {
        cur_index += 1;
        Some((rank as usize) - ('1' as usize))
    } else {
        None
    };

    let takes = if let Some(s) = move_text.get(cur_index..(cur_index + 1)) {
        match s {
            "x" => {
                cur_index += 1;
                true
            }
            _ => false,
        }
    } else {
        false
    };

    let dest = if let Some(s) = move_text.get(cur_index..(cur_index + 2)) {
        if let Ok(Some(q)) = Square::parse(s) {
            cur_index += 2;
            q
        } else {
            let sq = Square::from(source_rank.unwrap(), source_file.unwrap());
            source_rank = None;
            source_file = None;
            sq
        }
    } else {
        let sq = Square::from(source_rank.unwrap(), source_file.unwrap());
        source_rank = None;
        source_file = None;
        sq
    };

    let promotion = if let Some(s) = move_text.get(cur_index..(cur_index + 1)) {
        match s {
            "N" => {
                cur_index += 1;
                Some(KNIGHT)
            }
            "B" => {
                cur_index += 1;
                Some(BISHOP)
            }
            "R" => {
                cur_index += 1;
                Some(ROOK)
            }
            "Q" => {
                cur_index += 1;
                Some(QUEEN)
            }
            _ => None,
        }
    } else {
        None
    };

    if let Some(s) = move_text.get(cur_index..(cur_index + 1)) {
        let _maybe_check_or_mate = match s {
            "+" => {
                cur_index += 1;
                Some(false)
            }
            "#" => {
                cur_index += 1;
                Some(true)
            }
            _ => None,
        };
    }

    let ep = if let Some(s) = move_text.get(cur_index..) {
        s == " e.p."
    } else {
        false
    };

    //if ep {
    //    cur_index += 5;
    //}

    // Ok, now we have all the data from the SAN move, in the following structures
    // moveing_piece, source_rank, source_file, taks, dest, promotion, maybe_check_or_mate, and
    // ep

    let mut found_move: Option<Move> = None;
    let mut move_buffer = MoveVec::new();
    legal_moves(board.position(), &mut move_buffer);
    for m in move_buffer.iter() {
        // check that the move has the properties specified
        if board.position().at(m.from()).kind() != moving_piece {
            continue;
        }

        if let Some(rank) = source_rank {
            if m.from().row() != rank {
                continue;
            }
        }

        if let Some(file) = source_file {
            if m.from().col() != file {
                continue;
            }
        }

        if m.to() != dest {
            continue;
        }

        if (m.is_promotion() && Some(m.promote_to()) != promotion)
            || m.is_promotion() && promotion.is_none()
        {
            continue;
        }

        if found_move.is_some() {
            panic!("This shouldn't happen");
        }

        // takes is complicated, because of e.p.
        if !takes || (!ep && takes) {
            if board.position().at(m.to()).is_some() {
                continue;
            }
        }

        found_move = Some(*m);
    }

    found_move.unwrap()
}
