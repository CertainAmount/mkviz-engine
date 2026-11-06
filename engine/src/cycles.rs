//! 정책 금리 사이클: 목표금리가 바뀐 날들을 같은 방향끼리 묶는다(올림이 이어지면 인상 사이클, 방향이 바뀌면 새 사이클).
//! "연준이 내리다 다시 올리기 시작한 때"(인상 사이클의 첫 인상) 같은 사건 날짜를 만든다(METHODS.md A2·A5의 사건 정의).

use crate::series::Point;

#[derive(Debug, Clone, PartialEq)]
pub struct Cycle {
    pub up: bool,
    pub start: String,
    pub end: String,
    pub moves: usize,
    pub from: f64,
    pub to: f64,
}

pub fn cycles(pts: &[Point]) -> Vec<Cycle> {
    let mut out: Vec<Cycle> = Vec::new();
    for w in pts.windows(2) {
        let d = w[1].value - w[0].value;
        if d.abs() < 1e-9 {
            continue;
        }
        let up = d > 0.0;
        match out.last_mut() {
            Some(c) if c.up == up => {
                c.end = w[1].date.clone();
                c.moves += 1;
                c.to = w[1].value;
            }
            _ => out.push(Cycle { up, start: w[1].date.clone(), end: w[1].date.clone(), moves: 1, from: w[0].value, to: w[1].value }),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::parse;

    #[test]
    fn groups_moves_by_direction() {
        let p = parse("d,v\n2020-01-01,1\n2020-02-01,1.25\n2020-03-01,1.5\n2020-04-01,1.5\n2020-05-01,1.25\n2020-06-01,1.5\n");
        let c = cycles(&p);
        assert_eq!(c.len(), 3);
        assert_eq!((c[0].up, c[0].moves, c[0].start.as_str(), c[0].end.as_str()), (true, 2, "2020-02-01", "2020-03-01"));
        assert!(!c[1].up && c[2].up && c[2].start == "2020-06-01");
    }
}
