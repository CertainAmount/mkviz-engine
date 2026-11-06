//! 사건 이후: 주어진 날짜들 뒤 N일의 변화를 모으고, 같은 길이의 아무 구간(기저율)과 견준다(MacKinlay 1997을 거시에 맞춘 꼴,
//! 기저율·참조군: Kahneman & Tversky, Flyvbjerg 2006). 사건이 적으면 모든 사례를 함께 보여 준다(METHODS.md A2, A5, A6).

use crate::forward::{after, after_diff, worst};
use crate::series::Point;
use crate::stats::median;

pub struct Window {
    pub days: i64,
    pub per_event: Vec<Option<f64>>,
    pub median: Option<f64>,
    pub up: usize,
    pub n: usize,
    pub base_median: Option<f64>,
    pub base_up_share: Option<f64>,
    pub worst: Vec<Option<f64>>,
}

pub fn study(pts: &[Point], events: &[i64], horizons: &[i64], diff: bool) -> Vec<Window> {
    let f = |p: &[Point], a: i64, h: i64| if diff { after_diff(p, a, h) } else { after(p, a, h) };
    horizons
        .iter()
        .map(|&h| {
            let per: Vec<Option<f64>> = events.iter().map(|&e| f(pts, e, h)).collect();
            let got: Vec<f64> = per.iter().flatten().cloned().collect();
            // 기저율: 첫 점부터 21번째 점마다 시작한 같은 길이 구간 전부(겹침을 줄이려 간격을 둔다)
            let base: Vec<f64> = pts.iter().step_by(21).filter_map(|p| f(pts, p.day, h)).collect();
            Window {
                days: h,
                median: median(&got),
                up: got.iter().filter(|&&x| x > 0.0).count(),
                n: got.len(),
                base_median: median(&base),
                base_up_share: if base.is_empty() { None } else { Some(base.iter().filter(|&&x| x > 0.0).count() as f64 / base.len() as f64) },
                worst: events.iter().map(|&e| if diff { None } else { worst(pts, e, h) }).collect(),
                per_event: per,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::{day_number, parse};

    #[test]
    fn event_vs_base() {
        let mut s = String::from("d,v\n");
        for k in 0..400 {
            s += &format!("{},{}\n", crate_date(k), 100.0 + k as f64);
        }
        let p = parse(&s);
        let w = &study(&p, &[day_number("2000-01-11").unwrap()], &[30], false)[0];
        assert_eq!((w.n, w.up), (1, 1));
        assert!(w.base_up_share.unwrap() > 0.99);
    }

    fn crate_date(k: i64) -> String {
        // 2000-01-01부터 k일
        let mut d = (2000, 1, 1);
        let days = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for _ in 0..k {
            d.2 += 1;
            if d.2 > days[d.1 - 1] {
                d.2 = 1;
                d.1 += 1;
                if d.1 > 12 {
                    d.1 = 1;
                    d.0 += 1;
                }
            }
        }
        format!("{}-{:02}-{:02}", d.0, d.1, d.2)
    }
}
