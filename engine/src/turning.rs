//! 강세·약세장 판정: Pagan & Sossounov (2003), Bry–Boschan(1971)을 주가에 맞춘 규칙. 월말 값으로 본다.
//! 1) 앞뒤 window달 안에서 가장 높으면 고점 후보, 가장 낮으면 저점 후보 2) 고점·저점이 번갈아 오게(같은 종류가 이어지면 더 극단인 것)
//! 3) 양 끝 censor달 안의 전환점 제거 4) 한 국면(고점→저점, 저점→고점)이 min_phase달보다 짧으면 제거, 단 변화가 big 이상이면 둔다
//! 5) 한 주기(고점→고점, 저점→저점)가 min_cycle달보다 짧으면 제거. 기본값은 논문의 월간 값(8, 6, 4, 16, 20%).
//! 마지막 전환점은 window가 지나기 전까지 잠정이다(METHODS.md A1).

use crate::series::Point;

pub struct Params {
    pub window: usize,
    pub censor: usize,
    pub min_phase: usize,
    pub min_cycle: usize,
    pub big: f64,
}

impl Default for Params {
    fn default() -> Self {
        Params { window: 8, censor: 6, min_phase: 4, min_cycle: 16, big: 0.20 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub i: usize,
    pub peak: bool,
}

/// 일·주 자료를 달마다 마지막 값 하나로.
pub fn monthly(pts: &[Point]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    for p in pts {
        match out.last_mut() {
            Some(l) if l.date[..7] == p.date[..7] => *l = p.clone(),
            _ => out.push(p.clone()),
        }
    }
    out
}

fn alternate(v: &[f64], mut t: Vec<Turn>) -> Vec<Turn> {
    let mut out: Vec<Turn> = Vec::new();
    for x in t.drain(..) {
        match out.last() {
            Some(l) if l.peak == x.peak => {
                let better = if x.peak { v[x.i] > v[l.i] } else { v[x.i] < v[l.i] };
                if better {
                    *out.last_mut().unwrap() = x;
                }
            }
            _ => out.push(x),
        }
    }
    out
}

pub fn turns(m: &[Point], p: &Params) -> Vec<Turn> {
    let v: Vec<f64> = m.iter().map(|x| x.value).collect();
    let n = v.len();
    let mut t = Vec::new();
    for i in 0..n {
        let (a, b) = (i.saturating_sub(p.window), (i + p.window).min(n - 1));
        let w = &v[a..=b];
        if w.iter().all(|&x| x <= v[i]) {
            t.push(Turn { i, peak: true });
        } else if w.iter().all(|&x| x >= v[i]) {
            t.push(Turn { i, peak: false });
        }
    }
    let mut t = alternate(&v, t);
    t.retain(|x| x.i >= p.censor && x.i + p.censor < n);
    t = alternate(&v, t);
    loop {
        // 짧은 국면: 변화가 작은 쪽의 두 전환점을 지운다
        let short = (0..t.len().saturating_sub(1)).find(|&k| {
            let (x, y) = (&t[k], &t[k + 1]);
            y.i - x.i < p.min_phase && (v[y.i] / v[x.i] - 1.0).abs() < p.big
        });
        let cyc = (0..t.len().saturating_sub(2)).find(|&k| t[k + 2].i - t[k].i < p.min_cycle);
        match short.or(cyc) {
            Some(k) => {
                t.drain(k..(k + 2).min(t.len()));
                t = alternate(&v, t);
            }
            None => break,
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::parse;

    fn series(vals: &[f64]) -> Vec<Point> {
        let mut s = String::from("d,v\n");
        for (k, x) in vals.iter().enumerate() {
            s += &format!("{}-{:02}-01,{x}\n", 2000 + k / 12, k % 12 + 1);
        }
        parse(&s)
    }

    #[test]
    fn finds_one_bear_in_a_v() {
        // 20달 오르다 12달 40% 내리고 20달 오른다
        let mut v: Vec<f64> = (0..20).map(|k| 100.0 + k as f64).collect();
        v.extend((1..=12).map(|k| 119.0 * (1.0 - 0.4 * k as f64 / 12.0)));
        v.extend((1..=20).map(|k| 71.4 + 3.0 * k as f64));
        let t = turns(&series(&v), &Params::default());
        assert_eq!(t.len(), 2);
        assert!(t[0].peak && !t[1].peak);
        assert_eq!((t[0].i, t[1].i), (19, 31));
    }

    #[test]
    fn turns_alternate() {
        let v: Vec<f64> = (0..120).map(|k| 100.0 + 30.0 * ((k as f64) / 9.0).sin() + k as f64 * 0.2).collect();
        let t = turns(&series(&v), &Params::default());
        assert!(t.windows(2).all(|w| w[0].peak != w[1].peak));
    }

    #[test]
    fn monthly_keeps_last_value() {
        let m = monthly(&parse("d,v\n2020-01-02,1\n2020-01-31,2\n2020-02-03,3\n"));
        assert_eq!(m.iter().map(|p| p.value).collect::<Vec<_>>(), vec![2.0, 3.0]);
    }
}
