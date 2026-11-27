//! 비주얼 엔진: 분석 결과를 '프레임'(정해진 꼴의 그림 틀)에 채워 SVG로 낸다(Grammar of Graphics: 데이터 → 위치·색 매핑 → 표시).
//! 지각 연구(Cleveland & McGill 1984): 비교는 같은 축 위의 위치로. 기억(Borkin 외 2013): 강조는 하나(지금 사례)만.
//! 프레임은 이 파일에 함수 하나씩 더한다. 폭 600 viewBox, 글자 12 이상, 사이트 팔레트(METHODS.md B4·B5).

use crate::series::{Point, day_number};
use std::fmt::Write;

const W: f64 = 480.0; // 지침 §4-2: 휴대폰 폭에서 글자가 줄지 않게(600이면 7px까지 줄었다, 2026-11-06)
const H: f64 = 280.0;
const L: f64 = 44.0;
const R: f64 = 16.0;
const T: f64 = 14.0;
const B: f64 = 26.0;
const BLUE: &str = "#2563eb";
const GRAY: &str = "#94a3b8";
const SHADE: &str = "#e2e8f0";
const INK: &str = "#475569";

fn head(title: &str) -> String {
    format!(
        "<svg viewBox=\"0 0 {W} {H}\" width=\"{W}\" height=\"{H}\" style=\"width:100%;max-width:560px;height:auto\" role=\"img\" xmlns=\"http://www.w3.org/2000/svg\" font-family=\"system-ui,sans-serif\" font-size=\"16\"><title>{}</title>",
        title.replace('&', "&amp;").replace('<', "&lt;")
    )
}

fn pct(v: f64) -> String {
    let r = (v * 100.0).round();
    format!("{}%", if r == 0.0 { 0.0 } else { r }) // -0% 대신 0%
}

/// 고점 대비 하락폭(물속 그림): 0선 아래로 얼마나 잠겼는지. 기준선(예: -10%, -20%)과 flag(경기침체) 음영.
pub fn underwater(pts: &[Point], lines: &[f64], flag: &[Point], marks_in: &[(i64, f64)], title: &str) -> String {
    let mut peak = f64::MIN;
    let dd: Vec<(i64, f64)> = pts.iter().map(|p| { peak = peak.max(p.value); (p.day, p.value / peak - 1.0) }).collect();
    let (x0, x1) = (dd[0].0 as f64, dd.last().unwrap().0 as f64);
    let lo = dd.iter().map(|d| d.1).fold(0.0, f64::min).min(-0.1);
    let lo = (lo * 10.0).floor() / 10.0;
    let x = |d: f64| L + (d - x0) / (x1 - x0).max(1.0) * (W - L - R);
    let y = |v: f64| T + (0.0 - v) / (0.0 - lo) * (H - T - B);
    let mut s = head(title);
    // 경기침체 음영(월간 flag가 1인 달)
    for f in flag.iter().filter(|f| f.value >= 1.0 && f.day as f64 >= x0 && f.day as f64 <= x1) {
        let _ = write!(s, "<rect x=\"{:.1}\" y=\"{T}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{SHADE}\"/>", x(f.day as f64), (x(f.day as f64 + 31.0) - x(f.day as f64)).max(1.0), H - T - B);
    }
    let mut v = 0.0;
    while v >= lo - 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#e5e7eb\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", W - R, y(v), y(v), L - 6.0, y(v) + 4.0, pct(v));
        v -= 0.2;
    }
    for &t in lines {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R, y(-t), y(-t));
        if ((t * 100.0).round() as i64) % 20 != 0 {
            let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", L - 6.0, y(-t) + 4.0, pct(-t));
        }
    }
    let mut path = format!("M{:.1},{:.1}", x(x0), y(0.0));
    for (d, v) in &dd {
        let _ = write!(path, "L{:.1},{:.1}", x(*d as f64), y(*v));
    }
    let _ = write!(path, "L{:.1},{:.1}Z", x(x1), y(0.0));
    let _ = write!(s, "<path d=\"{path}\" fill=\"{BLUE}\" fill-opacity=\"0.25\" stroke=\"{BLUE}\" stroke-width=\"1.2\"/>");
    // 기준선(가장 얕은 lines)보다 깊었던 하락마다 바닥 아래에 그 해를 적는다(그림만 떼어 봐도 어느 하락인지 읽히게)
    let cut = lines.iter().cloned().fold(f64::MAX, f64::min);
    if cut < f64::MAX {
        let mut seg: Option<(i64, f64)> = None;
        let mut marks: Vec<(i64, f64)> = vec![];
        for &(d, v) in dd.iter().chain(std::iter::once(&(i64::MAX, 0.0))).filter(|_| marks_in.is_empty()) {
            if v < 0.0 {
                if seg.map_or(true, |m| v < m.1) { seg = Some((d, v)); }
            } else if let Some(m) = seg.take() {
                if m.1 <= -cut { marks.push(m); }
            }
        }
        if !marks_in.is_empty() {
            // 원래 해상도(일별)의 하락 기록에서 받은 바닥: 그림은 월평균이라도 숫자는 본문 표와 같게
            marks = marks_in.iter().filter(|m| m.1 <= -cut).cloned().collect();
        }
        // 이름표가 겹치면(가로 44 이내) 뒤의 것을 한 줄 아래로 내린다(지금 진행 중인 하락도 빠지지 않게)
        let mut placed: Vec<(f64, f64)> = vec![];
        for (d, v) in marks {
            let mut ly = (y(v) + 18.0).min(H - B - 4.0);
            while placed.iter().any(|&(px, py)| (x(d as f64) - px).abs() < 70.0 && (ly - py).abs() < 16.0) { ly += 17.0; }
            placed.push((x(d as f64), ly));
            // 바닥이 든 달(달 첫 관측일 ≤ 바닥일인 마지막 점): 1987-12-04 바닥이 다음 달 점을 집어 1988로 적혔다
            let year = pts.iter().filter(|p| p.day <= d).last().or(pts.first()).map(|p| &p.date[..4]).unwrap_or("");
            // 해와 깊이를 함께(그림만 봐도 '얼마나'가 읽히게, 주석 연구: Borkin 외 2016)
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{year} {}</text>", x(d as f64).clamp(L + 30.0, W - R - 30.0), ly, pct(v));
        }
    }
    // 지금 위치: 마지막 점에 점과 'Now'(독자가 '지금은 어디냐'를 바로 찾게)
    if let Some(&(d, v)) = dd.last() {
        if v < 0.0 {
            // 깊은 하락의 오른쪽 끝은 촘촘한 선 위라 흰 테두리를 둘러도 글자가 깨져 보였다(2026-11-24 콜드 리드 두 독자).
            // 점 바로 왼쪽, 0% 아래 빈 띠에 선이 올라오지 않으면 'Now'와 깊이를 두 줄로 그 띠에 두고 점까지 선을 긋는다
            let wide = 6.0 + 8.0 * pct(v).chars().count().max(3) as f64;  // 굵은 14px 글자 너비 어림
            let free = y(v) - T >= 50.0 && dd.iter().filter(|p| x(p.0 as f64) >= x(d as f64) - wide).all(|p| y(p.1) - T > 38.0);
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{BLUE}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(d as f64), y(v));
            if free {
                let xe = x(d as f64) - 6.0;
                let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\"/><text text-anchor=\"end\" fill=\"{INK}\" font-weight=\"600\" font-size=\"14\"><tspan x=\"{xe:.1}\" y=\"{:.1}\">Now</tspan><tspan x=\"{xe:.1}\" y=\"{:.1}\">{}</tspan></text>",
                    x(d as f64), x(d as f64), T + 4.0, y(v) - 6.0, T + 16.0, T + 32.0, pct(v));
            } else {
                let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-weight=\"600\" font-size=\"14\" stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\">Now {}</text>", x(d as f64) - 8.0,
                    // 0% 선 가까이면 점 아래에(위에 두면 기준선·선과 겹쳤다, 2026-11-17 콜드 리드), 흰 테두리로 선 위에서도 읽히게
                    if y(v) - T < 30.0 { y(v) + 22.0 } else { y(v) - 10.0 }, pct(v));
            }
        }
    }
    let (ya, yb) = (&pts[0].date[..4], &pts.last().unwrap().date[..4]);
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\">{ya}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{yb}</text></svg>", H - 6.0, W - R, H - 6.0);
    s
}

/// 사건 뒤 경로 겹치기: 각 사건일을 0으로 맞춘 뒤 days일까지의 변화. 지난 사례는 회색, 마지막(지금) 사례만 파랑.
/// diff: 비율 대신 수준 차이(금리 %p). labels: 선마다 끝에 붙일 이름(시작 연도 등, 비우면 없음).
pub fn paths_with(pts: &[Point], starts: &[String], days: i64, title: &str, diff: bool, labels: &[String]) -> String {
    let lines: Vec<Vec<(f64, f64)>> = starts
        .iter()
        .filter_map(|d| {
            let s = day_number(d)?;
            let i = pts.iter().position(|p| p.day >= s)?;
            let w: Vec<&Point> = pts[i..].iter().take_while(|p| p.day - pts[i].day <= days).collect();
            // 선 하나에 구간 60개 안팎(파일을 작게). 구간마다 최고·최저를 둘 다 남겨 꼭대기·바닥이 깎이지 않게(2000년 3월 고점이 사라졌다)
            let step = (w.len() / 60).max(1);
            let v = |p: &Point| if diff { p.value - pts[i].value } else { p.value / pts[i].value - 1.0 };
            let mut out: Vec<(f64, f64)> = vec![(0.0, 0.0)]; // 출발점은 늘 0
            for chunk in w.chunks(step) {
                let hi = chunk.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap()).unwrap();
                let lo = chunk.iter().min_by(|a, b| a.value.partial_cmp(&b.value).unwrap()).unwrap();
                let mut two = [hi, lo];
                two.sort_by_key(|p| p.day);
                for p in two { if out.last().map_or(true, |l| l.0 != (p.day - pts[i].day) as f64) { out.push(((p.day - pts[i].day) as f64, v(p))); } }
            }
            if let Some(last) = w.last() { if out.last().map_or(true, |l| l.0 != (last.day - pts[i].day) as f64) { out.push(((last.day - pts[i].day) as f64, v(last))); } }
            Some(out)
        })
        .collect();
    let all: Vec<f64> = lines.iter().flatten().map(|p| p.1).collect();
    let (mut lo, mut hi) = (all.iter().cloned().fold(0.0, f64::min), all.iter().cloned().fold(0.0, f64::max));
    let unit = if diff { 2.0 } else { 10.0 }; // diff: 0.5%p 눈금
    lo = (lo * unit).floor() / unit;
    hi = (hi * unit).ceil() / unit;
    let right = if labels.is_empty() { R } else { R + 76.0 }; // 끝 이름표 + 값 두 줄 자리
    let lm = if hi >= 9.995 || lo <= -0.995 { L + 18.0 } else if hi >= 0.995 { L + 10.0 } else { L }; // 1000%·100% 눈금 글자가 잘리지 않게
    let x = |d: f64| lm + d / days as f64 * (W - lm - right);
    let y = |v: f64| T + (hi - v) / (hi - lo).max(0.01) * (H - T - B);
    let mut s = head(title);
    // 눈금 6개 이하(휴대폰에서 숫자가 겹치지 않게)
    let step = [0.05, 0.1, 0.2, 0.25, 0.5, 1.0, 2.0, 5.0].into_iter().map(|st| if diff { st * 10.0 } else { st }).find(|st| (hi - lo) / st <= 5.0).unwrap_or(1.0);
    let fmt = |v: f64| if diff { if v.abs() < 1e-9 { "0".to_string() } else { format!("{:+.1}", v) } } else { pct(v) };
    let mut v = (lo / step).ceil() * step; // 눈금은 0을 지나게
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{lm}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", W - right, y(v), y(v), if v.abs() < 1e-9 { INK } else { "#e5e7eb" }, lm - 6.0, y(v) + 4.0, fmt(v));
        v += step;
    }
    // 끝 이름표(이름 + 값, 두 줄) 자리: 끝이 가까운 선끼리 세로로 34px 이상 벌린다(1980·2011이 한 자리에 겹쳤다, 2026-11-24 금 그림)
    let mut spots: Vec<(usize, f64, f64)> = lines.iter().enumerate().filter_map(|(k, ln)| ln.last().map(|e| (k, x(e.0), y(e.1)))).collect();
    spots.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    for i in 1..spots.len() {
        for j in 0..i {
            if (spots[i].1 - spots[j].1).abs() < 60.0 && spots[i].2 - spots[j].2 < 34.0 { spots[i].2 = spots[j].2 + 34.0; }
        }
    }
    let spot = |k: usize| spots.iter().find(|s| s.0 == k).map(|s| s.2);
    for (k, ln) in lines.iter().enumerate() {
        let last = k + 1 == lines.len();
        let d: Vec<String> = ln.iter().map(|(a, b)| format!("{:.1},{:.1}", x(*a), y(*b))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\"/>", d.join(" "), if last { BLUE } else { GRAY }, if last { 2.5 } else { 1.0 }, if last { 1.0 } else { 0.6 });
        if let (Some(l), Some(end)) = (labels.get(k), ln.last()) {
            // 끝 이름표에 값까지(그림만 봐도 '얼마나'가 읽히게)
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"15\" font-weight=\"600\">{}</text><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{}</text>",
                           x(end.0) + 6.0, spot(k).unwrap_or(y(end.1)) - 2.0, if last { BLUE } else { INK }, l, x(end.0) + 6.0, spot(k).unwrap_or(y(end.1)) + 15.0, if end.1 > 0.0 { format!("+{}", fmt(end.1)) } else { fmt(end.1) });
        }
    }
    // 선이 하나면 꼭대기와 바닥에 값을 단다(그림만 봐도 '얼마나 올랐다 얼마나 빠졌나'가 읽히게)
    if lines.len() == 1 {
        let ln = &lines[0];
        // 바닥 = 전체 최저, 꼭대기 = 그 바닥 앞의 최고(오른 뒤 빠진 모양을 짚는다)
        let low = ln.iter().cloned().fold((0.0, f64::MAX), |a, p| if p.1 < a.1 { p } else { a });
        let top = ln.iter().filter(|p| p.0 < low.0).cloned().fold((0.0, f64::MIN), |a, p| if p.1 > a.1 { p } else { a });
        if top.1 > f64::MIN {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">Peak +{}</text>",
                           x(top.0), y(top.1), x(top.0) + 8.0, y(top.1) + 5.0, fmt(top.1));
        }
        if low.1 < f64::MAX {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">Low {}</text>",
                           x(low.0), y(low.1), x(low.0), y(low.1) + 20.0, fmt(low.1));
        }
    }
    let _ = write!(s, "<text x=\"{lm}\" y=\"{}\" fill=\"{INK}\">0</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{}</text></svg>", H - 6.0, W - right, H - 6.0, if days % 365 == 0 || days > 730 { let y = (days as f64 / 365.25).round() as i64; format!("{} year{}", y, if y > 1 { "s" } else { "" }) } else { format!("{days} days") });
    s
}

#[cfg(test)]
mod val_label_test {
    #[test]
    fn dollars_and_suffix() {
        assert_eq!(super::val_label(3901.4, "$", 1), "$3,901");
        assert_eq!(super::val_label(5318.4, "$", 2), "$5,318");
        assert_eq!(super::val_label(7.28, "%", 2), "7.28%");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::parse;

    #[test]
    fn frames_are_valid_small_svg() {
        let p = parse("d,v\n2000-01-01,100\n2000-06-01,70\n2001-01-01,110\n");
        let u = underwater(&p, &[0.1, 0.2], &parse("d,v\n2000-03-01,1\n"), &[], "t");
        assert!(u.starts_with("<svg") && u.ends_with("</svg>") && u.contains("-20%") && u.contains("<rect"));
        let q = paths_with(&p, &["2000-01-01".into()], 400, "t", false, &[]);
        assert!(q.contains("polyline") && q.len() < 20_000);
        let st = stack(&[("Now".into(), 4.84, 2.34)], Some(1.67), ["Treasury", "Gap"], "t");
        assert!(st.contains("7.18%") && st.contains("6.51% with the usual gap") && st.contains("+2.34"));
        let g = grid(&[6.0, 7.0], &[-0.1, 0.0], &[vec![0.25, 0.28], vec![0.29, 0.33]], 0.30, Some((1, 1)), "t");
        let sv = savings(&[(0.0, 0.0), (1.0, 208.0)], &[("Usual gap".into(), 0.67, 140.0)], 7.18, "t");
        assert!(sv.contains("$140 a month") && sv.contains("Usual gap") && sv.contains("7.18%"));
        assert!(g.contains("Today") && g.contains("-10%") && g.contains("33%") && g.contains("stroke-width=\"2.5\""));
    }
}

/// 역사 속 위치 띠: 전체 기간 값의 10~90% 범위(띠)와 중앙값(점선), 시계열 선, 지금 점 강조.
pub fn band(pts: &[Point], title: &str, as_pct: bool, suffix: &str) -> String {
    let mut vals: Vec<f64> = pts.iter().map(|p| p.value).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |f: f64| vals[((vals.len() - 1) as f64 * f) as usize];
    // 극단값 몇 개(2020년 청구 건수 600만)가 나머지를 납작하게 만들지 않게 0.5~99.5% 범위로 자르고, 넘는 값은 가장자리에 붙인다
    let (lo, hi) = (q(0.005).min(pts.last().unwrap().value), q(0.995).max(pts.last().unwrap().value));
    let pad = (hi - lo) * 0.05;
    let (lo, hi) = (lo - pad, hi + pad);
    let (x0, x1) = (pts[0].day as f64, pts.last().unwrap().day as f64);
    let x = |d: f64| L + (d - x0) / (x1 - x0).max(1.0) * (W - L - R);
    let y = |v: f64| T + (hi - v.clamp(lo, hi)) / (hi - lo).max(1e-9) * (H - T - B);
    let lab = |v: f64| if as_pct { pct(v) } else if v.abs() >= 1000.0 { format!("{:.0}k", v / 1000.0) } else { format!("{:.1}{suffix}", v) };
    let mut s = head(title);
    let _ = write!(s, "<rect x=\"{L}\" y=\"{:.1}\" width=\"{}\" height=\"{:.1}\" fill=\"{SHADE}\"/>", y(q(0.9)), W - L - R, y(q(0.1)) - y(q(0.9)));
    let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R, y(q(0.5)), y(q(0.5)));
    let mut last_y = f64::MIN;
    for v in [q(0.9), q(0.5), q(0.1)] {
        if y(v) - last_y < 14.0 {
            continue; // 눈금 글자가 겹치면 뺀다
        }
        last_y = y(v);
        let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", L - 6.0, y(v) + 4.0, lab(v));
    }
    let step = (pts.len() / 400).max(1);
    let d: Vec<String> = pts.iter().step_by(step).chain(pts.last()).map(|p| format!("{:.1},{:.1}", x(p.day as f64), y(p.value))).collect();
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{GRAY}\" stroke-width=\"1.2\"/>", d.join(" "));
    let l = pts.last().unwrap();
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{BLUE}\"/>", x(l.day as f64), y(l.value));
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\">{}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{}</text></svg>", H - 6.0, &pts[0].date[..4], W - R, H - 6.0, &l.date[..4]);
    s
}

/// 추적기 그림(데이터 페이지, 2026-11-13): 긴 역사 위의 지금. 선(월말 값), 모든 관측의 가운데 80% 띠와 중앙값 점선, 이름 붙은 고점 몇 개,
/// 지금 점과 값. 띠(--mid, 기본 가운데 80%)·중앙값은 그리는 월말 값이 아니라 모든 관측(rank와 같은 기준)으로 잰다. 범례는 그림 안 한 줄(note).
/// 값 표기: suffix가 "$"면 달러 앞 기호·천 단위 쉼표·정수(금값 그림), 아니면 소수 d자리 + suffix
pub fn val_label(v: f64, suffix: &str, d: usize) -> String {
    if suffix != "$" { return format!("{:.*}{suffix}", d, v); }
    // 100달러 아래(휘발유 갤런당)는 센트까지: $3.64. 그 위(금값)는 정수와 쉼표: $3,901
    if v.abs() < 100.0 { return format!("{}${:.*}", if v < 0.0 { "-" } else { "" }, d.max(2), v.abs()); }
    let n = format!("{:.0}", v.abs());
    let mut out = String::new();
    for (i, c) in n.chars().enumerate() { if i > 0 && (n.len() - i) % 3 == 0 { out.push(','); } out.push(c); }
    format!("{}${out}", if v < 0.0 { "-" } else { "" })
}

pub fn tracker(all: &[Point], drawn: &[Point], title: &str, suffix: &str, peaks: usize, years: &[String], note: &str, mid: f64, now_label: &str) -> String {
    let (qa, qb) = ((1.0 - mid) / 2.0, 1.0 - (1.0 - mid) / 2.0); // 음영: 가운데 mid 몫(0.5면 사분위 범위)
    const HALO: &str = "stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\"";
    let mut vals: Vec<f64> = all.iter().map(|p| p.value).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // 분위수는 두 이웃 값 사이를 잇는다(짝수 개 자료의 중앙값이 아래쪽 값으로 기울지 않게, 본문 중앙값 팩트와 같은 값)
    let q = |f: f64| { let x = (vals.len() - 1) as f64 * f; let (i, w) = (x.floor() as usize, x - x.floor()); vals[i] + (vals[(i + 1).min(vals.len() - 1)] - vals[i]) * w };
    let now = all.last().unwrap();
    let lab = |v: f64| val_label(v, suffix, 1);
    let left = ([q(qb), q(0.5), q(qa)].iter().map(|v| lab(*v).chars().count()).max().unwrap_or(4) as f64 * 8.4 + 12.0).max(L);
    let top = if note.is_empty() { T } else { T + 22.0 };
    let (lo, hi) = (vals[0].min(now.value), vals[vals.len() - 1].max(now.value));
    let pad = (hi - lo) * 0.06;
    let (lo, hi) = (lo - pad, hi + pad * 2.0); // 위 여백: 고점 이름표 자리
    let (x0, x1) = (drawn[0].day as f64, now.day as f64);
    let x = |d: f64| left + (d - x0) / (x1 - x0).max(1.0) * (W - left - R);
    let y = |v: f64| top + (hi - v) / (hi - lo).max(1e-9) * (H - top - B);
    let mut s = head(title);
    if !note.is_empty() {
        let _ = write!(s, "<text x=\"2\" y=\"{:.1}\" font-size=\"13\" fill=\"{INK}\">{}</text>", T + 2.0, note.replace('&', "&amp;")); // 그림 위 제목과 같은 왼쪽 선
    }
    let _ = write!(s, "<rect x=\"{left}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{SHADE}\"/>", y(q(qb)), W - left - R, y(q(qa)) - y(q(qb)));
    let _ = write!(s, "<line x1=\"{left}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R, y(q(0.5)), y(q(0.5)));
    let mut last_y = f64::MIN;
    for v in [q(qb), q(0.5), q(qa)] {
        if y(v) - last_y < 14.0 { continue; }
        last_y = y(v);
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" font-size=\"14\" fill=\"{INK}\">{}</text>", left - 6.0, y(v) + 4.0, lab(v));
    }
    let step = (drawn.len() / 500).max(1);
    let d: Vec<String> = drawn.iter().step_by(step).chain(std::iter::once(now)).map(|p| format!("{:.1},{:.1}", x(p.day as f64), y(p.value))).collect();
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{GRAY}\" stroke-width=\"1.3\"/>", d.join(" "));
    // 고점: 큰 값부터, 서로 6년 넘게 떨어진 것만(같은 사건의 이웃 달을 겹쳐 세지 않게), 지금과 2년 안은 뺀다(지금 점이 말한다)
    let mut order: Vec<&Point> = drawn.iter().collect();
    order.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
    let mut picked: Vec<&Point> = Vec::new();
    for p in order {
        if picked.len() >= peaks { break; }
        if (now.day - p.day) < 730 || picked.iter().any(|k| (k.day - p.day).abs() < 6 * 365) { continue; }
        picked.push(p);
    }
    if !years.is_empty() {  // 이름 붙일 해를 고르면(글이 말하는 사건) 그해 최고값에 단다: 자동 고점이 이웃 해를 고르는 일을 막는다
        // "low:2021"은 그해 최저값에, 이름표는 점 아래에
        picked = years.iter().filter_map(|yr| {
            let (low, yr) = match yr.strip_prefix("low:") { Some(y) => (true, y), None => (false, yr.as_str()) };
            // 이름표 값은 줄여 그린 점(달마다)이 아니라 원 자료에서 고른다: 주간 자료의 2022 최고가 달 표본보다 높았다(2026-11-26 휘발유)
            let it = all.iter().filter(|p| p.date.starts_with(yr));
            if low { it.min_by(|a, b| a.value.partial_cmp(&b.value).unwrap()) } else { it.max_by(|a, b| a.value.partial_cmp(&b.value).unwrap()) }
        }).collect();
    }
    let lows: Vec<bool> = years.iter().map(|y| y.starts_with("low:")).collect();
    let mut boxes: Vec<(f64, f64, f64, f64)> = Vec::new(); // 이름표 자리: '지금' 이름표가 겹치면 점 아래로 내린다
    for (i, p) in picked.iter().enumerate() {
        let px = x(p.day as f64).clamp(left + 18.0, W - R - 18.0);
        let py = if lows.get(i).copied().unwrap_or(false) { y(p.value) + 18.0 } else { y(p.value) - 7.0 };
        let half = (5 + lab(p.value).chars().count()) as f64 * 3.6;
        boxes.push((px - half, px + half, py - 13.0, py + 2.0));
        let _ = write!(s, "<text x=\"{px:.1}\" y=\"{py:.1}\" text-anchor=\"middle\" font-size=\"13\" fill=\"{INK}\" {HALO}>{} {}</text>", &p.date[..4], lab(p.value));
    }
    let (nx, ny) = (x(now.day as f64), y(now.value));
    let nw = (4 + lab(now.value).chars().count()) as f64 * 8.0;
    let hit = |ty: f64| boxes.iter().any(|&(a, b, c, d)| a < nx - 9.0 && b > nx - 9.0 - nw && c < ty + 2.0 && d > ty - 14.0);
    // 위·아래 중 이름표와 겹치지 않고 선을 덜 가리는 쪽(이름표 칸 안에 들어오는 선의 점 수)
    let cover = |ty: f64| drawn.iter().filter(|p| { let (px, py) = (x(p.day as f64), y(p.value)); px > nx - 9.0 - nw && px < nx - 9.0 && py > ty - 14.0 && py < ty + 2.0 }).count() + if hit(ty) { 1000 } else { 0 };
    let span: Vec<f64> = drawn.iter().filter(|p| x(p.day as f64) > nx - 9.0 - nw).map(|p| y(p.value)).collect();
    let (top_y, bot_y) = (span.iter().cloned().fold(ny, f64::min), span.iter().cloned().fold(ny, f64::max));
    let ty = [ny - 9.0, ny + 22.0, bot_y + 18.0, top_y - 6.0].into_iter().filter(|t| *t > T + 14.0 && *t < H - B - 2.0)
        .min_by_key(|t| cover(*t)).unwrap_or(ny - 9.0);
    // 이름표가 점에서 멀리 떨어지면(빈자리가 아래·위에만 있을 때) 점까지 가는 가는 선(독자: 'Now 3.9%'가 선 아래 엉뚱한 자리, 2026-11-26)
    if (ty - ny).abs() > 30.0 {
        let ly = if ty > ny { ty - 15.0 } else { ty + 4.0 };
        let _ = write!(s, "<line x1=\"{:.1}\" y1=\"{ly:.1}\" x2=\"{nx:.1}\" y2=\"{ny:.1}\" stroke=\"#0f172a\" stroke-width=\"1\"/>", nx - 12.0);
    }
    let _ = write!(s, "<circle cx=\"{nx:.1}\" cy=\"{ny:.1}\" r=\"5\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" font-size=\"14\" font-weight=\"600\" fill=\"#0f172a\" {HALO}>{now_label} {}</text>", nx - 9.0, ty, lab(now.value));
    // 가로축: 10년마다 눈금 글자, 양 끝 연도와 겹치면 뺀다
    let (y0, y1): (i32, i32) = (drawn[0].date[..4].parse().unwrap_or(0), now.date[..4].parse().unwrap_or(0));
    let _ = write!(s, "<text x=\"{left}\" y=\"{}\" font-size=\"14\" fill=\"{INK}\">{y0}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" font-size=\"14\" fill=\"{INK}\">{y1}</text>", H - 6.0, W - R, H - 6.0);
    let mut yr = (y0 / 10 + 1) * 10;
    while yr < y1 {
        let dx = x(crate::series::day_number(&format!("{yr}-01-01")).unwrap_or(0) as f64);
        if dx - left > 44.0 && W - R - dx > 64.0 {
            let _ = write!(s, "<text x=\"{dx:.1}\" y=\"{}\" text-anchor=\"middle\" font-size=\"14\" fill=\"{INK}\">{yr}</text>", H - 6.0);
        }
        yr += 10;
    }
    s.push_str("</svg>");
    s
}

/// 최근 그림(데이터 페이지 맨 위, 2026-11-13): 최근 N개 관측의 선, 그 기간의 최저·최고와 지금에 점과 값, 첫·가운데·끝 달 눈금.
/// '좋아지나 나빠지나'를 매주 보는 독자용(콜드 리드: 추적기라면 최근 흐름이 먼저 보여야 한다).
/// line: 기준선(값, 이름) 하나를 점선으로(예: 3% 대출 금리). 기간이 1년을 넘으면 최저·최고 이름표에 해를 붙인다(10년 그림에서 "Jan"만으로는 몇 년인지 몰랐다).
pub fn recent(pts: &[Point], title: &str, suffix: &str, decimals: usize, now_label: &str, line: Option<(f64, &str)>) -> String {
    const HALO: &str = "stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\"";
    const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let lab = |v: f64| val_label(v, suffix, decimals);
    let mo = |d: &str| format!("{} {}", MON[d[5..7].parse::<usize>().unwrap_or(1) - 1], &d[..4]);
    let lo = pts.iter().min_by(|a, b| a.value.partial_cmp(&b.value).unwrap()).unwrap();
    let hi = pts.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap()).unwrap();
    let now = pts.last().unwrap();
    let (vmin, vmax) = (lo.value, hi.value);
    let pad = (vmax - vmin).max(0.1) * 0.25;
    let (top, left, right) = (T + 4.0, 16.0, 16.0);
    let (x0, x1) = (pts[0].day as f64, now.day as f64);
    let x = |d: f64| left + (d - x0) / (x1 - x0).max(1.0) * (W - left - right - 70.0);
    let y = |v: f64| top + (vmax + pad - v) / (vmax - vmin + 2.0 * pad) * (H - top - B - 10.0);
    let mut s = head(title);
    if let Some((v, name)) = line {
        let ly = y(v);
        // 이름 자리: 오른쪽 위 → 오른쪽 아래 → 왼쪽 위 → 왼쪽 아래 중 선(자료)과 겹치지 않는 첫 자리(근원 물가 선이 이름을 지나갔다, 2026-11-26)
        let w = name.chars().count() as f64 * 7.0;
        let xr = W - right - 70.0;
        // 점 사이 선분도 잰다(점만 보면 선분이 이름을 지나갔다)
        let hits = |xa: f64, xb: f64, yb: f64| pts.windows(2).map(|w2| (0..=12).filter(|&k| {
            let f = k as f64 / 12.0;
            let (px, py) = (x(w2[0].day as f64 + (w2[1].day - w2[0].day) as f64 * f), y(w2[0].value + (w2[1].value - w2[0].value) * f));
            px >= xa - 6.0 && px <= xb + 6.0 && py >= yb - 16.0 && py <= yb + 5.0
        }).count()).sum::<usize>();
        // 네 자리가 모두 막히면 선을 따라 30씩 옮겨 가며 위·아래 빈자리를 찾는다
        let mut spots = vec![(xr, ly - 6.0, "end"), (xr, ly + 16.0, "end"), (left, ly - 6.0, "start"), (left, ly + 16.0, "start")];
        let mut sx = left + 30.0;
        while sx + w < xr { spots.push((sx, ly - 6.0, "start")); spots.push((sx, ly + 16.0, "start")); sx += 30.0; }
        // 빈자리가 없으면 겹침이 가장 적은 자리(앞쪽 후보 우선)
        let (lx, lyy, anchor) = spots.iter().cloned().min_by_key(|&(sx, sy, a)| { let (xa, xb) = if a == "end" { (sx - w, sx) } else { (sx, sx + w) }; hits(xa, xb, sy) }).unwrap_or(spots[0]);
        let _ = write!(s, "<line x1=\"{left}\" x2=\"{:.1}\" y1=\"{ly:.1}\" y2=\"{ly:.1}\" stroke=\"#eb6834\" stroke-width=\"1.5\" stroke-dasharray=\"4 3\"/><text x=\"{lx:.1}\" y=\"{lyy:.1}\" text-anchor=\"{anchor}\" font-size=\"13\" fill=\"#eb6834\" {HALO}>{}</text>", xr, name.replace('&', "&amp;"));
    }
    // 흐린 가로 눈금과 값(독자가 세로축이 없다고 짚었다, 2026-11-26 물가 추적기): 왼쪽 안쪽에 작게
    {
        let span = vmax - vmin + 2.0 * pad;
        let raw = span / 4.0;
        let mag = 10f64.powf(raw.log10().floor());
        let step = [1.0, 2.0, 2.5, 5.0, 10.0].iter().map(|m| m * mag).find(|st| *st >= raw).unwrap_or(10.0 * mag);
        let mut v = ((vmin - pad) / step).ceil() * step;
        while v <= vmax + pad + 1e-9 {
            let gy = y(v);
            let _ = write!(s, "<line x1=\"{left}\" x2=\"{:.1}\" y1=\"{gy:.1}\" y2=\"{gy:.1}\" stroke=\"#eef2f7\"/><text x=\"{left}\" y=\"{:.1}\" font-size=\"11\" fill=\"#94a3b8\" {HALO}>{}</text>", W - right - 70.0, gy - 3.0, val_label(v + 0.0, suffix, decimals.min(2)));
            v += step;
        }
    }
    let long = true; // 이름표에 늘 연도(독자: 'Low 2.3% (Apr)'가 어느 해인지 모름, 2026-11-26)
    let d: Vec<String> = pts.iter().map(|p| format!("{:.1},{:.1}", x(p.day as f64), y(p.value))).collect();
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{GRAY}\" stroke-width=\"2\"/>", d.join(" "));
    for (p, name, below) in [(lo, "Low", true), (hi, "High", false)] {
        if std::ptr::eq(p, now) { continue; }
        let (px, py) = (x(p.day as f64), y(p.value));
        let ty = if below { py + 22.0 } else { py - 10.0 };
        // 왼쪽·오른쪽 끝 가까운 점은 이름표가 잘렸다(2026-11-18 곡선 그림 'Low … (Dec 2024)'): 끝에서는 안쪽으로 붙인다
        let (anchor, tx) = if px < 100.0 { ("start", (px - 6.0).max(4.0)) } else if px > W - 170.0 { ("end", px + 6.0) } else { ("middle", px) };
        let _ = write!(s, "<circle cx=\"{px:.1}\" cy=\"{py:.1}\" r=\"4\" fill=\"{INK}\"/><text x=\"{tx:.1}\" y=\"{ty:.1}\" text-anchor=\"{anchor}\" font-size=\"14\" fill=\"{INK}\" {HALO}>{name} {} ({})</text>", lab(p.value), if long { mo(&p.date) } else { mo(&p.date)[..3].to_string() });
    }
    let (nx, ny) = (x(now.day as f64), y(now.value));
    let _ = write!(s, "<circle cx=\"{nx:.1}\" cy=\"{ny:.1}\" r=\"6\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" font-size=\"15\" font-weight=\"600\" fill=\"#0f172a\">{}</text><text x=\"{:.1}\" y=\"{:.1}\" font-size=\"15\" font-weight=\"600\" fill=\"#0f172a\">{}</text>", nx + 10.0, ny - 2.0, now_label.replace('&', "&amp;"), nx + 10.0, ny + 16.0, lab(now.value));
    let mid = &pts[pts.len() / 2];
    for (p, anchor) in [(&pts[0], "start"), (mid, "middle"), (now, "end")] {
        let px = if anchor == "end" { W - right } else { x(p.day as f64) };
        let _ = write!(s, "<text x=\"{px:.1}\" y=\"{}\" text-anchor=\"{anchor}\" font-size=\"14\" fill=\"{INK}\">{}</text>", H - 6.0, mo(&p.date));
    }
    s.push_str("</svg>");
    s
}

/// 사례 점 그림: 사례마다 점 하나(가로 = 시작 연도, 세로 = 깊이). 기준선 하나, 지금 사례만 파랑(Cleveland & McGill: 같은 축 위 위치).
/// marks: 이름을 붙일 점(날, 깊이, 해). 기준선·지금 점·가장 깊은 점에 이름표: 그림만 봐도 읽히게(2026-11-17 콜드 리드 두 명이 '이름 없는 점'을 짚음)
pub fn dots(items: &[(i64, f64, bool)], line: f64, first_year: &str, last_year: &str, title: &str, marks: &[(i64, f64, String)]) -> String {
    let lo = items.iter().map(|i| i.1).fold(0.0, f64::min);
    let lo = (lo * 10.0).floor() / 10.0;
    let (x0, x1) = (items.iter().map(|i| i.0).min().unwrap_or(0) as f64, items.iter().map(|i| i.0).max().unwrap_or(1) as f64);
    let x = |d: f64| L + 6.0 + (d - x0) / (x1 - x0).max(1.0) * (W - L - R - 12.0);
    let y = |v: f64| T + (0.0 - v) / (0.0 - lo).max(1e-9) * (H - T - B);
    let mut s = head(title);
    let mut v = 0.0;
    while v >= lo - 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#e5e7eb\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", W - R, y(v), y(v), L - 6.0, y(v) + 4.0, pct(v));
        v -= 0.2;
    }
    let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R, y(-line), y(-line));
    let halo = "stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\"";
    let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\" {halo}>Bear market: {}% down</text>", L + 4.0, y(-line) + 16.0, (line * 100.0).round());
    for &(d, v, now) in items {
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{}\" fill=\"{}\" fill-opacity=\"{}\"/>", x(d as f64), y(v), if now { 6 } else { 4 }, if now { BLUE } else { GRAY }, if now { 1.0 } else { 0.8 });
        if now {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{BLUE}\" font-weight=\"600\" font-size=\"14\" {halo}>This drop so far {}</text>", x(d as f64) - 10.0, y(v) - 8.0, pct(v));
        }
    }
    for (d, v, yr) in marks {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\" {halo}>{yr} {}</text>", (x(*d as f64) + 8.0).min(W - R - 70.0), y(*v) + 4.0, pct(*v));
    }
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\">{first_year}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{last_year}</text></svg>", H - 6.0, W - R, H - 6.0);
    s
}

/// 사분면 점 그림(quad): 달마다 (x의 n개월 변화, y의 n개월 변화율) 한 점, 0 선 두 개, x의 ±at 점선,
/// 네 칸마다 점 수, 마지막 점(지금)은 파란 점과 이름표. xlab/ylab은 축 이름(축 끝에 짧게).
pub fn quad(pts: &[(String, f64, f64)], at: f64, xlab: &str, ylab: &str, xunit: &str, title: &str) -> String {
    let (xs, ys): (Vec<f64>, Vec<f64>) = pts.iter().map(|p| (p.1, p.2)).unzip();
    let ext = |v: &[f64]| v.iter().fold(0.0f64, |a, b| a.max(b.abs()));
    let (xm, ym) = ((ext(&xs) * 10.0).ceil() / 10.0, (ext(&ys) * 10.0).ceil() / 10.0);
    let (left, top, bot) = (L + 8.0, T + 18.0, B + 10.0);
    let x = |v: f64| left + (v + xm) / (2.0 * xm) * (W - left - R);
    let y = |v: f64| top + (ym - v) / (2.0 * ym) * (H - top - bot);
    let halo = "stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\"";
    let mut s = head(title);
    for v in [-ym, -ym / 2.0, 0.0, ym / 2.0, ym] {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text>", left - 6.0, y(v) + 4.0, if v > 0.0 { format!("+{}", pct(v)) } else { pct(v) });
    }
    let _ = write!(s, "<line x1=\"{left}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\"/><line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{top}\" y2=\"{:.1}\" stroke=\"{INK}\"/>", W - R, y(0.0), y(0.0), x(0.0), x(0.0), H - bot);
    for a in [-at, at] {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{top}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", x(a), x(a), H - bot);
    }
    let n = pts.len().saturating_sub(1);
    for p in &pts[..n] {
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"2.6\" fill=\"{GRAY}\" fill-opacity=\"0.7\"/>", x(p.1), y(p.2));
    }
    // 점선 바깥 두 칸의 점 수와 오른 몫(왼쪽: x가 at 넘게 내린 달, 오른쪽: at 넘게 오른 달)
    let side = |f: &dyn Fn(f64) -> bool| { let v: Vec<f64> = pts[..n].iter().filter(|p| f(p.1)).map(|p| p.2).collect(); (v.len(), v.iter().filter(|&&g| g > 0.0).count()) };
    let (ln, lu) = side(&|v| v <= -at);
    let (rn, ru) = side(&|v| v >= at);
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\" {halo}>{lu} of {ln} up</text>", left + 4.0, top - 4.0);
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\" {halo}>{ru} of {rn} up</text>", W - R - 2.0, top - 4.0);
    if let Some(p) = pts.last() {
        let (cx, cy) = (x(p.1), y(p.2));
        let anchor = if cx > W / 2.0 { "end" } else { "start" };
        let dx = if cx > W / 2.0 { -10.0 } else { 10.0 };
        let _ = write!(s, "<circle cx=\"{cx:.1}\" cy=\"{cy:.1}\" r=\"6\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"#0f172a\" font-weight=\"600\" font-size=\"14\" {halo}>Now</text>", cx + dx, cy + 5.0);
    }
    let _ = write!(s, "<text x=\"{left}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\">{xlab} \u{2190} fell</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">rose \u{2192} {}{xunit}</text>", H - 6.0, W - R, H - 6.0, if at > 0.0 { format!("dashed \u{b1}{at} ") } else { String::new() });
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"12\" {halo}>{ylab}</text></svg>", left + 4.0, y(0.0) - 6.0);
    s
}

/// 사례 줄 그림: 사례마다 한 줄(위에서 아래로), 같은 가로축(변화율) 위에 도중 최저점·첫 시점·둘째 시점을 점으로.
/// 최저점에서 끝 점까지 옅은 선이 '도중에 빠졌다가 어디서 끝났나'를 보여 준다. 세로 점선은 기저율(B5: 같은 축 위치, A5: 평소와 견준다).
/// rows: (이름, 도중 최저, 첫 시점 값, 둘째 시점 값). names: [최저, 첫 시점, 둘째 시점] 범례 이름.
pub fn cases(rows: &[(String, f64, f64, Option<f64>)], base: f64, names: [&str; 3], title: &str) -> String {
    const ORANGE: &str = "#eb6834";
    // 왼쪽 이름표 길이에 맞춘 여백("Facebook 2012"가 잘렸다)
    let left = (rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(4) as f64 * 8.6 + 12.0).max(56.0);
    let (rh, top) = (24.0, 62.0);
    let h = top + rh * rows.len() as f64 + 30.0;
    let vals: Vec<f64> = rows.iter().flat_map(|r| [r.1.min(0.0), r.2, r.3.unwrap_or(0.0)]).collect();
    let lo = (vals.iter().cloned().fold(0.0, f64::min) * 10.0).floor() / 10.0;
    // 튀는 값 하나가 나머지를 한쪽에 몰지 않게: 가장 큰 값이 둘째의 두 배를 넘으면 축을 둘째 근처에서 끊고 그 점은 끝에 실제 값과 함께 둔다
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let cap = match sorted.as_slice() { [a, b, ..] if *a > 0.5 && *a > 2.0 * b.max(0.05) => Some(b.max(0.05)), _ => None };
    let hi = (cap.unwrap_or(sorted.first().copied().unwrap_or(0.0).max(0.0)) * 10.0 + 1.0).floor() / 10.0;
    let x = |v: f64| left + (v.min(hi) - lo) / (hi - lo).max(0.01) * (W - left - R);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    // 범례: 표시는 색, 글자는 잉크색. 이름이 빈 항목은 뺀다, 간격은 이름 길이로
    let mut lx = left;
    for (c, n) in [ORANGE, BLUE, GRAY].iter().zip(names).filter(|(_, n)| !n.is_empty()) {
        let _ = write!(s, "<circle cx=\"{}\" cy=\"18\" r=\"6\" fill=\"{c}\"/><text x=\"{}\" y=\"24\" fill=\"{INK}\">{}</text>", lx + 6.0, lx + 16.0, n.replace('&', "&amp;"));
        lx += 16.0 + n.chars().count() as f64 * 8.6 + 20.0;
    }
    // 눈금은 6개 이하(휴대폰에서 숫자가 겹치지 않게)
    let step = [0.05, 0.1, 0.2, 0.25, 0.5, 1.0].into_iter().find(|st| (hi - lo) / st <= 5.0).unwrap_or(1.0);
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{}\" y2=\"{}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" fill=\"{INK}\">{}</text>",
                       x(v), x(v), top - 8.0, h - 26.0, if v.abs() < 1e-9 { INK } else { "#e5e7eb" }, x(v), h - 8.0, pct(v));
        v += step;
    }
    if !base.is_nan() {
    let base_label = if names[1].chars().count() <= 12 { format!("typical {}", names[1]) } else { "median".to_string() };
    let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{}\" y2=\"{}\" stroke=\"{BLUE}\" stroke-dasharray=\"4 3\"/><text x=\"{:.1}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\">{}</text>", x(base), x(base), top - 8.0, h - 26.0, x(base) + 4.0, top - 12.0, base_label);
    }
    for (k, (name, w, a, b)) in rows.iter().enumerate() {
        let y = top + rh * k as f64 + rh / 2.0;
        let ends = [w.min(0.0), *a, b.unwrap_or(*a)];
        let (l, r) = (ends.iter().cloned().fold(f64::MAX, f64::min), ends.iter().cloned().fold(f64::MIN, f64::max));
        let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text><line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{y:.1}\" y2=\"{y:.1}\" stroke=\"#cbd5e1\" stroke-width=\"2\"/>",
                       left - 8.0, y + 5.0, name.replace('&', "&amp;"), x(l), x(r));
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"{ORANGE}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(w.min(0.0)));
        if let Some(b) = b {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"{GRAY}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(*b));
        }
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"6\" fill=\"{BLUE}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(*a));
        if *a > hi {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">+{} →</text>", x(*a) - 10.0, y + 5.0, pct(*a));
        }
    }
    s.push_str("</svg>");
    s
}

/// 그림문자(ISOTYPE) 그림: 묶음마다 단위 점 total개(10개씩 줄), 그중 filled개를 칠한다. "100달러 중 17달러가 남는다"처럼
/// 비율을 셀 수 있는 물건으로 보여 준다(Haroz·Kosara·Franconeri 2015: 그림문자는 정확도를 해치지 않고 기억·몰입을 높인다).
/// groups: (이름, 칠할 수, 전체 수, 큰 글씨 값). 두 묶음까지 나란히, 그 이상은 아래로.
pub fn pictogram(groups: &[(String, usize, usize, String)], title: &str) -> String {
    let (cell, cols) = (18.0, 10usize);
    let per_row = if groups.len() <= 2 { groups.len().max(1) } else { 2 };
    let gw = cell * cols as f64;
    let rows_per = groups.iter().map(|g| g.2.div_ceil(cols)).max().unwrap_or(1) as f64;
    let gh = 44.0 + rows_per * cell + 30.0;
    let h = 8.0 + gh * groups.len().div_ceil(per_row) as f64;
    let gap = (W - gw * per_row as f64) / (per_row as f64 + 1.0);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    for (k, (name, filled, total, big)) in groups.iter().enumerate() {
        let x0 = gap + (k % per_row) as f64 * (gw + gap);
        let y0 = 8.0 + (k / per_row) as f64 * gh;
        let color = if k == 0 { BLUE } else { "#eb6834" };
        let _ = write!(s, "<text x=\"{x0:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"28\" font-weight=\"700\">{}</text>", y0 + 28.0, big.replace('&', "&amp;"));
        for i in 0..*total {
            let (cx, cy) = (x0 + (i % cols) as f64 * cell + cell / 2.0, y0 + 44.0 + (i / cols) as f64 * cell + cell / 2.0);
            let _ = write!(s, "<circle cx=\"{cx:.1}\" cy=\"{cy:.1}\" r=\"6.5\" fill=\"{}\"/>", if i < *filled { color } else { SHADE });
        }
        let _ = write!(s, "<text x=\"{x0:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"15\">{}</text>", y0 + 44.0 + rows_per * cell + 22.0, name.replace('&', "&amp;"));
    }
    s.push_str("</svg>");
    s
}

/// 세로 시간선: 사건마다 해·점·굵은 한 줄·결과 한두 줄. 점 색은 결과의 성격(좋음 초록·중간 노랑·나쁨 주황)이고 글자가 그 뜻을 함께 말한다.
/// 사례 결말이 제각각인 이야기(거래소 사고 뒤 이용자 돈)를 표보다 순서대로 읽히게. items: (해, 사건, 결과, 성격 good|mixed|bad).
pub fn timeline(items: &[(String, String, String, String)], title: &str) -> String {
    fn wrap(t: &str, n: usize) -> Vec<String> {
        let mut out: Vec<String> = vec![String::new()];
        for w in t.split_whitespace() {
            if out.last().unwrap().len() + w.len() + 1 > n && !out.last().unwrap().is_empty() { out.push(String::new()); }
            let l = out.last_mut().unwrap();
            if !l.is_empty() { l.push(' '); }
            l.push_str(w);
        }
        out
    }
    // 왼쪽 이름표(해·날짜) 길이에 맞춰 여백을 넓힌다("Sept. 15", "Summer"가 잘렸다)
    let lw = items.iter().map(|i| i.0.chars().count()).max().unwrap_or(4) as f64 * 9.0 + 10.0;
    let (xl, xd, xt) = (lw.max(52.0), lw.max(52.0) + 22.0, lw.max(52.0) + 40.0);
    let narrow = ((xt - 92.0) / 8.0) as usize;
    let blocks: Vec<(Vec<String>, Vec<String>)> = items.iter().map(|i| (wrap(&i.1, 40 - narrow.min(12)), wrap(&i.2, 46 - narrow.min(12)))).collect();
    let hs: Vec<f64> = blocks.iter().map(|(a, b)| 12.0 + a.len() as f64 * 20.0 + b.len() as f64 * 19.0 + 14.0).collect();
    let h = 16.0 + hs.iter().sum::<f64>();
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let _ = write!(s, "<line x1=\"{xd}\" x2=\"{xd}\" y1=\"20\" y2=\"{:.1}\" stroke=\"#e2e8f0\" stroke-width=\"2\"/>", h - 14.0);
    let mut y = 16.0;
    for ((year, _, _, tone), ((a, b), bh)) in items.iter().zip(blocks.iter().zip(&hs)) {
        let c = match tone.as_str() { "good" => "#1baf7a", "bad" => "#eb6834", _ => "#eda100" };
        let _ = write!(s, "<text x=\"{xl}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-weight=\"600\">{}</text><circle cx=\"{xd}\" cy=\"{:.1}\" r=\"7\" fill=\"{c}\" stroke=\"#fff\" stroke-width=\"3\"/>",
                       y + 16.0, year.replace('&', "&amp;"), y + 11.0);
        let mut ty = y + 16.0;
        for l in a { let _ = write!(s, "<text x=\"{xt}\" y=\"{ty:.1}\" fill=\"#0f172a\" font-weight=\"600\">{}</text>", l.replace('&', "&amp;")); ty += 20.0; }
        for l in b { let _ = write!(s, "<text x=\"{xt}\" y=\"{ty:.1}\" fill=\"{INK}\" font-size=\"15\">{}</text>", l.replace('&', "&amp;")); ty += 19.0; }
        y += bh;
    }
    s.push_str("</svg>");
    s
}

/// 달력 열지도: 해(줄) × 달(칸), 칸 색은 그해 평균 대비(아래 파랑, 위 주황, 0 근처 회색: 발산 색은 두 색 + 회색 가운데).
/// 맨 아래 줄은 달별 중앙값과 숫자. 계절이 해마다 되풀이되는지 한눈에(달력 열지도: Van Wijk & Van Selow 1999).
pub fn calendar(years: &[(i32, [f64; 12])], typical: &[f64], scale: f64, title: &str) -> String {
    let (left, top, cw) = (62.0, 30.0, (W - 62.0 - R) / 12.0);
    let rh = 11.0;
    let h = top + rh * years.len() as f64 + 10.0 + 30.0 + 26.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let color = |v: f64| {
        let t = (v / scale).clamp(-1.0, 1.0);
        let (r0, g0, b0) = (241.0, 245.0, 249.0);
        let (r1, g1, b1) = if t < 0.0 { (37.0, 99.0, 235.0) } else { (235.0, 104.0, 52.0) };
        let a = t.abs();
        format!("rgb({:.0},{:.0},{:.0})", r0 + (r1 - r0) * a, g0 + (g1 - g0) * a, b0 + (b1 - b0) * a)
    };
    for (k, m) in ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"].iter().enumerate() {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{m}</text>", left + cw * (k as f64 + 0.5), top - 10.0);
    }
    for (r, (y, dev)) in years.iter().enumerate() {
        let yy = top + rh * r as f64;
        if y % 5 == 0 || r == 0 || r + 1 == years.len() {
            let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"12\">{y}</text>", left - 6.0, yy + 9.5);
        }
        for (k, v) in dev.iter().enumerate() {
            let _ = write!(s, "<rect x=\"{:.1}\" y=\"{yy:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\"/>", left + cw * k as f64 + 1.0, cw - 2.0, rh - 1.5, color(*v));
        }
    }
    let ty = top + rh * years.len() as f64 + 10.0;
    let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"12\" font-weight=\"600\">Typical</text>", left - 6.0, ty + 19.0);
    for (k, v) in typical.iter().enumerate() {
        let x = left + cw * k as f64 + 1.0;
        let _ = write!(s, "<rect x=\"{x:.1}\" y=\"{ty:.1}\" width=\"{:.1}\" height=\"28\" rx=\"3\" fill=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{}\" font-size=\"12\" font-weight=\"600\">{}</text>",
                       cw - 2.0, color(*v), x + (cw - 2.0) / 2.0, ty + 19.0, if (v / scale).abs() > 0.5 { "#fff" } else { "#0f172a" }, if *v > 0.0 { format!("+{}", pct(*v)) } else { pct(*v) });
    }
    let ly = ty + 28.0 + 20.0;
    let _ = write!(s, "<rect x=\"{left}\" y=\"{:.1}\" width=\"14\" height=\"10\" fill=\"{}\"/><text x=\"{}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Cheaper than that year's average</text><rect x=\"{}\" y=\"{:.1}\" width=\"14\" height=\"10\" fill=\"{}\"/><text x=\"{}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Pricier</text>",
                   ly - 9.0, color(-scale), left + 20.0, ly, left + 260.0, ly - 9.0, color(scale), left + 280.0, ly);
    s.push_str("</svg>");
    s
}

/// 게이지: 반원 위 바늘 하나로 "지금은 역사 범위의 어디"(백분위). 양끝에 최저·최고 값과 해, 가운데 큰 글씨로 지금 값과 백분위.
/// 위치 하나를 길게 설명하는 대신 한눈에(작은 다이얼은 정확한 비교보다 '대략 어디'를 말할 때만 쓴다).
pub fn gauge(rank: f64, now_label: &str, lo_label: &str, hi_label: &str, title: &str) -> String {
    let (cx, cy, r) = (W / 2.0, 180.0, 150.0);
    let h = 262.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let pt = |f: f64, rr: f64| { let a = std::f64::consts::PI * (1.0 - f); (cx + rr * a.cos(), cy - rr * a.sin()) };
    // 세 구간(하위 3분의 1 파랑, 가운데 회색, 상위 3분의 1 주황)
    for (a, b, c) in [(0.0, 1.0 / 3.0, "#bfdbfe"), (1.0 / 3.0, 2.0 / 3.0, "#e2e8f0"), (2.0 / 3.0, 1.0, "#fed7aa")] {
        let (x0, y0) = pt(a, r);
        let (x1, y1) = pt(b, r);
        let _ = write!(s, "<path d=\"M{x0:.1},{y0:.1} A{r},{r} 0 0 1 {x1:.1},{y1:.1}\" fill=\"none\" stroke=\"{c}\" stroke-width=\"26\"/>");
    }
    let (nx, ny) = pt(rank.clamp(0.0, 1.0), r - 6.0);
    let _ = write!(s, "<line x1=\"{cx}\" y1=\"{cy}\" x2=\"{nx:.1}\" y2=\"{ny:.1}\" stroke=\"#0f172a\" stroke-width=\"3\" stroke-linecap=\"round\"/><circle cx=\"{cx}\" cy=\"{cy}\" r=\"7\" fill=\"#0f172a\"/>");
    let _ = write!(s, "<text x=\"{cx}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"#0f172a\" font-size=\"22\" font-weight=\"700\">{}</text>", cy + 64.0, now_label.replace('&', "&amp;"));
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{}</text><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{}</text>",
                   cx - r, cy + 26.0, lo_label.replace('&', "&amp;"), cx + r, cy + 26.0, hi_label.replace('&', "&amp;"));
    s.push_str("</svg>");
    s
}

/// 작은 그림 여러 개(small multiples): 사례마다 작은 선 하나, 모두 같은 축(고점 = 0에서 days일). 칸마다 해·바닥 값·끝 값.
/// 사례끼리 모양을 나란히 견준다(Tufte 1983: 같은 틀 안에서 바뀌는 것만 보이게). 마지막 칸(지금)은 파랑.
pub fn multiples(cases: &[(String, Vec<(f64, f64)>)], days: f64, title: &str) -> String {
    let cols = 2usize;
    let (cw, chh, gap) = ((W - 16.0) / cols as f64, 120.0, 10.0);
    let rows = cases.len().div_ceil(cols);
    let h = rows as f64 * (chh + gap) + 8.0;
    let lo = cases.iter().flat_map(|c| c.1.iter().map(|p| p.1)).fold(0.0, f64::min).max(-1.0);
    let hi = cases.iter().flat_map(|c| c.1.iter().map(|p| p.1)).fold(0.0, f64::max).min(1.0); // 튀는 사례 하나(+639%)가 나머지를 납작하게 만들지 않게 위는 +100%에서 자른다(끝 값 글자는 실제 값)
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    for (k, (name, pts)) in cases.iter().enumerate() {
        let (x0, y0) = (8.0 + (k % cols) as f64 * cw, 4.0 + (k / cols) as f64 * (chh + gap));
        let (px0, px1, py0, py1) = (x0 + 6.0, x0 + cw - 50.0, y0 + 26.0, y0 + chh - 10.0);
        let x = |d: f64| px0 + d / days * (px1 - px0);
        let y = |v: f64| py0 + (hi - v.clamp(lo, hi)) / (hi - lo).max(0.01) * (py1 - py0);
        let now = k + 1 == cases.len();
        let c = if now { BLUE } else { INK };
        let _ = write!(s, "<text x=\"{px0:.1}\" y=\"{:.1}\" fill=\"{c}\" font-size=\"15\" font-weight=\"600\">{}</text>", y0 + 16.0, name.replace('&', "&amp;"));
        let _ = write!(s, "<line x1=\"{px0:.1}\" x2=\"{px1:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#cbd5e1\"/>", y(0.0), y(0.0));
        let d: Vec<String> = pts.iter().map(|(a, b)| format!("{:.1},{:.1}", x(*a), y(*b))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>", d.join(" "), if now { BLUE } else { "#64748b" });
        if let Some(low) = pts.iter().cloned().reduce(|a, b| if b.1 < a.1 { b } else { a }) {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"3\" fill=\"#eb6834\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"12\" text-anchor=\"middle\">{}</text>", x(low.0), y(low.1), x(low.0), (y(low.1) + 15.0).min(py1 + 8.0), pct(low.1));
        }
        if let Some(end) = pts.last() {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{c}\" font-size=\"13\" font-weight=\"600\">{}{}</text>", px1 + 4.0, y(end.1) + 4.0, if end.1 > 0.0 { "+" } else { "" }, pct(end.1));
        }
    }
    s.push_str("</svg>");
    s
}

/// 띠 점 그림(strip plot): 모든 사례를 한 축 위의 점으로, 강조 묶음은 파랑(이름표), 나머지는 회색. 묶음마다 중앙값 세로선.
/// "이 묶음이 나머지보다 높은가"를 개별 점과 함께 보여 준다(평균 막대 두 개보다 분포가 보인다, Weissgerber 외 2015).
pub fn strip(rows: &[(String, f64, bool)], names: [&str; 2], axis: Option<&str>, title: &str) -> String {
    let (top, left, rr) = (26.0, 16.0, R + 8.0);
    // 가로축 이름(선택): 점이 무엇을 재는지 그림만 보고 알게(2026-11-11 콜드 리드 두 독자: '0%~10%가 무엇인지 모르겠다')
    let h = if axis.is_some() { 222.0 } else { 200.0 };
    let tick_y = 190.0;
    let vals: Vec<f64> = rows.iter().map(|r| r.1).collect();
    let lo = (vals.iter().cloned().fold(0.0, f64::min) * 20.0).floor() / 20.0;
    let hi = (vals.iter().cloned().fold(0.0, f64::max) * 20.0).ceil() / 20.0;
    let x = |v: f64| left + (v - lo) / (hi - lo).max(0.01) * (W - left - rr);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let (yo, yh) = (top + 40.0, top + 110.0); // 회색 줄, 파랑 줄
    let step = [0.02, 0.05, 0.1, 0.2].into_iter().find(|st| (hi - lo) / st <= 6.0).unwrap_or(0.2);
    let signed = lo < 0.0; // 모두 0 이상(오차 크기 등)이면 + 부호를 붙이지 않는다
    let fmt = |v: f64| if hi - lo <= 0.12 { let r = (v * 1000.0).round() / 10.0; format!("{}%", if r == 0.0 { 0.0 } else { r }) } else { pct(v) }; // 좁은 범위는 소수 한 자리
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{top}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{}</text>",
                       x(v), x(v), tick_y - 18.0, if v.abs() < 1e-9 { INK } else { "#e5e7eb" }, x(v), tick_y, if v > 0.0 && signed { format!("+{}", pct(v)) } else { pct(v) });
        v += step;
    }
    for (grp, y, c, name) in [(false, yo, GRAY, names[1]), (true, yh, BLUE, names[0])] {
        let g: Vec<f64> = rows.iter().filter(|r| r.2 == grp).map(|r| r.1).collect();
        let _ = write!(s, "<text x=\"{left}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">{}</text>", y - 22.0, name.replace('&', "&amp;"));
        for r in rows.iter().filter(|r| r.2 == grp) {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"6\" fill=\"{c}\" fill-opacity=\"0.75\" stroke=\"#fff\" stroke-width=\"1.5\"><title>{}</title></circle>", x(r.1), r.0);
        }
        if let Some(m) = crate::stats::median(&g) {
            let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{c}\" stroke-width=\"2.5\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\" font-weight=\"600\">median {}{}</text>",
                           x(m), x(m), y - 14.0, y + 14.0, x(m) + 5.0, y + 28.0, if m > 0.0 && signed { "+" } else { "" }, fmt(m));
        }
    }
    if let Some(a) = axis {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{}</text>", (left + W - rr) / 2.0, h - 6.0, a.replace('&', "&amp;"));
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod strip_test {
    use super::*;

    #[test]
    fn groups_and_medians() {
        let r = vec![("VA".into(), 0.12, true), ("PA".into(), 0.12, true), ("TX".into(), 0.03, false), ("CA".into(), -0.04, false)];
        let svg = strip(&r, ["PJM", "Other"], None, "t");
        assert_eq!(svg.matches("<circle").count(), 4);
        assert!(svg.contains("median +12%") && svg.contains("PJM"));
    }
}

#[cfg(test)]
mod multiples_test {
    use super::*;

    #[test]
    fn one_panel_per_case() {
        let c = vec![("2000".to_string(), vec![(0.0, 0.0), (100.0, -0.8), (200.0, -0.5)]), ("Now".to_string(), vec![(0.0, 0.0), (50.0, -0.4)])];
        let svg = multiples(&c, 200.0, "t");
        assert_eq!(svg.matches("<polyline").count(), 2);
        assert!(svg.contains("-80%") && svg.contains(BLUE));
    }
}

#[cfg(test)]
mod gauge_test {
    use super::*;

    #[test]
    fn needle_points_right_for_high_rank() {
        let svg = gauge(0.9, "Now", "Low", "High", "t");
        assert!(svg.contains("Now") && svg.matches("<path").count() == 3);
        // 바늘 끝 x가 가운데보다 오른쪽
        let i = svg.find("x2=\"").unwrap() + 4;
        let x: f64 = svg[i..].split('"').next().unwrap().parse().unwrap();
        assert!(x > W / 2.0);
    }
}

#[cfg(test)]
mod timeline_test {
    use super::*;

    #[test]
    fn one_dot_per_event_with_tone_color() {
        let it = vec![("2014".to_string(), "Mt. Gox collapses".to_string(), "Repayments start in 2024".to_string(), "bad".to_string()),
                      ("2019".to_string(), "Binance hot wallet hack".to_string(), "Covered in full".to_string(), "good".to_string())];
        let svg = timeline(&it, "t");
        assert_eq!(svg.matches("<circle").count(), 2);
        assert!(svg.contains("#eb6834") && svg.contains("#1baf7a") && svg.contains("Mt. Gox"));
    }
}

#[cfg(test)]
mod pictogram_test {
    use super::*;

    #[test]
    fn fills_the_right_number_of_units() {
        let svg = pictogram(&[("Fund".into(), 17, 100, "$17".into()), ("Portfolio".into(), 83, 100, "$83".into())], "t");
        assert_eq!(svg.matches(BLUE).count(), 17);
        assert_eq!(svg.matches("#eb6834").count(), 83);
        assert!(svg.contains("$83") && svg.contains("Portfolio"));
    }
}

#[cfg(test)]
mod cases_test {
    use super::*;

    #[test]
    fn one_row_per_case() {
        let r = vec![("1974".to_string(), -0.12, 0.2, Some(0.3)), ("1986".to_string(), -0.02, 0.1, Some(-0.12))];
        let svg = cases(&r, 0.07, ["Worst dip", "6 months", "1 year"], "t");
        assert_eq!(svg.matches("<circle").count(), 3 + 6);
        assert!(svg.contains("1986") && svg.contains("stroke-dasharray"));
    }
}

#[cfg(test)]
mod more_frames {
    use super::*;
    use crate::series::parse;

    #[test]
    fn band_and_dots() {
        let p = parse("d,v\n2000-01-01,1\n2001-01-01,3\n2002-01-01,2\n");
        assert!(band(&p, "t", false, "%").contains("<circle"));
        let d = dots(&[(0, -0.15, false), (400, -0.3, true)], 0.2, "1971", "2026", "t", &[(0, -0.15, "1971".to_string())]);
        assert!(d.matches("<circle").count() == 2 && d.contains("#2563eb"));
    }
}

/// 반응 경로 그림: 사건(선행 지표가 움직이기 시작한 날)을 0일로 맞추고 후행 지표의 변화 경로를 그룹마다 중앙값 선 + 사분위 띠로.
/// groups: (이름, 색, 사례마다 offsets 길이의 경로). 같은 축에서 오를 때·내릴 때의 모양(빠르기)을 비교한다(B5: 같은 축 위치).
/// end_values: 끝 이름표 아래 값. 표가 같은 값을 다른 표본(날짜 기준)으로 따로 줄 때는 끈다(--no-end-values: 주 단위 끝값 +14%와 표의 2년 +16.2%가 달라 보였다, 2026-11-18)
pub fn response(groups: &[(String, &str, Vec<Vec<f64>>)], offsets: &[i64], title: &str, end_values: bool) -> String {
    let q = |v: &mut Vec<f64>, f: f64| -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[((v.len() - 1) as f64 * f).round() as usize] };
    // offsets마다 (25%, 50%, 75%)
    let stats: Vec<Vec<(f64, f64, f64)>> = groups.iter().map(|(_, _, paths)| {
        (0..offsets.len()).map(|k| {
            let mut col: Vec<f64> = paths.iter().filter_map(|p| p.get(k).copied()).filter(|x| x.is_finite()).collect();
            if col.is_empty() { (0.0, 0.0, 0.0) } else { (q(&mut col, 0.25), q(&mut col, 0.5), q(&mut col, 0.75)) }
        }).collect()
    }).collect();
    let all: Vec<f64> = stats.iter().flatten().flat_map(|t| [t.0, t.2]).collect();
    let (mut lo, mut hi) = (all.iter().cloned().fold(0.0, f64::min), all.iter().cloned().fold(0.0, f64::max));
    lo = (lo * 20.0).floor() / 20.0;
    hi = (hi * 20.0).ceil() / 20.0;
    let xmax = *offsets.last().unwrap() as f64;
    let x = |d: f64| L + d / xmax * (W - L - R - 84.0);
    let y = |v: f64| T + (hi - v) / (hi - lo).max(0.01) * (H - T - B);
    let mut s = head(title);
    let step = if hi - lo > 0.3 { 0.1 } else { 0.05 };
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>",
                       W - R - 84.0, y(v), y(v), if v.abs() < 1e-9 { INK } else { "#e5e7eb" }, L - 6.0, y(v) + 4.0, pct(v));
        v += step;
    }
    // 끝 이름표 자리: 두 묶음의 끝값이 가까우면 이름표가 겹쳤다(2026-11-18 역전 뒤 주식 그림). 34px 이상 벌린다
    let mut ly: Vec<(usize, f64)> = stats.iter().enumerate().map(|(g, st)| (g, y(st.last().unwrap().1))).collect();
    ly.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for k in 1..ly.len() { if ly[k].1 - ly[k - 1].1 < 34.0 { ly[k].1 = ly[k - 1].1 + 34.0; } }
    let label_y = |g: usize| ly.iter().find(|(i, _)| *i == g).map(|t| t.1).unwrap_or(0.0);
    for (g, (name, color, _)) in groups.iter().enumerate() {
        let st = &stats[g];
        let top: Vec<String> = offsets.iter().zip(st).map(|(d, t)| format!("{:.1},{:.1}", x(*d as f64), y(t.2))).collect();
        let bot: Vec<String> = offsets.iter().zip(st).rev().map(|(d, t)| format!("{:.1},{:.1}", x(*d as f64), y(t.0))).collect();
        let _ = write!(s, "<polygon points=\"{} {}\" fill=\"{color}\" fill-opacity=\"0.12\"/>", top.join(" "), bot.join(" "));
        let mid: Vec<String> = offsets.iter().zip(st).map(|(d, t)| format!("{:.1},{:.1}", x(*d as f64), y(t.1))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2.5\"/>", mid.join(" "));
        let last = st.last().unwrap().1;
        let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" fill=\"{color}\" font-weight=\"600\">{}</text><text x=\"{}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{}{}</text>",
                       W - R - 80.0, label_y(g) - 4.0, name.replace('&', "&amp;"), W - R - 80.0, label_y(g) + 13.0, if !end_values { "" } else if last > 0.0 { "+" } else { "" }, if end_values { pct(last) } else { String::new() });
    }
    // 띠의 뜻을 그림 안에(두 독자가 띠 범례가 없다고 짚었다, 2026-11-18)
    let _ = write!(s, "<rect x=\"{}\" y=\"{}\" width=\"14\" height=\"10\" fill=\"{INK}\" fill-opacity=\"0.15\"/><text x=\"{}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\">Shaded: middle half of cases</text>", L + 4.0, T + 2.0, L + 22.0, T + 11.0);
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\">0</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{} weeks</text></svg>",
                   H - 6.0, x(xmax), H - 6.0, offsets.last().unwrap() / 7);
    s
}

#[cfg(test)]
mod response_test {
    use super::*;

    #[test]
    fn response_draws_band_and_median() {
        let g = vec![("Drops".to_string(), BLUE, vec![vec![0.0, -0.05, -0.08], vec![0.0, -0.03, -0.06], vec![0.0, -0.04, -0.07]])];
        let svg = response(&g, &[0, 7, 14], "t", true);
        assert!(svg.contains("<polygon") && svg.contains("<polyline") && svg.contains("Drops") && svg.contains("2 weeks"));
    }
}

/// 구간 그림: 선 하나 + 기준선, 기준 이상으로 이어진 구간(spells)을 옅은 띠로 칠하고 띠 위에 길이(몇 년)를 쓴다.
/// "이런 고비가 과거엔 얼마나 갔나"를 한눈에(진행 중인 구간은 파랑 띠 + 'so far'). 짧은 구간(min_days 미만)은 칠하지 않는다.
pub fn spells(pts: &[Point], th: f64, min_days: i64, now: Option<(f64, &str)>, title: &str) -> String {
    let lo = pts.iter().map(|p| p.value).fold(f64::MAX, f64::min).min(th);
    let hi = pts.iter().map(|p| p.value).fold(f64::MIN, f64::max).max(th);
    let (lo, hi) = ((lo * 20.0).floor() / 20.0, (hi * 20.0).ceil() / 20.0);
    let top = T + 22.0; // 띠 위 글자 자리
    let (x0, x1) = (pts[0].day as f64, pts.last().unwrap().day as f64);
    let x = |d: f64| L + (d - x0) / (x1 - x0).max(1.0) * (W - L - R - 34.0);
    let y = |v: f64| top + (hi - v) / (hi - lo).max(1e-9) * (H - top - B);
    let mut s = head(title);
    // 띠 먼저(선 아래)
    let mut i = 0;
    while i < pts.len() {
        if pts[i].value < th { i += 1; continue; }
        let st = i;
        // 1년 안에 다시 넘으면 한 고비로 본다(1986~87년 반년 쉼에 글자 둘이 겹쳤다)
        loop {
            while i < pts.len() && pts[i].value >= th { i += 1; }
            let next = pts[i..].iter().position(|p| p.value >= th).map(|k| i + k);
            match next { Some(k) if pts[k].day - pts[i - 1].day <= 366 => i = k, _ => break }
        }
        let (a, b) = (&pts[st], &pts[i - 1]);
        let ongoing = i == pts.len();
        if b.day - a.day < min_days && !ongoing { continue; }
        let (xa, xb) = (x(a.day as f64), x(b.day as f64).max(x(a.day as f64) + 3.0));
        let fill = if ongoing { "#dbeafe" } else { SHADE };
        let _ = write!(s, "<rect x=\"{xa:.1}\" y=\"{top:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{fill}\"/>", xb - xa, H - top - B);
        let yrs = (b.day - a.day) as f64 / 365.25;
        let lab = if ongoing { format!("{yrs:.1} yrs so far") } else { format!("{yrs:.1} yrs") };
        let anchor = if ongoing { "end" } else { "middle" };
        let tx = if ongoing { xb.max(xa + 10.0) + 30.0 } else { (xa + xb) / 2.0 };
        let _ = write!(s, "<text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{}\" font-size=\"13\">{lab}</text>", top - 6.0, if ongoing { BLUE } else { INK });
    }
    // 눈금(5%p마다, 최대 6개)
    let mut v = lo;
    let step = if (hi - lo) / 0.05 > 6.0 { 0.1 } else { 0.05 };
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text>", W - R - 34.0, y(v), y(v), L - 6.0, y(v) + 4.0, pct(v));
        v += step;
    }
    let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R - 34.0, y(th), y(th));
    let d: Vec<String> = pts.iter().map(|p| format!("{:.1},{:.1}", x(p.day as f64), y(p.value))).collect();
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{INK}\" stroke-width=\"2\"/>", d.join(" "));
    // 기준선 값 글자(눈금이 기준을 건너뛰면 선의 뜻이 안 보였다)
    let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\" font-weight=\"600\">{}</text>", L - 6.0, y(th) + 4.0, pct(th));
    let l = pts.last().unwrap();
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{INK}\"/>", x(l.day as f64), y(l.value));
    // --now: 마지막 자료 뒤 오늘 추정(예: 지난 분기 집값에 이번 주 금리). 속 빈 파랑 점과 값, 마지막 값은 회색 글자로 아래에
    match now {
        Some((v, lab)) => {
            let xn = x(l.day as f64) + 10.0;
            let _ = write!(s, "<circle cx=\"{xn:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"#fff\" stroke=\"{BLUE}\" stroke-width=\"2.5\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{BLUE}\" font-weight=\"600\" font-size=\"14\">{}</text><title>{lab}</title>", y(v), xn + 7.0, y(v) + 5.0, pct(v));
            // 마지막 자료 값 글자는 쓰지 않는다: 두 값이 나란히 있으면 독자가 어느 것이 지금인지 묻는다(2026-11-10 콜드 리드 3명)
        }
        None => { let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{BLUE}\" font-weight=\"600\" font-size=\"14\">{}</text>", x(l.day as f64) + 7.0, y(l.value) + 5.0, pct(l.value)); }
    }
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\">{}</text><text x=\"{:.1}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text></svg>", H - 6.0, &pts[0].date[..4], W - R - 34.0, H - 6.0, &l.date[..4]);
    s
}

/// 시차 그림: 신호마다 한 줄, 0(신호 시작)에서 사건 시작까지 막대와 몇 달 글자. 사건이 아직 없는 줄은 파랑 'so far'.
/// "보통 얼마 뒤에 왔나"와 "지금은 그 범위를 넘었나"를 같은 가로축 위치로(B5).
/// mark: 사건까지 mark개월 이하였던 막대를 주황으로(예: 1년 안에 침체가 온 신호), axis: 눈금 아래 축 이름
pub fn lead(rows: &[(String, f64, bool)], zero: &str, mark: f64, axis: &str, title: &str) -> String {
    let row = 26.0;
    let legend = mark > 0.0 && rows.iter().any(|r| r.2 && r.1 <= mark);
    let h = T + rows.len() as f64 * row + B + 8.0 + if axis.is_empty() { 0.0 } else { 20.0 } + if legend { 22.0 } else { 0.0 };
    let max = rows.iter().map(|r| r.1).fold(12.0, f64::max);
    let step = if max > 72.0 { 24.0 } else if max > 36.0 { 12.0 } else { 6.0 };
    let xmax = (max / step).ceil() * step;
    // 줄 이름이 해(4자)보다 길면(예: '1984 · 11.6%') 왼쪽 여백을 늘린다
    let l = rows.iter().map(|r| r.0.chars().count()).max().map(|n| n as f64 * 7.6 + 14.0).unwrap_or(52.0).max(52.0);
    let r = 96.0;
    let x = |m: f64| l + m / xmax * (W - l - r);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let lg = if legend { 22.0 } else { 0.0 };
    let ax = if axis.is_empty() { 0.0 } else { 20.0 } + lg;
    if !axis.is_empty() {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>", (l + W - r) / 2.0, h - B + 16.0 - lg, axis.replace('&', "&amp;"));
    }
    let mut v = 0.0;
    while v <= xmax + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{T}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       x(v), x(v), h - B - ax, x(v), h - B - ax + 16.0, match v { 0.0 => "0".into(), _ if step < 12.0 => format!("{v} mo"), 12.0 => "1 yr".into(), _ => format!("{} yrs", v / 12.0) });
        v += step;
    }
    // 주황 막대의 뜻을 그림 안에(두 독자가 범례 없이 몰랐다, 2026-11-18): 맨 아래 축 이름 줄 왼쪽
    if legend {
        let _ = write!(s, "<rect x=\"{l}\" y=\"{:.1}\" width=\"12\" height=\"12\" rx=\"2\" fill=\"#ea580c\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Recession came within {} months</text>", h - 16.0, l + 18.0, h - 6.0, mark);
    }
    for (k, (year, m, done)) in rows.iter().enumerate() {
        let y = T + k as f64 * row + 6.0;
        let color = if !*done { BLUE } else if *m <= mark { "#ea580c" } else { GRAY };
        let lab = match (*done, m.round()) { (false, _) => format!("{:.0} mo so far", m), (true, z) if z == 0.0 => zero.to_string(), _ => format!("{:.0} mo", m) };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"14\">{year}</text><rect x=\"{l}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"12\" rx=\"2\" fill=\"{color}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"13\"{}>{lab}</text>",
                       l - 8.0, y + 11.0, (x(*m) - l).max(2.0), x(*m) + 6.0, y + 11.0, if *done { INK } else { BLUE }, if *done { "" } else { " font-weight=\"600\"" });
    }
    s.push_str("</svg>");
    s
}

/// 신호판: 신호마다 한 줄 트랙(0~축 끝), 경고선 눈금과 '지금' 점. 경고선을 넘으면 점이 주황.
/// 서로 다른 단위의 신호를 '경고선까지 얼마나 남았나'라는 같은 위치로 읽게(B5). 값은 엔진 결과를 넣는다.
pub fn board(rows: &[(String, f64, f64, f64, String)], title: &str) -> String {
    let row = 58.0;
    let h = T + rows.len() as f64 * row + 10.0;
    let (l, r) = (16.0, 16.0);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    for (k, (name, now, trig, max, lab)) in rows.iter().enumerate() {
        let y = T + k as f64 * row + 18.0;
        let x = |v: f64| l + (v / max).clamp(0.0, 1.0) * (W - l - r);
        let over = now >= trig;
        let c = if over { "#ea580c" } else { BLUE };
        let _ = write!(s, "<text x=\"{l}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{}</text>", y, name.replace('&', "&amp;"));
        let _ = write!(s, "<rect x=\"{l}\" y=\"{:.1}\" width=\"{:.1}\" height=\"6\" rx=\"3\" fill=\"{SHADE}\"/><rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"6\" fill=\"#fed7aa\"/>",
                       y + 12.0, W - l - r, x(*trig), y + 12.0, W - r - x(*trig));
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#ea580c\" stroke-width=\"2\"/>", x(*trig), x(*trig), y + 6.0, y + 24.0);
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"7\" fill=\"{c}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(*now), y + 15.0);
        let anchor = if x(*now) > W / 2.0 { "end" } else { "start" };
        let tx = if anchor == "end" { x(*now) - 10.0 } else { x(*now) + 10.0 };
        let _ = write!(s, "<text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{c}\" font-size=\"13\" font-weight=\"600\">{}</text>", y + 38.0, lab.replace('&', "&amp;"));
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod lead_test {
    #[test]
    fn lead_marks_short_leads_orange() {
        let svg = super::lead(&[("1990".into(), 8.0, true), ("2000".into(), 30.0, true)], "0 mo", 12.0, "Months", "t");
        assert_eq!(svg.matches("#ea580c").count(), 2); // 막대 하나 + 범례
        assert!(svg.contains(">Months<") && svg.contains("Recession came within 12 months"));
    }
}

#[cfg(test)]
mod board_test {
    #[test]
    fn board_marks_over_trigger_orange() {
        let svg = super::board(&[("A".into(), 0.6, 0.5, 1.0, "over".into()), ("B".into(), 0.1, 0.5, 1.0, "calm".into())], "t");
        assert!(svg.contains("#ea580c\" stroke=\"#fff") && svg.contains("calm"));
    }
}

/// 예상 vs 실제 그림: 실제(진한 선)와 회귀 예상(회색 점선), 그 사이를 옅게 칠하고 끝에 두 값. 학습 기간 끝을 세로 점선으로.
/// '조건으로 보면 이쯤이어야 하는데 실제는 여기'를 같은 축 위치로(B5).
pub fn fit(rows: &[(String, f64, f64)], train_end: &str, labels: (&str, &str), title: &str) -> String {
    let lo = rows.iter().flat_map(|r| [r.1, r.2]).fold(f64::MAX, f64::min);
    let hi = rows.iter().flat_map(|r| [r.1, r.2]).fold(f64::MIN, f64::max);
    let (lo, hi) = ((lo / 10.0).floor() * 10.0, (hi / 10.0).ceil() * 10.0);
    let right = 92.0; // 끝 이름표 자리
    let n = rows.len() as f64;
    let x = |i: usize| L + i as f64 / (n - 1.0).max(1.0) * (W - L - R - right);
    let y = |v: f64| T + (hi - v) / (hi - lo).max(1e-9) * (H - T - B);
    let mut s = head(title);
    let mut v = lo;
    let step = if (hi - lo) / 20.0 > 6.0 { 40.0 } else { 20.0 };
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{v}</text>", W - R - right, y(v), y(v), L - 6.0, y(v) + 4.0);
        v += step;
    }
    // 차이 띠: 실제 → 예상 다각형
    let up: Vec<String> = rows.iter().enumerate().map(|(i, r)| format!("{:.1},{:.1}", x(i), y(r.1))).collect();
    let dn: Vec<String> = rows.iter().enumerate().rev().map(|(i, r)| format!("{:.1},{:.1}", x(i), y(r.2))).collect();
    let _ = write!(s, "<polygon points=\"{} {}\" fill=\"#fed7aa\" opacity=\"0.55\"/>", up.join(" "), dn.join(" "));
    if let Some(k) = rows.iter().position(|r| r.0.as_str() > train_end) {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{T}\" y2=\"{:.1}\" stroke=\"{GRAY}\" stroke-dasharray=\"2 3\"/>", x(k), x(k), H - B);
    }
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{GRAY}\" stroke-width=\"2\" stroke-dasharray=\"5 3\"/>", dn.iter().rev().cloned().collect::<Vec<_>>().join(" "));
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{INK}\" stroke-width=\"2\"/>", up.join(" "));
    let l = rows.last().unwrap();
    let xe = x(rows.len() - 1) + 7.0;
    let (ya, ye) = (y(l.1), y(l.2));
    let (ya, ye) = if (ya - ye).abs() < 34.0 { if ya > ye { (ye + 34.0, ye) } else { (ya, ya + 34.0) } } else { (ya, ye) };
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{BLUE}\"/><text x=\"{xe:.1}\" y=\"{:.1}\" fill=\"{BLUE}\" font-size=\"13\" font-weight=\"600\">{}</text><text x=\"{xe:.1}\" y=\"{:.1}\" fill=\"{BLUE}\" font-size=\"14\" font-weight=\"600\">{:.0}</text>", xe - 7.0, y(l.1), ya - 2.0, labels.0, ya + 14.0, l.1);
    let _ = write!(s, "<text x=\"{xe:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">{}</text><text x=\"{xe:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">{:.0}</text>", ye - 2.0, labels.1, ye + 14.0, l.2);
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\">{}</text><text x=\"{:.1}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text></svg>", H - 6.0, &rows[0].0[..4], W - R - right, H - 6.0, &l.0[..4]);
    s
}

/// 해마다 막대(0 위·아래): 해마다 평균 수정치처럼 부호가 뜻을 갖는 값. 아래(음수)는 주황, 위는 회색, 경기침체가 있던 해는 음영,
/// 가장 낮은 해 marks개에 '연도 값' 이름표(Cleveland & McGill: 같은 축 위치, 강조는 몇 개만).
pub fn years(rows: &[(i32, f64, bool)], suffix: &str, marks: usize, title: &str) -> String {
    const ORANGE: &str = "#eb6834";
    let lo = rows.iter().map(|r| r.1).fold(0.0, f64::min);
    let hi = rows.iter().map(|r| r.1).fold(0.0, f64::max);
    let step = [10.0, 20.0, 25.0, 50.0, 100.0].into_iter().find(|st| (hi - lo) / st <= 6.0).unwrap_or(200.0);
    let (lo, hi) = ((lo / step).floor() * step, (hi / step).ceil() * step);
    let n = rows.len() as f64;
    let bw = (W - L - R) / n;
    let x = |i: usize| L + i as f64 * bw;
    let y = |v: f64| T + 16.0 + (hi - v) / (hi - lo).max(1e-9) * (H - T - B - 16.0);
    let mut s = head(title);
    for (i, r) in rows.iter().enumerate().filter(|(_, r)| r.2) {
        let _ = write!(s, "<rect x=\"{:.1}\" y=\"{}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{SHADE}\" opacity=\"0.7\"><title>{} recession</title></rect>", x(i), T + 16.0, bw, H - T - B - 16.0, r.0);
    }
    let mut v = lo;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}{v}{suffix}</text>",
                       W - R, y(v), y(v), if v == 0.0 { INK } else { "#eef2f7" }, L - 6.0, y(v) + 4.0, if v > 0.0 { "+" } else { "" });
        v += step;
    }
    for (i, r) in rows.iter().enumerate() {
        let (top, h) = if r.1 >= 0.0 { (y(r.1), y(0.0) - y(r.1)) } else { (y(0.0), y(r.1) - y(0.0)) };
        let _ = write!(s, "<rect x=\"{:.1}\" y=\"{top:.1}\" width=\"{:.1}\" height=\"{h:.1}\" fill=\"{}\"><title>{}: {:.0}{suffix}</title></rect>", x(i) + 1.0, (bw - 2.0).max(1.0), if r.1 < 0.0 { ORANGE } else { GRAY }, r.0, r.1);
    }
    let mut low: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].1 < 0.0).collect();
    low.sort_by(|&a, &b| rows[a].1.partial_cmp(&rows[b].1).unwrap());
    let mut picked: Vec<usize> = low.into_iter().take(marks).collect();
    picked.sort();
    let mut prev: Option<(f64, f64)> = None; // 앞 이름표 (x, y): 가까우면 한 줄 아래로
    for &i in &picked {
        let r = &rows[i];
        let mut ly = y(r.1) + 15.0;
        if let Some((px, py)) = prev { if x(i) - px < 80.0 && (ly - py).abs() < 15.0 { ly = py + 16.0; } }
        prev = Some((x(i), ly));
        let anchor = if x(i) > W - 70.0 { "end" } else if x(i) < L + 40.0 { "start" } else { "middle" };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{INK}\" font-size=\"13\" font-weight=\"600\">{} {:.0}{suffix}</text>", x(i) + bw / 2.0, ly, r.0, r.1);
    }
    let (first, last) = (rows[0].0, rows.last().unwrap().0);
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\">{first}</text><text x=\"{:.1}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{last}</text></svg>", H - 6.0, W - R, H - 6.0);
    s
}

#[cfg(test)]
mod years_tests {
    #[test]
    fn years_colors_negative_and_marks_lowest() {
        let rows = vec![(2007, 5.0, false), (2008, -73.0, true), (2009, 12.0, true), (2025, -58.0, false)];
        let s = super::years(&rows, "K", 2, "t");
        assert_eq!(s.matches("#eb6834").count(), 2);
        assert!(s.contains("2008 -73K") && s.contains("2025 -58K"));
        assert_eq!(s.matches("recession</title>").count(), 2);
    }
}

#[cfg(test)]
mod fit_tests {
    #[test]
    fn fit_draws_both_lines_and_end_values() {
        let rows: Vec<(String, f64, f64)> = (0..24).map(|i| (format!("{}-{:02}", 2000 + i / 12, i % 12 + 1), 90.0 - i as f64, 92.0)).collect();
        let s = super::fit(&rows, "2000-12", ("Actual", "Expected"), "t");
        assert_eq!(s.matches("<polyline").count(), 2);
        assert!(s.contains(">67<") && s.contains(">92<") && s.contains("Expected"));
    }
}

/// 쌓은 막대: 시점마다 한 줄, 바탕(파랑) + 나머지(주황) = 전체. 예: 모기지 금리 = 10년 국채 금리 + 금리 차.
/// 두 몫이 같은 0에서 이어져 길이로 비교된다(Cleveland & McGill 1984). typical: 바탕 + 보통 차이 자리에 점선 눈금(그 차이였다면 전체가 어디였나).
/// rows: (이름, 바탕, 나머지). 값은 %p 단위 그대로 글자로(소수 둘째 자리).
pub fn stack(rows: &[(String, f64, f64)], typical: Option<f64>, names: [&str; 2], title: &str) -> String {
    let (rh, top) = (46.0, 40.0);
    let h = top + rh * rows.len() as f64 + 40.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let lw = 92.0; // 줄 이름 칸
    let max = rows.iter().map(|r| r.1 + r.2).fold(0.0, f64::max).max(rows.iter().map(|r| r.1 + typical.unwrap_or(0.0)).fold(0.0, f64::max));
    let x = |v: f64| lw + v / max * (W - lw - 64.0);
    let orange = "#eb6834";
    // 범례(색만으로 뜻을 말하지 않게 글자로)
    let _ = write!(s, "<rect x=\"{lw}\" y=\"8\" width=\"12\" height=\"12\" rx=\"2\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"19\" fill=\"{INK}\" font-size=\"14\">{}</text>", lw + 17.0, names[0].replace('&', "&amp;"));
    let lx = lw + 30.0 + names[0].len() as f64 * 7.6;
    let _ = write!(s, "<rect x=\"{lx:.1}\" y=\"8\" width=\"12\" height=\"12\" rx=\"2\" fill=\"{orange}\"/><text x=\"{:.1}\" y=\"19\" fill=\"{INK}\" font-size=\"14\">{}</text>", lx + 17.0, names[1].replace('&', "&amp;"));
    for (k, (name, base, gap)) in rows.iter().enumerate() {
        let y0 = top + k as f64 * rh;
        let last = k + 1 == rows.len();
        let _ = write!(s, "<text x=\"0\" y=\"{:.1}\" fill=\"{}\" font-size=\"14\"{}>{}</text>", y0 + 21.0, if last { "#0f172a" } else { INK }, if last { " font-weight=\"700\"" } else { "" }, name.replace('&', "&amp;"));
        let (xa, xb, xc) = (x(0.0), x(*base), x(base + gap));
        let _ = write!(s, "<rect x=\"{xa:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"24\" fill=\"{BLUE}\"/>", y0 + 4.0, (xb - xa - 1.0).max(0.0));
        let _ = write!(s, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"24\" rx=\"3\" fill=\"{orange}\"/>", xb + 1.0, y0 + 4.0, (xc - xb - 1.0).max(0.0));
        // 몫 글자: 칸이 넓으면 안에 흰 글자, 좁으면 생략(전체 글자가 남는다)
        if xb - xa > 34.0 { let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#fff\" font-size=\"13\">{base:.2}</text>", xa + 4.0, y0 + 21.0); } // 0.97처럼 짧은 막대도 값이 보이게(2026-11-18)
        if xc - xb > 40.0 { let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#fff\" font-size=\"13\">+{gap:.2}</text>", xb + 6.0, y0 + 21.0); }
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#0f172a\" font-size=\"15\" font-weight=\"700\">{:.2}%</text>", xc + 6.0, y0 + 21.0, base + gap);
        if let Some(t) = typical.filter(|_| last) {
            let xt = x(base + t);
            let _ = write!(s, "<line x1=\"{xt:.1}\" x2=\"{xt:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#0f172a\" stroke-width=\"2\" stroke-dasharray=\"3 3\"/>", y0 + 1.0, y0 + 33.0);
            let _ = write!(s, "<text x=\"{xt:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"#0f172a\" font-size=\"13\">{:.2}% with the usual gap</text>", y0 + 50.0, base + t);
        }
    }
    s.push_str("</svg>");
    s
}

/// 격자: 줄 = 금리, 칸 = 집값 변화. 칸마다 월 상환액이 월 소득에서 차지하는 몫(%), line을 넘으면 주황, 아래면 파랑(진하기는 line에서 먼 정도).
/// 지금 칸(now_row, 0% 변화)은 굵은 테두리. 독자가 "무엇이 얼마나 움직여야 하나"를 한 장에서 찾게(세 길: 금리·집값·소득).
pub fn grid(rates: &[f64], changes: &[f64], cells: &[Vec<f64>], line: f64, now: Option<(usize, usize)>, title: &str) -> String {
    let (lw, top, cw, ch) = (64.0, 46.0, ((W - 64.0 - 8.0) / changes.len() as f64).min(84.0), 34.0);
    let h = top + ch * rates.len() as f64 + 34.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let _ = write!(s, "<text x=\"{lw}\" y=\"16\" fill=\"{INK}\" font-size=\"13\">Home price vs. today →</text><text x=\"0\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Rate</text>", top - 8.0);
    for (j, c) in changes.iter().enumerate() {
        let lab = if c.abs() < 1e-9 { "Today".to_string() } else { format!("{}{}", if *c > 0.0 { "+" } else { "" }, pct(*c)) };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{lab}</text>", lw + cw * (j as f64 + 0.5), top - 8.0);
    }
    for (i, r) in rates.iter().enumerate() {
        let y0 = top + i as f64 * ch;
        let _ = write!(s, "<text x=\"0\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{r:.2}%</text>", y0 + 22.0);
        for (j, v) in cells[i].iter().enumerate() {
            let x0 = lw + j as f64 * cw;
            let d = ((v - line).abs() / 0.12).min(1.0);
            let fill = if *v > line { format!("rgba(235,104,52,{:.2})", 0.15 + 0.6 * d) } else { format!("rgba(37,99,235,{:.2})", 0.12 + 0.55 * d) };
            let ink = if d > 0.6 { "#fff" } else { "#0f172a" };
            let _ = write!(s, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{fill}\"/>", x0 + 1.0, y0 + 1.0, cw - 2.0, ch - 2.0);
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{ink}\" font-size=\"14\">{}</text>", x0 + cw / 2.0, y0 + 22.0, pct(*v));
            if now == Some((i, j)) {
                let _ = write!(s, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"none\" stroke=\"#0f172a\" stroke-width=\"2.5\"/>", x0 + 1.5, y0 + 1.5, cw - 3.0, ch - 3.0);
            }
        }
    }
    let _ = write!(s, "<text x=\"{lw}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Orange: payment above {} of income. Blue: below.</text></svg>", h - 10.0, pct(line));
    s
}

/// 절약 곡선: 가로 = 금리가 얼마나 내리나(%p), 세로 = 같은 대출의 월 상환액이 얼마나 줄어드나($).
/// 선 위에 이름 붙은 점(예: 차이가 보통으로, 0.5%p, 1%p)을 찍어 "어느 쪽이 더 크나"를 위치로 견준다(Cleveland & McGill 1984).
/// 첫 점(마크의 첫 번째)은 파랑으로 강조. pts: (하락 %p, 월 절약 $), marks: (이름, 하락 %p, 월 절약 $)
pub fn savings(pts: &[(f64, f64)], marks: &[(String, f64, f64)], now_rate: f64, title: &str) -> String {
    let (x0, x1) = (0.0, pts.last().map_or(1.0, |p| p.0));
    let ymax = pts.iter().map(|p| p.1).fold(0.0, f64::max);
    let step = if ymax > 300.0 { 100.0 } else { 50.0 };
    let ytop = (ymax / step).ceil() * step;
    let (l, r, t, b) = (54.0, 20.0, 18.0, 46.0);
    let x = |v: f64| l + (v - x0) / (x1 - x0) * (W - l - r);
    let y = |v: f64| t + (ytop - v) / ytop * (H - t - b);
    let mut s = head(title);
    let mut v = 0.0;
    while v <= ytop + 1e-9 {
        let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">${}</text>", W - r, y(v), y(v), l - 6.0, y(v) + 4.0, v);
        v += step;
    }
    // 가로 눈금: 하락폭과 그때의 금리
    // 눈금은 정수 금리 자리에(7.11, 6.61 같은 눈금은 읽기 어려웠다, 2026-11-18). 하락폭이 1%p 미만이면 예전처럼 0.5%p 간격
    if x1 >= 1.0 {
        let mut r0 = now_rate.floor();
        while now_rate - r0 <= x1 + 1e-9 {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{r0:.0}%</text>", x(now_rate - r0), H - b + 18.0);
            r0 -= 1.0;
        }
    } else {
        let mut d = 0.0;
        while d <= x1 + 1e-9 {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{:.2}%</text>", x(d), H - b + 18.0, now_rate - d);
            d += 0.5;
        }
    }
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">Mortgage rate (now {now_rate:.2}%) →</text>", (l + W - r) / 2.0, H - 6.0);
    let line: Vec<String> = pts.iter().map(|p| format!("{:.1},{:.1}", x(p.0), y(p.1))).collect();
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{GRAY}\" stroke-width=\"2\"/>", line.join(" "));
    for (k, (name, dx, sv)) in marks.iter().enumerate() {
        let (cx, cy) = (x(*dx), y(*sv));
        let c = if k == 0 { BLUE } else { INK };
        let _ = write!(s, "<circle cx=\"{cx:.1}\" cy=\"{cy:.1}\" r=\"6\" fill=\"{c}\"/>");
        // 강조 점은 선 위 왼쪽, 나머지는 선 아래 오른쪽: 점들이 가까워도 글자가 겹치지 않게(2026-11-10 휴대폰 확인)
        let (anchor, tx, ty) = if k == 0 { ("end", cx - 12.0, cy - 30.0) } else if cx > W - 130.0 { ("end", cx - 14.0, cy + 2.0) } else { ("start", cx + 12.0, cy + 22.0) }; // 오른쪽 끝 이름표가 잘렸다
        let _ = write!(s, "<text x=\"{tx:.1}\" y=\"{ty:.1}\" text-anchor=\"{anchor}\" fill=\"{c}\" font-size=\"15\" font-weight=\"700\">${:.0} a month</text><text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{c}\" font-size=\"13\">{}</text>",
                       sv, ty + 16.0, name.replace('&', "&amp;"));
    }
    s.push_str("</svg>");
    s
}

/// 옮겨 가는 점(shift): 줄마다 처음 값(빈 점) → 고친 값(채운 점), 뒤에 과거 오차로 본 범위(옅은 띠), 기준선 하나(점선).
/// 두 추정이 고치면 한곳으로 모이는지(또는 갈라지는지)를 한 축 위에서 본다. rows: (이름, 처음, 고친 값, 범위 아래, 위), 단위 %
pub fn shift(rows: &[(String, f64, f64, f64, f64)], refv: f64, ref_label: &str, show_range: bool, title: &str) -> String {
    // 범위 띠는 선택(--range): 두 회차 독자 넷이 띠를 '못 믿을 값'으로 읽었다(2026-11-11)
    let lo = rows.iter().map(|r| if show_range { r.3.min(r.1) } else { r.1.min(r.2) }).fold(refv, f64::min);
    let hi = rows.iter().map(|r| if show_range { r.4.max(r.1) } else { r.1.max(r.2) }).fold(refv, f64::max);
    let (x0, x1) = ((lo * 2.0).floor() / 2.0, (hi * 2.0).ceil() / 2.0);
    let (l, r) = (20.0, 20.0);
    let x = |v: f64| l + (v - x0) / (x1 - x0).max(1e-9) * (W - l - r);
    let top = 62.0;
    let gap = (H - top - 40.0) / rows.len().max(1) as f64;
    // 기준 점선이 글자를 가로지르지 않게 글자 뒤에 흰 테두리(2026-11-11 휴대폰 확인)
    const HALO: &str = "stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\"";
    let mut s = head(title);
    let _ = write!(s, "<defs><marker id=\"shift-arrow\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"6\" markerHeight=\"6\" orient=\"auto\"><path d=\"M0,0L10,5L0,10z\" fill=\"{BLUE}\"/></marker></defs>");
    // 범례: 빈 점·화살표·채운 점·띠가 무엇인지 그림 안에서(2026-11-11 콜드 리드 두 독자: 범례가 없어 기호를 추측했다)
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"14\" r=\"5\" fill=\"#fff\" stroke=\"{GRAY}\" stroke-width=\"2\"/><text x=\"{:.1}\" y=\"19\" fill=\"{INK}\" font-size=\"14\">first fill</text><circle cx=\"{:.1}\" cy=\"14\" r=\"6\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"19\" fill=\"{INK}\" font-size=\"14\">after its usual miss</text>",
                   l + 6.0, l + 16.0, l + 98.0, l + 109.0);
    if show_range {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"14\" y2=\"14\" stroke=\"{SHADE}\" stroke-width=\"10\" stroke-linecap=\"round\"/><text x=\"{:.1}\" y=\"19\" fill=\"{INK}\" font-size=\"14\">past misses</text>", l + 280.0, l + 300.0, l + 310.0);
    }
    let mut v = x0;
    while v <= x1 + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{top}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{v:.1}%</text>", x(v), x(v), H - 34.0, x(v), H - 16.0);
        v += 0.5;
    }
    let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 4\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{} {refv:.2}%</text>",
                   x(refv), x(refv), top - 8.0, H - 34.0, x(refv), top - 12.0, ref_label.replace('&', "&amp;"));
    // 값 글자는 기준선 한쪽에만: 기준선보다 크면 오른쪽으로, 작으면 왼쪽으로 펼친다(가운데 정렬은 선을 가로질렀다, 2026-11-11 휴대폰 확인)
    let side = |v: f64, at: f64| if v >= refv { ("start", (at - 6.0).max(x(refv) + 5.0)) } else { ("end", (at + 6.0).min(x(refv) - 5.0)) };
    for (i, (name, raw, fixed, rlo, rhi)) in rows.iter().enumerate() {
        let y = top + gap * (i as f64 + 0.62);
        let _ = write!(s, "<text x=\"{l}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\" {HALO}>{}</text>", y - 30.0, name.replace('&', "&amp;"));
        if show_range { let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{y:.1}\" y2=\"{y:.1}\" stroke=\"{SHADE}\" stroke-width=\"10\" stroke-linecap=\"round\"><title>past range {rlo:.2}% to {rhi:.2}%</title></line>", x(*rlo), x(*rhi)); }
        let dir = if fixed >= raw { 1.0 } else { -1.0 };
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{y:.1}\" y2=\"{y:.1}\" stroke=\"{BLUE}\" stroke-width=\"2\" marker-end=\"url(#shift-arrow)\"/>", x(*raw) + dir * 7.0, x(*fixed) - dir * 8.0);
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"6\" fill=\"#fff\" stroke=\"{GRAY}\" stroke-width=\"2\"><title>as filled {raw:.2}%</title></circle>", x(*raw));
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"7\" fill=\"{BLUE}\"><title>corrected {fixed:.2}%</title></circle>", x(*fixed));
        // 고친 값은 점 위, 처음 값은 점 아래: 0.1%p(약 20px) 떨어져도 글자가 겹치지 않게
        let ((fa, fx), (ra, rx)) = (side(*fixed, x(*fixed)), side(*raw, x(*raw)));
        let _ = write!(s, "<text x=\"{fx:.1}\" y=\"{:.1}\" text-anchor=\"{fa}\" fill=\"{BLUE}\" font-size=\"15\" font-weight=\"700\" {HALO}>{fixed:.1}%</text><text x=\"{rx:.1}\" y=\"{:.1}\" text-anchor=\"{ra}\" fill=\"{INK}\" font-size=\"14\" {HALO}>{raw:.2}% first fill</text>",
                       y - 13.0, y + 24.0);
    }
    s.push_str("</svg>");
    s
}

/// 계절 칸(season): 품목마다 한 줄, 기준 달 뒤 몇 달을 칸으로. 칸 값은 기준 달 대비 변화(추세를 걷은 중앙값, %).
/// 내린 칸은 파랑이 진할수록 많이 내린 것, 오른 칸은 옅은 회색. 줄마다 가장 싼 달에 테두리. rows: (이름, [값…]), cols: 칸 이름
pub fn season(rows: &[(String, Vec<f64>)], cols: &[&str], note: &str, title: &str) -> String {
    // note: 칸 값이 무엇인지 그림 위 한 줄(2026-11-11 콜드 리드 두 독자: '10월 대비'인 줄 그림만 봐서는 몰랐다)
    let (left, top, rr) = (118.0, if note.is_empty() { 40.0 } else { 66.0 }, 12.0);
    let cw = (W - left - rr) / cols.len().max(1) as f64;
    let ch = 34.0;
    let h = top + ch * rows.len() as f64 + 28.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let lo = rows.iter().flat_map(|r| r.1.iter().copied()).fold(0.0, f64::min);
    if !note.is_empty() {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"20\" fill=\"{INK}\" font-size=\"14\">{}</text>", 8.0, note.replace('&', "&amp;"));
    }
    for (j, c) in cols.iter().enumerate() {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">{}</text>", left + cw * (j as f64 + 0.5), top - 12.0, c);
    }
    for (i, (name, vals)) in rows.iter().enumerate() {
        let y = top + ch * i as f64;
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"14\">{}</text>", left - 8.0, y + ch / 2.0 + 5.0, name.replace('&', "&amp;"));
        // 가장 싼 달: 최솟값과 0.05%p 안쪽이면 같이 표시(반올림하면 같은 두 칸 중 하나만 테두리면 헷갈린다, 2026-11-12 콜드 리드)
        let min = vals.iter().copied().fold(f64::MAX, f64::min);
        let is_best = |v: f64| v < 0.0 && v - min < 0.05;
        for (j, v) in vals.iter().enumerate() {
            let x = left + cw * j as f64;
            // 내린 정도를 같은 파랑의 진하기로(한 색상 단계, 0이 흰색에 가깝게)
            let a = if *v < 0.0 { 0.12 + 0.78 * (v / lo.min(-1e-9)).min(1.0) } else { 0.0 };
            let fill = if *v < 0.0 { format!("fill=\"{BLUE}\" fill-opacity=\"{a:.2}\"") } else { "fill=\"#f1f5f9\"".to_string() };
            let ink = if a > 0.5 { "#fff" } else { INK };
            let _ = write!(s, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"4\" {fill} stroke=\"#fff\" stroke-width=\"2\"><title>{} {}: {:+.1}%</title></rect>", x, y, cw, ch, name, cols[j], v);
            if is_best(*v) {
                let _ = write!(s, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"4\" fill=\"none\" stroke=\"#0f172a\" stroke-width=\"2\"/>", x + 2.0, y + 2.0, cw - 4.0, ch - 4.0);
            }
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{ink}\" font-size=\"14\" font-weight=\"{}\">{}{:.1}%</text>", x + cw / 2.0, y + ch / 2.0 + 5.0, if is_best(*v) { 700 } else { 400 }, if *v > 0.0 { "+" } else { "" }, v);
        }
    }
    s.push_str("</svg>");
    s
}

/// 손익분기(breakeven): 펀드마다 '금리가 그대로였을 때보다 몇 달러 앞서거나 뒤처지나'(가로 년). 0선을 지나는 해에 점과 이름표.
/// rows: (이름, 손익분기 년, [(년, 차이 달러)])
pub fn breakeven(rows: &[(String, Option<f64>, Vec<(f64, f64)>)], note: &str, title: &str) -> String {
    // note: 시작 금액·무엇과 견주는지 그림 위 한 줄(2026-11-13 콜드 리드: 세로축 달러의 기준이 안 보였다)
    // 범례: 펀드마다 한 줄(색 점 + 이름 + 손익분기 해). 점 옆 긴 이름표는 서로 겹쳤다(2026-11-13)
    let legend = rows.len() as f64 * 20.0;
    let (l, r, t, b) = (58.0, 20.0, if note.is_empty() { 24.0 } else { 44.0 } + legend, 40.0);
    let x1 = rows.iter().flat_map(|r| r.2.last().map(|p| p.0)).fold(1.0, f64::max);
    let lo = rows.iter().flat_map(|r| r.2.iter().map(|p| p.1)).fold(0.0, f64::min);
    let hi = rows.iter().flat_map(|r| r.2.iter().map(|p| p.1)).fold(0.0, f64::max);
    let step = [100.0, 200.0, 250.0, 500.0, 1000.0].into_iter().find(|s| (hi - lo) / s <= 6.0).unwrap_or(2000.0);
    let (y0, y1) = ((lo / step).floor() * step, (hi / step).ceil() * step);
    let x = |v: f64| l + v / x1 * (W - l - r);
    let y = |v: f64| t + (y1 - v) / (y1 - y0).max(1e-9) * (H - t - b);
    let mut s = head(title);
    if !note.is_empty() {
        let _ = write!(s, "<text x=\"8\" y=\"20\" fill=\"{INK}\" font-size=\"14\">{}</text>", note.replace('&', "&amp;"));
    }
    let shades_l = ["#93c5fd", BLUE, "#1e3a8a"];
    for (i, (name, be, _)) in rows.iter().enumerate() {
        let yy = if note.is_empty() { 18.0 } else { 40.0 } + 20.0 * i as f64;
        let c = shades_l[i.min(2)];
        let tail = be.map(|v| format!(": even in {v:.1} years")).unwrap_or_default();
        let _ = write!(s, "<circle cx=\"14\" cy=\"{:.1}\" r=\"5\" fill=\"{c}\"/><text x=\"24\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\"><tspan fill=\"{c}\" font-weight=\"700\">{}</tspan>{tail}</text>", yy - 4.0, yy + 1.0, name.replace('&', "&amp;"));
    }
    let mut v = y0;
    while v <= y1 + 1e-9 {
        let lab = if v > 0.0 { format!("+${v:.0}") } else if v < 0.0 { format!("-${:.0}", -v) } else { "$0".into() };
        let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{lab}</text>", W - r, y(v), y(v), if v.abs() < 1e-9 { INK } else { "#eef2f7" }, l - 6.0, y(v) + 4.0);
        v += step;
    }
    let tick = if x1 > 12.0 { 5.0 } else { 2.0 };
    let mut yr = 0.0;
    while yr <= x1 + 1e-9 {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>", x(yr), H - b + 18.0, if yr == 0.0 { "now".to_string() } else { format!("{yr:.0}") });
        yr += tick;
    }
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">years from now</text>", (l + W - r) / 2.0, H - 6.0);
    let shades = ["#93c5fd", BLUE, "#1e3a8a"];
    for (i, (name, be, pts)) in rows.iter().enumerate() {
        let c = shades[i.min(2)];
        let line: Vec<String> = pts.iter().map(|p| format!("{:.1},{:.1}", x(p.0), y(p.1))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{c}\" stroke-width=\"2.5\"><title>{}</title></polyline>", line.join(" "), name.replace('&', "&amp;"));
        if let Some(bx) = be.filter(|v| *v <= x1) {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{c}\"/>", x(bx), y(0.0));
        }
    }
    s.push_str("</svg>");
    s
}

/// 따라잡기 그림(2026-11-15, 채권 펀드 재작성): 하락마다 한 줄, 같은 가로축(년)에 '금리 그대로였을 경로'를 다시 넘기까지 걸린 해(막대)와
/// 그 시작 금리의 듀레이션(손익분기 규칙이 말하는 해, 주황 눈금). 아직 못 넘은 하락은 파랑 막대 '지금까지'. 규칙이 실제보다 느렸나를 위치로(B5)
pub fn catchup(rows: &[(String, f64, f64, bool)], title: &str) -> String {
    let row = 26.0;
    let leg = 42.0;
    let h = T + leg + rows.len() as f64 * row + B + 8.0;
    let max = rows.iter().map(|r| r.1.max(r.2)).fold(1.0, f64::max);
    let xmax = (max / 2.0).ceil() * 2.0;
    let l = rows.iter().map(|r| r.0.chars().count()).max().map(|n| n as f64 * 7.6 + 14.0).unwrap_or(52.0).max(52.0);
    let r = 70.0;
    let x = |v: f64| l + v / xmax * (W - l - r);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let _ = write!(s, "<rect x=\"{l}\" y=\"{:.1}\" width=\"18\" height=\"10\" rx=\"2\" fill=\"{GRAY}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Years to catch up</text><line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#ea580c\" stroke-width=\"3\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Duration rule</text>",
                   T + 2.0, l + 24.0, T + 11.0, l + 160.0, l + 160.0, T, T + 13.0, l + 168.0, T + 11.0);
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Year of the high · 7-year yield then</text>", 2.0, T + 31.0);
    let top = T + leg;
    let mut v = 0.0;
    while v <= xmax + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{top}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       x(v), x(v), h - B, x(v), h - B + 16.0, if v == 0.0 { "0".to_string() } else { format!("{v} yrs") });
        v += 2.0;
    }
    for (k, (name, yrs, dur, done)) in rows.iter().enumerate() {
        let y = top + k as f64 * row + 6.0;
        let (c, lab) = if *done { (GRAY, format!("{yrs:.1} yrs")) } else { (BLUE, format!("{yrs:.1} so far")) };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"14\">{}</text><rect x=\"{l}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"12\" rx=\"2\" fill=\"{c}\"/><line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#ea580c\" stroke-width=\"3\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"13\"{}>{lab}</text>",
                       l - 8.0, y + 11.0, name.replace('&', "&amp;"), (x(*yrs) - l).max(2.0), x(*dur), x(*dur), y - 3.0, y + 15.0,
                       if (dur - yrs).abs() < 0.9 || yrs > dur { x(yrs.max(*dur)) + 6.0 } else { x(*yrs) + 6.0 }, y + 11.0, if *done { INK } else { BLUE }, if *done { "" } else { " font-weight=\"600\"" });
    }
    s.push_str("</svg>");
    s
}

/// 폭 그림: 줄마다 n개의 동그라미(지표 하나에 하나). 침체마다 시작 달에 이미 꺾인 수(진한 회색)와 3달 뒤까지 꺾인 수(연한 주황),
/// 맨 위 '지금' 줄은 파랑. "지난 침체는 시작할 때 몇 개가 꺾여 있었나"와 지금을 같은 칸 수로 센다(그림문자, Haroz 외 2015)
pub fn breadth(rows: &[(String, usize, Option<usize>, bool)], n: usize, names: [&str; 2], title: &str) -> String {
    const LIGHT: &str = "#f6b89c";
    let (rh, top, left) = (34.0, 48.0, 92.0);
    let h = top + rh * rows.len() as f64 + 8.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let mut lx = left;
    for (c, nm) in [INK, LIGHT].iter().zip(names) {
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"18\" r=\"7\" fill=\"{c}\"/><text x=\"{:.1}\" y=\"23\" fill=\"{INK}\" font-size=\"14\">{}</text>", lx + 7.0, lx + 19.0, nm.replace('&', "&amp;"));
        lx += 19.0 + nm.chars().count() as f64 * 7.6 + 18.0;
    }
    let gap = ((W - left - R) / n.max(1) as f64).min(40.0);
    for (k, (lab, now, later, is_now)) in rows.iter().enumerate() {
        let y = top + rh * k as f64 + rh / 2.0;
        let ink = if *is_now { BLUE } else { INK };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{ink}\" font-size=\"14\"{}>{}</text>", left - 14.0, y + 5.0, if *is_now { " font-weight=\"600\"" } else { "" }, lab.replace('&', "&amp;"));
        for i in 0..n {
            let fill = if i < *now { if *is_now { BLUE } else { INK } } else if later.map_or(false, |l| i < l) { LIGHT } else { "#fff" };
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"10\" fill=\"{fill}\" stroke=\"#cbd5e1\" stroke-width=\"1.5\"/>", left + gap * i as f64 + gap / 2.0);
        }
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod breadth_test {
    #[test]
    fn breadth_fills_now_and_later_circles() {
        let rows = vec![("Now".to_string(), 1, None, true), ("2008".to_string(), 1, Some(3), false)];
        let svg = super::breadth(&rows, 6, ["At the start", "Three months in"], "t");
        assert_eq!(svg.matches("r=\"10\"").count(), 12);
        assert_eq!(svg.matches("#f6b89c").count(), 1 + 2); // 범례 1 + 2008년 줄 2개
        assert!(svg.ends_with("</svg>"));
    }
}

/// 경주 그림: 예측마다 평균 오차 막대 하나(작은 쪽이 위), 가장 작은 막대만 파랑. "어느 짐작이 가장 덜 빗나갔나"를 길이로(B5)
pub fn race(rows: &[(String, f64)], unit: &str, title: &str) -> String {
    let mut r: Vec<&(String, f64)> = rows.iter().collect();
    r.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let row = 46.0;
    let h = T + r.len() as f64 * row + 8.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let max = r.iter().map(|x| x.1).fold(0.0, f64::max).max(1e-9);
    let (x0, x1) = (16.0, W - R - 70.0);
    for (k, (name, v)) in r.iter().enumerate() {
        let y = T + k as f64 * row;
        let w = (v / max * (x1 - x0)).max(2.0);
        let fill = if k == 0 { BLUE } else { GRAY };
        let _ = write!(s, "<text x=\"{x0}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{}</text><rect x=\"{x0}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"14\" rx=\"3\" fill=\"{fill}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"14\" font-weight=\"600\">{v:.1}{unit}</text>",
            y + 13.0, name.replace('&', "&amp;"), y + 19.0, x0 + w + 6.0, y + 31.0, if k == 0 { BLUE } else { INK });
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod race_test {
    #[test]
    fn race_sorts_and_highlights_smallest() {
        let svg = super::race(&[("B".into(), 2.4), ("A".into(), 1.7)], " pts", "t");
        let (a, b) = (svg.find(">A<").unwrap(), svg.find(">B<").unwrap());
        assert!(a < b && svg.contains("1.7 pts") && svg.matches("#2563eb").count() >= 2);
    }
}

/// 사다리 그림: 깊이마다 한 줄, 지난 하락 중 그 깊이까지 간 수(막대 길이 = 몫)와 "k of n". 지금 하락이 이미 지난 깊이는 파랑.
/// "이만큼 빠졌을 때 더 깊어진 적이 얼마나 되나"를 위에서 아래로 줄어드는 막대로(B5)
pub fn ladder(rows: &[(String, usize, bool)], n: usize, title: &str) -> String {
    let row = 46.0;
    let h = T + rows.len() as f64 * row + 8.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let (x0, x1) = (16.0, W - R - 92.0);
    for (k, (name, c, now)) in rows.iter().enumerate() {
        let y = T + k as f64 * row;
        let w = (*c as f64 / n.max(1) as f64 * (x1 - x0)).max(2.0);
        let (fill, ink) = if *now { (BLUE, BLUE) } else { (GRAY, INK) };
        let _ = write!(s, "<text x=\"{x0}\" y=\"{:.1}\" fill=\"{ink}\" font-size=\"14\">{}</text><rect x=\"{x0}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"14\" rx=\"3\" fill=\"{fill}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{ink}\" font-size=\"14\" font-weight=\"600\">{c} of {n}</text>",
            y + 13.0, name.replace('&', "&amp;"), y + 19.0, x0 + w + 6.0, y + 31.0);
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod ladder_test {
    #[test]
    fn ladder_marks_reached_rungs_blue() {
        let rows = vec![("Fell 5% or more".to_string(), 74, true), ("10% or more".to_string(), 26, false)];
        let svg = super::ladder(&rows, 74, "t");
        assert!(svg.contains("74 of 74") && svg.contains("26 of 74"));
        assert_eq!(svg.matches("fill=\"#2563eb\"").count(), 3); // 파랑 줄의 이름·막대·값
        assert!(svg.ends_with("</svg>"));
    }
}

/// 경보 기간 그림: 신호마다 한 줄, 켜진 달은 진한 칸, 침체 달은 세로 음영. 월 문자열(YYYY-MM) 기준, from~to
pub fn lights(rows: &[(String, Vec<String>)], rec: &[String], from: &str, to: &str, title: &str) -> String {
    let idx = |m: &str| -> f64 { let y: f64 = m[..4].parse().unwrap_or(0.0); let mo: f64 = m[5..7].parse().unwrap_or(1.0); y * 12.0 + mo - 1.0 };
    let (a, b) = (idx(from), idx(to) + 1.0);
    let (x0, x1) = (16.0, W - R);
    let x = |m: &str| x0 + (idx(m) - a) / (b - a).max(1.0) * (x1 - x0);
    let mw = (x1 - x0) / (b - a).max(1.0);
    let short = b - a < 120.0; // 짧은 기간 그림은 막대 아래 날짜 줄 자리를 둔다
    let (top, rh) = (T + 8.0, if short { 72.0 } else { 52.0 });
    let h = top + rh * rows.len() as f64 + 70.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    for m in rec {
        // 침체 음영은 다른 그림보다 진하게: 막대와 겹쳐 옅으면 읽히지 않았다(2026-11-17 콜드 리드)
        let _ = write!(s, "<rect x=\"{:.1}\" y=\"{top}\" width=\"{:.2}\" height=\"{:.1}\" fill=\"#cbd5e1\"/>", x(m), mw.max(0.6), rh * rows.len() as f64);
    }
    for (k, (name, ms)) in rows.iter().enumerate() {
        let y = top + rh * k as f64;
        let _ = write!(s, "<text x=\"{x0}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{}</text>", y + 16.0, name.replace('&', "&amp;"));
        let _ = write!(s, "<line x1=\"{x0}\" x2=\"{x1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#e5e7eb\"/>", y + 34.0, y + 34.0);
        // 빈 줄은 '없음'을 글로(침체 줄이 비어 있으면 그 빈칸이 결론이다, 2026-11-18)
        if ms.is_empty() {
            let _ = write!(s, "<text x=\"{x0}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\" font-style=\"italic\">None in this period</text>", y + 39.0);
        }
        // 이어진 달은 한 막대로(달마다 칸을 그리면 줄무늬로 쪼개져 보였다)
        let mut i = 0;
        while i < ms.len() {
            let mut j = i;
            while j + 1 < ms.len() && idx(&ms[j + 1]) - idx(&ms[j]) == 1.0 { j += 1; }
            let _ = write!(s, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.2}\" height=\"14\" rx=\"2\" fill=\"{BLUE}\"/>", x(&ms[i]), y + 27.0, (mw * (j - i + 1) as f64).max(2.0));
            // 기간이 짧은 그림(10년 미만)에서 긴 막대엔 처음·끝 달 이름(두 측정의 날짜 차이를 그림으로, 2026-11-18)
            if short && j - i >= 6 {
                const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
                let lab = |m: &str| format!("{} {}", MON[m[5..7].parse::<usize>().unwrap_or(1) - 1], &m[..4]);
                let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">{} to {}</text>", x(&ms[i]), y + 58.0, lab(&ms[i]), lab(&ms[j]));
            }
            i = j + 1;
        }
    }
    // 10년 눈금과 음영 설명
    let yb = top + rh * rows.len() as f64 + 20.0;
    let (ya, yz): (i32, i32) = (from[..4].parse().unwrap_or(0), to[..4].parse().unwrap_or(0));
    let every = if yz - ya < 10 { 1 } else { 10 }; // 짧은 기간은 해마다 눈금
    for yr in (ya..=yz).filter(|y| y % every == 0 && format!("{y}-01").as_str() >= from) {
        let m = format!("{yr}-01");
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{yb:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{yr}</text>", x(&m));
    }
    if !rec.is_empty() {
        let _ = write!(s, "<rect x=\"{x0}\" y=\"{:.1}\" width=\"12\" height=\"12\" fill=\"#cbd5e1\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Recessions</text>", yb + 12.0, x0 + 18.0, yb + 22.0);
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod lights_test {
    #[test]
    fn lights_draws_on_months_and_recessions() {
        let rows = vec![("Curve".to_string(), vec!["2000-01".to_string(), "2000-02".to_string()]), ("Claims".to_string(), vec!["2000-03".to_string()])];
        let svg = super::lights(&rows, &["2000-03".to_string()], "2000-01", "2000-12", "t");
        assert_eq!(svg.matches("fill=\"#2563eb\"").count(), 2); // 1~2월은 한 막대
        assert!(svg.contains("Claims") && svg.ends_with("</svg>"));
    }
}

/// 필요 국채 그림: 목표 금리마다 한 줄. 띠 = 금리 차가 넓을 때~좁을 때(가운데 80%) 필요한 10년 국채, 점 = 보통 금리 차일 때,
/// 세로 점선 = 지금 10년 국채. 줄 아래에 '그만큼 낮았던 주의 몫'. rows: (이름, 넓은 차 필요값, 보통, 좁은 차, 몫)
pub fn need(rows: &[(String, f64, f64, f64, f64)], now: f64, title: &str) -> String {
    const HALO: &str = "stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\"";
    let (top, rh) = (T + 30.0, 66.0);
    let h = top + rh * rows.len() as f64 + 50.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let lo = rows.iter().map(|r| r.1).fold(0.0_f64, f64::min).floor();
    let hi = (now.max(rows.iter().map(|r| r.3).fold(0.0, f64::max)) + 0.5).ceil();
    let (x0, x1) = (16.0, W - R);
    let x = |v: f64| x0 + (v - lo) / (hi - lo).max(0.1) * (x1 - x0);
    let bottom = top + rh * rows.len() as f64;
    for k in (lo as i64)..=(hi as i64) {
        let xv = x(k as f64);
        let _ = write!(s, "<line x1=\"{xv:.1}\" x2=\"{xv:.1}\" y1=\"{top}\" y2=\"{bottom:.1}\" stroke=\"#f1f5f9\"/><text x=\"{xv:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{k}%</text>", bottom + 18.0);
    }
    for (k, (name, wide, mid, tight, share)) in rows.iter().enumerate() {
        let y = top + rh * k as f64;
        let _ = write!(s, "<text x=\"{x0}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">{}</text>", y + 14.0, name.replace('&', "&amp;"));
        let (a, b) = (x(*wide), x(*tight));
        let _ = write!(s, "<rect x=\"{a:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"10\" rx=\"5\" fill=\"#bfdbfe\"/><circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"6\" fill=\"{BLUE}\" stroke=\"#fff\" stroke-width=\"2\"><title>{name}: 10-year near {mid:.1}%</title></circle>",
            y + 22.0, (b - a).max(2.0), x(*mid), y + 27.0);
        let _ = write!(s, "<text x=\"{x0}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\" {HALO}>needs the 10-year near {mid:.1}% · that low in {:.0}% of weeks</text>", y + 52.0, share * 100.0);
    }
    // 범례: 점 = 보통 금리 차, 띠 = 넓은 차~좁은 차(두 독자가 띠 뜻을 몰랐다, 2026-11-18)
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">typical markup</text><rect x=\"{:.1}\" y=\"{:.1}\" width=\"22\" height=\"8\" rx=\"4\" fill=\"#bfdbfe\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">wide to narrow markup</text>",
        x0 + 5.0, bottom + 34.0, x0 + 14.0, bottom + 38.0, x0 + 128.0, bottom + 30.0, x0 + 156.0, bottom + 38.0);
    let xn = x(now);
    let _ = write!(s, "<line x1=\"{xn:.1}\" x2=\"{xn:.1}\" y1=\"{:.1}\" y2=\"{bottom:.1}\" stroke=\"#eb6834\" stroke-width=\"2\" stroke-dasharray=\"4 3\"/><text x=\"{xn:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"#eb6834\" font-size=\"13\" font-weight=\"600\">10-year now {now:.1}%</text></svg>",
        top - 12.0, top - 16.0);
    s
}

#[cfg(test)]
mod need_fig_test {
    #[test]
    fn need_draws_rows_and_now_line() {
        let rows = vec![("3% mortgage".to_string(), 0.5, 1.3, 1.9, 0.05), ("6% mortgage".to_string(), 3.5, 4.3, 4.9, 0.6)];
        let svg = super::need(&rows, 5.1, "t");
        assert!(svg.contains("3% mortgage") && svg.contains("10-year now 5.1%") && svg.contains("that low in 5% of weeks"));
        assert!(svg.ends_with("</svg>"));
    }
}

/// 재역전 그림: 사례마다 한 줄, 첫 역전 달부터 n달. 뒤집힌 달은 파랑, 아닌 달은 옅은 회색, 침체 시작 달에 주황 눈금과 이름.
/// rows: (이름, 달마다 뒤집혔나, 침체 시작 달 위치(없으면 None)), 자료가 끝난 줄은 짧다.
pub fn rounds(rows: &[(String, Vec<bool>, Option<usize>)], months: usize, title: &str) -> String {
    const ORANGE: &str = "#eb6834";
    let (top, rh) = (T + 6.0, 54.0);
    let h = top + rh * rows.len() as f64 + 44.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let (x0, x1) = (16.0, W - R);
    let mw = (x1 - x0) / months.max(1) as f64;
    for (k, (name, on, rec)) in rows.iter().enumerate() {
        let y = top + rh * k as f64;
        let _ = write!(s, "<text x=\"{x0}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">{}</text>", y + 14.0, name.replace('&', "&amp;"));
        for (m, &v) in on.iter().enumerate().take(months) {
            let _ = write!(s, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.2}\" height=\"14\" fill=\"{}\"/>", x0 + m as f64 * mw, y + 22.0, mw + 0.3, if v { BLUE } else { "#e2e8f0" });
        }
        if let Some(r) = rec {
            let xr = x0 + *r as f64 * mw;
            let anchor = if xr > W - 90.0 { "end" } else { "start" };
            let _ = write!(s, "<line x1=\"{xr:.1}\" x2=\"{xr:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{ORANGE}\" stroke-width=\"3\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{ORANGE}\" font-size=\"13\" font-weight=\"600\">Recession</text>",
                y + 18.0, y + 40.0, if anchor == "end" { xr - 4.0 } else { xr + 4.0 }, y + 14.0);
        } else if on.len() < months {
            let xe = x0 + on.len() as f64 * mw;
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Now</text>", xe + 4.0, y + 33.0);
        }
    }
    let yb = top + rh * rows.len() as f64 + 4.0;
    let tick = if months > 48 { 12 } else { 6 };
    for m in (0..=months).step_by(tick) {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{yb:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>", x0 + m as f64 * mw, if m == 0 { "0".to_string() } else { format!("{}y", m / 12) });
    }
    let _ = write!(s, "<rect x=\"{x0}\" y=\"{:.1}\" width=\"12\" height=\"12\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Inverted</text><rect x=\"{:.1}\" y=\"{:.1}\" width=\"12\" height=\"12\" fill=\"#e2e8f0\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Positive</text><text x=\"{x1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">Years after first inverting</text></svg>",
        yb + 14.0, x0 + 18.0, yb + 24.0, x0 + 90.0, yb + 14.0, x0 + 108.0, yb + 24.0, yb + 24.0);
    s
}

#[cfg(test)]
mod rounds_test {
    #[test]
    fn rounds_marks_inverted_months_and_recession() {
        let rows = vec![("1966".to_string(), vec![true, true, false, true], Some(3)), ("2022".to_string(), vec![true, false], None)];
        let svg = super::rounds(&rows, 4, "t");
        assert_eq!(svg.matches("fill=\"#2563eb\"/>").count(), 4 + 1); // 뒤집힌 달 4칸(두 줄) + 범례
        assert!(svg.contains(">Recession<") && svg.contains(">Now<") && svg.ends_with("</svg>"));
    }
}


/// 하락 기록 그림: 무리(시작 해 묶음)마다 한 줄, 그 무리의 시작 주마다 N년 안 최저 금리를 점으로. 세로 기준선(예: 3%, 5%)과 지금 금리.
/// rows: (이름, 최저값들). 점이 모두 기준선 오른쪽이면 '여기서 그만큼 내려간 적이 없다'가 그림으로 보인다.
pub fn falls(rows: &[(String, Vec<f64>)], lines: &[(f64, String)], now: f64, title: &str) -> String {
    const ORANGE: &str = "#eb6834";
    let (top, rh) = (T + 30.0, 30.0);
    let h = top + rh * rows.len() as f64 + 44.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let lo = lines.iter().map(|l| l.0).fold(f64::MAX, f64::min).min(rows.iter().flat_map(|r| r.1.iter().cloned()).fold(f64::MAX, f64::min)).floor() - 1.0;
    let hi = now.max(rows.iter().flat_map(|r| r.1.iter().cloned()).fold(0.0, f64::max)).ceil() + 0.5;
    let (x0, x1) = (110.0, W - R);
    let x = |v: f64| x0 + (v - lo) / (hi - lo).max(0.1) * (x1 - x0);
    let bottom = top + rh * rows.len() as f64;
    for k in (lo.ceil() as i64)..=(hi.floor() as i64) {
        let xv = x(k as f64);
        let _ = write!(s, "<line x1=\"{xv:.1}\" x2=\"{xv:.1}\" y1=\"{top}\" y2=\"{bottom:.1}\" stroke=\"#f1f5f9\"/><text x=\"{xv:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{k}%</text>", bottom + 18.0);
    }
    for (v, name) in lines {
        let xv = x(*v);
        let _ = write!(s, "<line x1=\"{xv:.1}\" x2=\"{xv:.1}\" y1=\"{:.1}\" y2=\"{bottom:.1}\" stroke=\"{INK}\" stroke-dasharray=\"3 3\"/><text x=\"{xv:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>", top - 8.0, top - 12.0, name.replace('&', "&amp;"));
    }
    let xn = x(now);
    let _ = write!(s, "<line x1=\"{xn:.1}\" x2=\"{xn:.1}\" y1=\"{:.1}\" y2=\"{bottom:.1}\" stroke=\"{ORANGE}\" stroke-width=\"2\"/><text x=\"{xn:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{ORANGE}\" font-size=\"13\" font-weight=\"600\">Now {now:.2}%</text>", top - 8.0, top - 12.0);
    for (k, (name, vals)) in rows.iter().enumerate() {
        let y = top + rh * k as f64 + rh / 2.0;
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">{}</text>", x0 - 10.0, y + 5.0, name.replace('&', "&amp;"));
        for (j, v) in vals.iter().enumerate() {
            let jitter = ((j * 7919) % 11) as f64 - 5.0; // 같은 값이 겹치지 않게 줄 안에서 위아래로
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"3.5\" fill=\"{BLUE}\" fill-opacity=\"0.45\"/>", x(*v), y + jitter);
        }
    }
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{BLUE}\" fill-opacity=\"0.6\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">One week like today's: the lowest rate in the five years after it</text></svg>",
        x0 - 96.0, bottom + 36.0, x0 - 88.0, bottom + 40.0);
    s
}

#[cfg(test)]
mod falls_test {
    #[test]
    fn falls_draws_dots_lines_and_now() {
        let rows = vec![("1998-2002".to_string(), vec![5.21, 5.3]), ("1971-72".to_string(), vec![7.23])];
        let svg = super::falls(&rows, &[(3.0, "3%".to_string()), (5.0, "5%".to_string())], 7.11, "t");
        assert_eq!(svg.matches("fill-opacity=\"0.45\"").count(), 3);
        assert!(svg.contains("Now 7.11%") && svg.contains(">1998-2002<") && svg.ends_with("</svg>"));
    }
}


/// 점 띠 그림: 줄(묶음)마다 사례별 값을 점으로, 기준선(0 또는 '보통') 하나, 줄마다 중앙값 표시와 '몇 개 중 몇 개 위'.
/// 한 줄에 점 한 종류만 둔다(점 세 종류×11줄은 읽히지 않았다, 2026-11-18 콜드 리드).
pub fn strips(rows: &[(String, Vec<(String, f64)>)], unit: &str, refline: Option<(f64, &str)>, title: &str) -> String {
    strips_with(rows, unit, refline, "up in", "no change", title)
}

/// strips에 '0보다 큰 사례' 이름을 바꿔 단다(예: 메우기 오차는 "too high in"). 값이 작은 띠(폭 2 미만, %p)는 소수 둘째 자리까지
pub fn strips_with(rows: &[(String, Vec<(String, f64)>)], unit: &str, refline: Option<(f64, &str)>, up_word: &str, zero_word: &str, title: &str) -> String {
    // 줄 이름 아래 점 띠, 0선·기준선은 점 띠 구간에만(이름 글자와 겹쳤다), 범례는 맨 아래 따로(2026-11-18 휴대폰 확인)
    let (top, rh) = (T + 22.0, 76.0);
    let h = top + rh * rows.len() as f64 + 54.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let all: Vec<f64> = rows.iter().flat_map(|r| r.1.iter().map(|p| p.1)).chain(refline.map(|r| r.0)).chain([0.0]).collect();
    let (mut lo, mut hi) = (all.iter().cloned().fold(f64::MAX, f64::min), all.iter().cloned().fold(f64::MIN, f64::max));
    let pad = (hi - lo).max(1e-9) * 0.06;
    lo -= pad;
    hi += pad;
    let (x0, x1) = (16.0, W - R);
    let x = |v: f64| x0 + (v - lo) / (hi - lo) * (x1 - x0);
    let bottom = top + rh * rows.len() as f64;
    // 반올림해서 0이면 부호 없이 0('-0.0 pt'가 나왔다)
    let dp: usize = if unit != "%" && hi - lo < 2.0 { 2 } else { 1 };
    let fmt = |v: f64| if unit == "%" { let r = (v * 100.0).round(); if r == 0.0 { "0%".to_string() } else { format!("{}{:.0}%", if r > 0.0 { "+" } else { "" }, r) } } else { let k = 10f64.powi(dp as i32); let r = (v * k).round() / k; if r == 0.0 { format!("{:.dp$}{unit}", 0.0) } else { format!("{}{:.dp$}{unit}", if r > 0.0 { "+" } else { "" }, r) } };
    let z = x(0.0);
    let _ = write!(s, "<text x=\"{z:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{zero_word}</text>", top - 4.0);
    // 눈금(두 독자가 점 띠에 눈금이 없어 크기를 못 읽었다, 2026-11-18): 둥근 값 4~6개를 맨 아래에
    let span = hi - lo;
    let raw = span / 5.0;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0].iter().map(|m| m * mag).find(|st| span / st <= 6.0).unwrap_or(raw);
    let mut t = (lo / step).ceil() * step;
    while t <= hi + 1e-9 {
        let td = if step < 1.0 && unit != "%" { if step < 0.1 { 2 } else { 1 } } else { 0 };
        let lab = if unit == "%" { format!("{:.0}%", t * 100.0) } else { format!("{:.td$}{unit}", if t.abs() < 1e-9 { 0.0 } else { t }) }; // 단위를 눈금에(독자가 단위를 못 읽었다)
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"12\">{lab}</text>", x(t), bottom + 20.0);
        t += step;
    }
    if let Some((v, name)) = refline {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{BLUE}\" font-size=\"13\">{}</text>", x(v), top - 4.0, name.replace('&', "&amp;"));
    }
    for (k, (name, pts)) in rows.iter().enumerate() {
        let y = top + rh * k as f64;
        let up = pts.iter().filter(|p| p.1 > 0.0).count();
        let mut v: Vec<f64> = pts.iter().map(|p| p.1).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = if v.is_empty() { 0.0 } else if v.len() % 2 == 1 { v[v.len() / 2] } else { (v[v.len() / 2 - 1] + v[v.len() / 2]) / 2.0 };
        let _ = write!(s, "<text x=\"{x0}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">{}</text><text x=\"{x1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{up_word} {up} of {} · middle {}</text>",
            y + 16.0, name.replace('&', "&amp;"), y + 16.0, pts.len(), fmt(med));
        let cy = y + 46.0;
        let _ = write!(s, "<line x1=\"{x0}\" x2=\"{x1}\" y1=\"{cy:.1}\" y2=\"{cy:.1}\" stroke=\"#eef2f7\"/><line x1=\"{z:.1}\" x2=\"{z:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\"/>", cy - 16.0, cy + 16.0);
        if let Some((rv, _)) = refline {
            let xr = x(rv);
            let _ = write!(s, "<line x1=\"{xr:.1}\" x2=\"{xr:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{BLUE}\" stroke-dasharray=\"4 3\"/>", cy - 16.0, cy + 16.0);
        }
        for (j, (lab, val)) in pts.iter().enumerate() {
            let jitter = ((j * 7919) % 9) as f64 - 4.0;
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"6\" fill=\"{BLUE}\" fill-opacity=\"0.55\" stroke=\"#fff\"><title>{lab}: {}</title></circle>", x(*val), cy + jitter, fmt(*val));
        }
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#0f172a\" stroke-width=\"3\"/>", x(med), x(med), cy - 12.0, cy + 12.0);
        // 양 끝 점에 해(어느 사례인지 찾을 수 없었다)
        if let (Some(mn), Some(mx)) = (pts.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()), pts.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())) {
            for (p, anchor, dx) in [(mn, "start", -6.0), (mx, "end", 6.0)] {
                let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{INK}\" font-size=\"12\">{}</text>", x(p.1) + dx, cy + 22.0, &p.0[..4.min(p.0.len())]);
            }
        }
    }
    let ly = bottom + 46.0;
    let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#0f172a\" stroke-width=\"3\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">middle case</text><circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"6\" fill=\"{BLUE}\" fill-opacity=\"0.55\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">one case</text></svg>",
        x0 + 4.0, x0 + 4.0, ly - 10.0, ly + 4.0, x0 + 12.0, ly + 1.0, x0 + 120.0, ly - 3.0, x0 + 130.0, ly + 1.0);
    s
}

#[cfg(test)]
mod strips_test {
    #[test]
    fn strips_counts_up_and_marks_median() {
        let rows = vec![("A".to_string(), vec![("1".to_string(), 1.0), ("2".to_string(), -0.5), ("3".to_string(), 2.0)])];
        let svg = super::strips(&rows, " pt", None, "t");
        assert!(svg.contains("up in 2 of 3") && svg.contains("middle +1.0 pt") && svg.ends_with("</svg>"));
        // 작은 값(메우기 오차): 둘째 자리, 이름 바꿈
        let small = vec![("M".to_string(), vec![("1985-10".to_string(), 0.05), ("2008-10".to_string(), -0.2), ("2005-10".to_string(), 0.51)])];
        let svg = super::strips_with(&small, " pt", None, "too high in", "exact", "t");
        assert!(svg.contains("too high in 2 of 3") && svg.contains("middle +0.05 pt"), "{svg}");
    }
}


/// 사건 전후 그림: 사건일을 0으로 맞춰 앞 before일부터 뒤 after일까지, 사건일 값 대비 변화(수준 차이). 사례마다 옅은 선, 중앙값 굵은 선, 0일 세로선.
/// '움직임이 사건 전에 일어났나 뒤에 일어났나'를 한 그림으로(2026-11-19 연준 첫 인하와 대출 금리).
pub fn around(paths: &[(String, Vec<(f64, f64)>)], before: f64, after: f64, unit: &str, event_label: &str, title: &str, highlight: &[(String, String)]) -> String {
    let all: Vec<f64> = paths.iter().flat_map(|p| p.1.iter().map(|q| q.1)).collect();
    let (mut lo, mut hi) = (all.iter().cloned().fold(0.0_f64, f64::min), all.iter().cloned().fold(0.0_f64, f64::max));
    let pad = (hi - lo).max(0.1) * 0.08;
    lo -= pad;
    hi += pad;
    let (l, r, t, b) = (66.0, 34.0, T + 22.0, 40.0); // 왼쪽 눈금 글자의 부호가 잘렸다
    let x = |d: f64| l + (d + before) / (before + after) * (W - l - r);
    let y = |v: f64| t + (hi - v) / (hi - lo) * (H - t - b);
    let mut s = head(title);
    // 가로 눈금(값)
    let span = hi - lo;
    let step = [0.25, 0.5, 1.0, 2.0].iter().cloned().find(|st| span / st <= 6.0).unwrap_or(2.0);
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}{v:.1}{unit}</text>",
            W - r, y(v), y(v), if v.abs() < 1e-9 { INK } else { "#eef2f7" }, l - 4.0, y(v) + 4.0, if v > 0.0 { "+" } else { "" });
        v += step;
    }
    // 사례 선
    for (_, p) in paths {
        let d: Vec<String> = p.iter().map(|q| format!("{:.1},{:.1}", x(q.0), y(q.1))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{GRAY}\" stroke-width=\"1.2\" stroke-opacity=\"0.6\"/>", d.join(" "));
    }
    // 짚을 사례: 주황 선과 끝 이름(예: 예상 못 한 1987 인하)
    for (key, _) in highlight {
        if let Some((_, p)) = paths.iter().find(|p| &p.0 == key) {
            let d: Vec<String> = p.iter().map(|q| format!("{:.1},{:.1}", x(q.0), y(q.1))).collect();
            let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"#eb6834\" stroke-width=\"2\"/>", d.join(" "));
        }
    }
    // 중앙값 선: 30일 간격 칸마다 각 사례의 가장 가까운 값
    let mut med = Vec::new();
    let mut dd = -before;
    while dd <= after + 1e-9 {
        let mut vals: Vec<f64> = paths.iter().filter_map(|p| p.1.iter().min_by(|a, b| (a.0 - dd).abs().partial_cmp(&(b.0 - dd).abs()).unwrap()).filter(|q| (q.0 - dd).abs() <= 20.0).map(|q| q.1)).collect();
        if !vals.is_empty() {
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = vals.len();
            med.push((dd, if n % 2 == 1 { vals[n / 2] } else { (vals[n / 2 - 1] + vals[n / 2]) / 2.0 }));
        }
        dd += 15.0;
    }
    let md: Vec<String> = med.iter().map(|q| format!("{:.1},{:.1}", x(q.0), y(q.1))).collect();
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{BLUE}\" stroke-width=\"3\"/>", md.join(" "));
    // 0일 세로선과 이름
    let x0 = x(0.0);
    let _ = write!(s, "<line x1=\"{x0:.1}\" x2=\"{x0:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-width=\"1.5\" stroke-dasharray=\"4 3\"/><text x=\"{x0:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\" font-weight=\"600\">{}</text>",
        t - 6.0, H - b, t - 10.0, event_label.replace('&', "&amp;"));
    // 가로축: 3달마다 눈금(독자가 '3달 전' 값을 그림에서 찾지 못했다, 2026-11-19)
    let mut m = -(before / 30.4).round() as i64;
    while (m as f64) * 30.4 <= after + 20.0 {
        if m % 3 == 0 && m != 0 {
            let d = (m as f64 * 30.4).clamp(-before, after);
            let lab = if m < 0 { format!("{m} mo") } else { format!("+{m} mo") };
            let anchor = "middle"; // 끝 눈금도 가운데: 오른쪽 여백을 넓혀 +9·+12가 붙지 않게
            let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{INK}\" font-size=\"13\">{lab}</text>", x(d), x(d), t, H - b, x(d), H - b + 18.0);
        }
        m += 1;
    }
    // 범례
    let ly = H - 8.0;
    let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{BLUE}\" stroke-width=\"3\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">middle case</text><line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{GRAY}\" stroke-width=\"1.5\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">each cycle</text>",
        l + 18.0, ly - 4.0, ly - 4.0, l + 24.0, ly, l + 120.0, l + 138.0, ly - 4.0, ly - 4.0, l + 144.0, ly);
    let mut lx = l + 240.0;
    for (_, name) in highlight {
        let _ = write!(s, "<line x1=\"{lx:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eb6834\" stroke-width=\"2\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">{}</text>", lx + 18.0, ly - 4.0, ly - 4.0, lx + 24.0, ly, name.replace('&', "&amp;"));
        lx += 30.0 + name.len() as f64 * 7.0;
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod around_test {
    #[test]
    fn around_draws_event_line_and_median() {
        let paths = vec![("a".to_string(), vec![(-90.0, 0.5), (0.0, 0.0), (90.0, 0.1)]), ("b".to_string(), vec![(-90.0, 0.3), (0.0, 0.0), (90.0, -0.1)])];
        let svg = super::around(&paths, 182.0, 365.0, " pt", "First cut", "t", &[("a".to_string(), "Case A".to_string())]);
        assert!(svg.contains(">Case A<") && svg.contains("-3 mo"));
        assert!(svg.contains(">First cut<") && svg.contains("middle case") && svg.ends_with("</svg>"));
    }
}


/// 여러 계열 선 그림(수준): 같은 단위(%)의 계열 2~4개를 한 축에, 끝에 이름, 날짜 하나에 세로 표시(예: 연준 첫 인하일).
/// 기준금리가 내려도 10년 국채·대출 금리가 오른 구간처럼 '따로 움직인다'를 위치로 보이려고(2026-11-19)
pub fn lines(series: &[(String, Vec<(i64, f64)>)], mark: Option<(i64, &str)>, ticks: &[(i64, String)], title: &str) -> String {
    lines_with(series, mark, ticks, &["#2563eb", "#eb6834", "#0f172a", "#16a34a"], title)
}

/// 색을 정해 그리는 선 그림(같은 글의 다른 그림에서 주황이 '금리 오름'처럼 뜻을 맡았으면 겹치지 않게 고른다)
pub fn lines_with(series: &[(String, Vec<(i64, f64)>)], mark: Option<(i64, &str)>, ticks: &[(i64, String)], colors: &[&str], title: &str) -> String {
    lines_band(series, mark, ticks, colors, None, title)
}

/// band: (시작일, 끝일, 이름) 구간을 옅게 칠하고 위에 이름을 적는다
pub fn lines_band(series: &[(String, Vec<(i64, f64)>)], mark: Option<(i64, &str)>, ticks: &[(i64, String)], colors: &[&str], band: Option<(i64, i64, &str)>, title: &str) -> String {
    let colors = if colors.is_empty() { &["#2563eb", "#eb6834", "#0f172a", "#16a34a"][..] } else { colors };
    let all: Vec<f64> = series.iter().flat_map(|s| s.1.iter().map(|p| p.1)).collect();
    let (mut lo, mut hi) = (all.iter().cloned().fold(f64::MAX, f64::min), all.iter().cloned().fold(f64::MIN, f64::max));
    let pad = (hi - lo).max(0.1) * 0.08;
    lo -= pad;
    hi += pad;
    let d0 = series.iter().flat_map(|s| s.1.first().map(|p| p.0)).min().unwrap_or(0);
    let d1 = series.iter().flat_map(|s| s.1.last().map(|p| p.0)).max().unwrap_or(1);
    let (l, r, t, b) = (44.0, 130.0, T + 22.0, 30.0);
    let x = |d: i64| l + (d - d0) as f64 / (d1 - d0).max(1) as f64 * (W - l - r);
    let y = |v: f64| t + (hi - v) / (hi - lo) * (H - t - b);
    let mut s = head(title);
    // 눈금 간격: 폭이 8%p를 넘으면 2%p마다(1%p마다면 글자가 빽빽했다)
    let step = if hi - lo > 8.0 { 2.0 } else { 1.0 };
    let mut v = (lo / step).ceil() * step + 0.0;
    if v.abs() < 1e-9 { v = 0.0; }
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{v:.0}%</text>", W - r, y(v), y(v), if v.abs() < 1e-9 && lo < 0.0 { INK } else { "#eef2f7" }, l - 4.0, y(v) + 4.0);
        v += step;
        if v.abs() < 1e-9 { v = 0.0; } // -0% 대신 0%
    }
    if let Some((b0, b1, name)) = band {
        let _ = write!(s, "<rect x=\"{:.1}\" y=\"{t:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{SHADE}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       x(b0), (x(b1) - x(b0)).max(2.0), H - t - b, (x(b0) + x(b1)) / 2.0, t - 6.0, name.replace('&', "&amp;"));
    }
    if let Some((md, name)) = mark {
        let xm = x(md);
        let _ = write!(s, "<line x1=\"{xm:.1}\" x2=\"{xm:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{GRAY}\" stroke-dasharray=\"4 3\"/><text x=\"{xm:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\" font-weight=\"600\">{}</text>", t - 4.0, H - b, t - 8.0, name.replace('&', "&amp;"));
    }
    // 끝 이름표 자리: 겹치면 18px 이상 벌린다
    let mut ends: Vec<(usize, f64)> = series.iter().enumerate().filter_map(|(k, s)| s.1.last().map(|p| (k, y(p.1)))).collect();
    ends.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for k in 1..ends.len() { if ends[k].1 - ends[k - 1].1 < 18.0 { ends[k].1 = ends[k - 1].1 + 18.0; } }
    for (k, (name, pts)) in series.iter().enumerate() {
        let c = colors[k % colors.len()];
        let d: Vec<String> = pts.iter().map(|p| format!("{:.1},{:.1}", x(p.0), y(p.1))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{c}\" stroke-width=\"2.5\"/>", d.join(" "));
        let ly = ends.iter().find(|e| e.0 == k).map(|e| e.1).unwrap_or(0.0);
        let lastv = pts.last().map(|p| p.1).unwrap_or(0.0);
        // 수준(금리 %)은 소수 둘째 자리, 변화(--rebase, 0선이 있는 그림)는 첫째 자리와 부호(설계 검토: -11.62%는 본문 11.6%보다 정밀해 보였다)
        let lab = if lo < 0.0 && hi > 0.0 { format!("{}{lastv:.1}%", if lastv > 0.0 { "+" } else { "" }) } else { format!("{lastv:.2}%") };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{c}\" font-size=\"13\" font-weight=\"600\">{} {lab}</text>", W - r + 6.0, ly + 4.0, name.replace('&', "&amp;"));
    }
    for (d, lab) in ticks {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{lab}</text>", x(*d), H - 8.0);
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod lines_test {
    #[test]
    fn lines_draws_each_series_and_mark() {
        let sers = vec![("Fed".to_string(), vec![(0, 5.5), (100, 4.5)]), ("10-year".to_string(), vec![(0, 3.8), (100, 4.6)])];
        let svg = super::lines(&sers, Some((50, "First cut")), &[(0, "Jun".into())], "t");
        assert!(svg.contains("Fed 4.50%") && svg.contains("10-year 4.60%") && svg.contains(">First cut<") && svg.ends_with("</svg>"));
    }
}


/// 해마다 그해 안의 가장 깊은 하락(그해 최고 종가에서 그 뒤 최저 종가, 막대 아래로)과 그해 등락(점).
/// '조정은 몇 년에 한 번 오나'와 '그래도 그해는 올랐나'를 한 그림에(2026-11-20). line: 짚을 하락 깊이(예: 0.10)
pub fn intrayear(rows: &[(i32, f64, f64)], line: f64, title: &str) -> String {
    let lo = rows.iter().map(|r| r.1.min(r.2)).fold(-0.1_f64, f64::min).min(-line) * 1.08;
    let hi = rows.iter().map(|r| r.2).fold(0.1_f64, f64::max) * 1.08;
    let (l, r, t, b) = (44.0, 16.0, T + 22.0, 46.0);
    let n = rows.len().max(1) as f64;
    let bw = (W - l - r) / n;
    let x = |i: usize| l + i as f64 * bw;
    let y = |v: f64| t + (hi - v) / (hi - lo) * (H - t - b);
    let mut s = head(title);
    for v in [-0.4_f64, -0.2, 0.0, 0.2, 0.4] {
        if v < lo || v > hi { continue; }
        let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}{:.0}%</text>",
            W - r, y(v), y(v), if v == 0.0 { INK } else { "#eef2f7" }, l - 4.0, y(v) + 4.0, if v > 0.0 { "+" } else { "" }, v * 100.0);
    }
    // 짚는 깊이 선
    let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"3 3\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\" font-weight=\"600\">-{:.0}%</text>",
        W - r, y(-line), y(-line), l - 4.0, y(-line) + 4.0, line * 100.0);
    for (i, (_, drop, ret)) in rows.iter().enumerate() {
        let c = if -drop >= 0.20 { "#1e3a8a" } else if -drop >= line { BLUE } else { GRAY }; // 20% 넘은 해는 짙게: 13해를 그림에서 바로 센다(설계 검토 2026-11-20)
        let _ = write!(s, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{c}\"/>", x(i) + 0.5, y(0.0), (bw - 1.0).max(1.0), y(*drop) - y(0.0));
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"2\" fill=\"#eb6834\"/>", x(i) + bw / 2.0, y(*ret));
    }
    // 마지막 해가 진행 중이면 'so far'(설계 검토: 셈은 1950~2025인데 그림엔 2026이 있다)
    if let Some((yr, _, ret)) = rows.last() {
        if *yr >= 2026 {
            let i = rows.len() - 1;
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"12\">{yr} so far</text>", x(i) + bw, y(ret.max(0.0)) - 10.0);
        }
    }
    // 가로축: 10년마다
    for (i, (yr, _, _)) in rows.iter().enumerate() {
        if yr % 10 == 0 {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{yr}</text>", x(i) + bw / 2.0, H - b + 16.0);
        }
    }
    let ly = H - 8.0;
    let _ = write!(s, "<rect x=\"{l}\" y=\"{:.1}\" width=\"10\" height=\"10\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">fell {:.0}%+ within the year</text><circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"#eb6834\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">how the year ended</text><rect x=\"{:.1}\" y=\"{:.1}\" width=\"10\" height=\"10\" fill=\"#1e3a8a\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">20%+</text></svg>",
        ly - 10.0, l + 14.0, ly, line * 100.0, l + 214.0, ly - 4.0, l + 222.0, ly, l + 360.0, ly - 10.0, l + 374.0, ly);
    s
}

#[cfg(test)]
mod intrayear_test {
    #[test]
    fn intrayear_marks_deep_years() {
        let rows = vec![(1990, -0.2, -0.06), (1991, -0.06, 0.26)];
        let svg = super::intrayear(&rows, 0.10, "t");
        assert!(svg.contains("#2563eb") && svg.contains("how the year ended") && svg.ends_with("</svg>"));
    }
}


/// 줄마다 이름 하나, 같은 축(%)에 표시 여러 개(예: 평소 1년 변화 · 올해 변화 · 연말 할인 폭). 0 세로선.
/// '할인이 1년 추세보다 작다'를 위치로 보이려고(설계 검토 2026-11-20). 값은 비율(0.097 = 9.7%)
/// unit: "pt"(금리 %p), "yr"(년, 0부터 오른쪽으로: 회복까지 걸린 해), 그 밖은 %
pub fn rowdots(rows: &[(String, Vec<f64>)], marks: &[String], unit: &str, line: Option<(f64, &str)>, title: &str) -> String {
    let points = unit == "pt";
    let years = unit == "yr";
    const C: [&str; 4] = ["#94a3b8", "#2563eb", "#eb6834", "#0f172a"];
    let all: Vec<f64> = rows.iter().flat_map(|r| r.1.iter().cloned()).collect();
    let lo = all.iter().cloned().fold(0.0_f64, f64::min) * 1.1;
    let hi = all.iter().cloned().fold(0.0_f64, f64::max) * 1.1;
    let (l, r, t, b) = (118.0, 20.0, T + 22.0, 52.0);
    let x = |v: f64| l + (v - lo) / (hi - lo).max(1e-9) * (W - l - r);
    let rh = (H - t - b) / rows.len().max(1) as f64;
    let mut s = head(title);
    // points: 값이 %p(금리 변화)일 때 눈금을 1 pt 단위로
    let steps: &[f64] = if points { &[0.5, 1.0, 2.0] } else if years { &[1.0, 2.0, 5.0] } else { &[0.02, 0.05, 0.1] };
    let step = steps.iter().cloned().find(|st| (hi - lo) / st <= 8.0).unwrap_or(steps[steps.len() - 1]);
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{t}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"12\">{}</text>",
            x(v), x(v), H - b, if v.abs() < 1e-9 { INK } else { "#eef2f7" }, x(v), H - b + 14.0,
            if v.abs() < 1e-9 { "0".to_string() } else if years { format!("{v} yr") } else if points { format!("{}{} pt", if v > 0.0 { "+" } else { "" }, v) } else { format!("{}{:.0}%", if v > 0.0 && unit != "level" { "+" } else { "" }, v * 100.0) });
        v += step;
    }
    // --line "값|이름": 기준선(예: 중앙값) 점선과 위쪽 이름표. 점 아래에 깔리게 먼저 그린다
    if let Some((v, name)) = line {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"12\">{}</text>",
            x(v), x(v), t - 4.0, H - b, x(v) + 4.0, t - 8.0, name.replace('&', "&amp;"));
    }
    for (i, (name, vals)) in rows.iter().enumerate() {
        let y = t + rh * (i as f64 + 0.5);
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text>", l - 8.0, y + 4.0, name.replace('&', "&amp;"));
        let (mn, mx) = vals.iter().fold((f64::MAX, f64::MIN), |a, &v| (a.0.min(v), a.1.max(v)));
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{y:.1}\" y2=\"{y:.1}\" stroke=\"#e2e8f0\" stroke-width=\"2\"/>", x(mn), x(mx));
        for (k, v) in vals.iter().enumerate() {
            // 년 단위: 겹치는 점이 서로 가리지 않게 뒤 표시일수록 작게(첫 표시는 큰 테두리 원). 같은 값이면 겹친 고리로 보인다
            let (r, fill, stroke) = if years {
                if k == 0 { (7.5, "#fff".to_string(), "#64748b".to_string()) } else { (6.5 - k as f64, C[k % C.len()].to_string(), "#fff".to_string()) }
            } else { (5.0, C[k % C.len()].to_string(), "#fff".to_string()) };
            let sw = if years && k == 0 { 2.5 } else { 1.5 };
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"{r}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"/>", x(*v));
        }
        // 년 단위: 5년 넘는 긴 기다림은 가장 긴 값 오른쪽에 숫자
        if years && mx >= 5.0 {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"12\">{:.0} yr</text>", x(mx) + 14.0, y + 4.0, mx);
        }
    }
    let ly = H - 10.0;
    let mut lx = l;
    for (k, m) in marks.iter().enumerate() {
        let (lf, ls) = if years && k == 0 { ("#fff", "#64748b") } else { (C[k % C.len()], "none") };
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{lf}\" stroke=\"{ls}\" stroke-width=\"2\"/><text x=\"{:.1}\" y=\"{ly:.1}\" fill=\"{INK}\" font-size=\"13\">{}</text>", lx + 5.0, ly - 4.0, lx + 14.0, m.replace('&', "&amp;"));
        lx += 26.0 + m.len() as f64 * 7.0;
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod rowdots_test {
    #[test]
    fn dots_draws_rows_and_legend() {
        let rows = vec![("Computers".to_string(), vec![-0.09, 0.097, -0.007])];
        let svg = super::rowdots(&rows, &["Usual year".into(), "This year".into(), "Holiday dip".into()], "", None, "t");
        assert!(svg.contains(">+5%<"));
        let pts = super::rowdots(&[("1994".to_string(), vec![3.0, 1.59, 1.83])], &["Fed".into()], "pt", None, "t");
        assert!(pts.contains(">+1 pt<") && !pts.contains("%<"));
        let yrs = super::rowdots(&[("1973".to_string(), vec![7.5, 3.5, 12.0])], &["Price".into()], "yr", Some((2.0, "Median")), "t");
        assert!(yrs.contains(">10 yr<") && !yrs.contains("%<") && !yrs.contains("+"));
        assert!(yrs.contains(">12 yr<")); // 5년 넘는 줄은 가장 긴 값에 숫자
        assert!(yrs.contains(">Median<") && yrs.contains("stroke-dasharray"));
        assert!(svg.contains(">Computers<") && svg.contains(">Holiday dip<") && svg.ends_with("</svg>"));
    }
}

/// 침체 뒤 그림: 줄마다 침체 하나. 회색 막대 = 공식 침체(시작→끝), 파랑 막대 = 끝→일자리가 시작 전 고점을 되찾은 달,
/// 주황 눈금 = 실업률이 가장 높았던 달. "공식 끝"과 "일자리로 느끼는 끝"의 차이를 같은 달 축에서 길이로 읽게(B5)
pub fn aftermath(rows: &[(String, f64, Option<f64>, f64)], title: &str) -> String {
    const ORANGE: &str = "#ea580c";
    let (row, leg) = (24.0, 46.0);
    let h = T + leg + rows.len() as f64 * row + B + 8.0;
    let max = rows.iter().map(|r| r.1 + r.2.unwrap_or(0.0)).fold(12.0, f64::max);
    let xmax = (max / 12.0).ceil() * 12.0;
    let (l, r) = (48.0, 34.0);
    let x = |v: f64| l + v / xmax * (W - l - r);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let _ = write!(s, "<rect x=\"2\" y=\"{:.1}\" width=\"16\" height=\"10\" rx=\"2\" fill=\"{GRAY}\"/><text x=\"24\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Recession</text>\
<rect x=\"104\" y=\"{:.1}\" width=\"16\" height=\"10\" rx=\"2\" fill=\"{BLUE}\"/><text x=\"126\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">After the end, until jobs were back</text>\
<line x1=\"2\" x2=\"2\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{ORANGE}\" stroke-width=\"3\"/><text x=\"10\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Unemployment rate at its peak</text>",
                   T + 2.0, T + 11.0, T + 2.0, T + 11.0, T + 19.0, T + 32.0, T + 30.0);
    let top = T + leg;
    let mut v = 0.0;
    while v <= xmax + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{top}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       x(v), x(v), h - B, x(v), h - B + 16.0, if v == 0.0 { "Start".to_string() } else { format!("{} yr", v / 12.0) });
        v += 12.0;
    }
    for (k, (name, months, back, peak)) in rows.iter().enumerate() {
        let y = top + k as f64 * row + 6.0;
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"14\">{}</text><rect x=\"{l}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"12\" rx=\"2\" fill=\"{GRAY}\"/>",
                       l - 8.0, y + 11.0, name.replace('&', "&amp;"), (x(*months) - l - 1.0).max(2.0));
        let end = match back {
            Some(b) => { let _ = write!(s, "<rect x=\"{:.1}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"12\" rx=\"2\" fill=\"{BLUE}\"/>", x(*months) + 1.0, (x(months + b) - x(*months) - 1.0).max(2.0)); months + b }
            None => *months,
        };
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{ORANGE}\" stroke-width=\"3\" stroke-linecap=\"round\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       x(*peak), x(*peak), y - 3.0, y + 15.0, x(end.max(*peak)) + 6.0, y + 11.0, back.map_or(String::new(), |b| format!("+{b:.0} mo")));
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod aftermath_test {
    #[test]
    fn aftermath_draws_two_bars_and_tick_per_row() {
        let rows = vec![("2008".to_string(), 18.0, Some(59.0), 21.0), ("2020".to_string(), 2.0, Some(26.0), 1.0)];
        let svg = super::aftermath(&rows, "t");
        assert_eq!(svg.matches("fill=\"#2563eb\"").count(), 1 + 2);
        assert!(svg.contains(">+59 mo<") && svg.contains(">+26 mo<"));
        assert!(svg.ends_with("</svg>"));
    }
}

/// 가로 막대: 줄마다 이름과 값 하나(%p), 기준선 하나(예: 첫 인상 폭 0.25). 기준을 넘은 막대는 진한 회색, 못 넘은 막대는 연한 회색,
/// 강조 줄(--highlight)은 파랑. "모든 사례에서 얼마나 움직였나"를 선 다발 대신 길이로(독자 3명이 선 그림을 복잡하다고 짚음, 2026-11-21)
pub fn hbars(rows: &[(String, f64)], line: Option<(f64, String)>, highlight: &str, unit: &str, title: &str) -> String {
    let (row, top) = (30.0, T + 30.0);
    let h = top + rows.len() as f64 * row + B + 8.0;
    let max = rows.iter().map(|r| r.1).fold(line.as_ref().map_or(0.0, |l| l.0), f64::max) * 1.15;
    let l = rows.iter().map(|r| r.0.chars().count()).max().map(|n| n as f64 * 7.6 + 14.0).unwrap_or(52.0).max(52.0);
    let r = 64.0;
    let x = |v: f64| l + v.max(0.0) / max.max(1e-9) * (W - l - r);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    if let Some((lv, lab)) = &line {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#ea580c\" stroke-width=\"2\" stroke-dasharray=\"4 3\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       x(*lv), x(*lv), top - 8.0, h - B, x(*lv) + 5.0, top - 12.0, lab.replace('&', "&amp;"));
    }
    for (k, (name, v)) in rows.iter().enumerate() {
        let y = top + k as f64 * row + 8.0;
        let hl = name == highlight;
        let c = if hl { BLUE } else if line.as_ref().map_or(true, |l| *v >= l.0) { INK } else { GRAY };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{}\" font-size=\"14\"{}>{}</text><rect x=\"{l}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"14\" rx=\"2\" fill=\"{c}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"13\">+{:.2}{unit}</text>",
                       l - 8.0, y + 12.0, if hl { BLUE } else { INK }, if hl { " font-weight=\"600\"" } else { "" }, name.replace('&', "&amp;"), (x(*v) - l).max(2.0), x(v.max(line.as_ref().map_or(0.0, |l| l.0))) + 6.0, y + 12.0, if hl { BLUE } else { INK }, v);
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod hbars_test {
    #[test]
    fn hbars_colors_by_line_and_highlight() {
        let rows = vec![("1994".to_string(), 0.22), ("2022".to_string(), 1.26), ("2026".to_string(), 0.48)];
        let svg = super::hbars(&rows, Some((0.25, "The first hike, 0.25".into())), "2026", " pt", "t");
        assert!(svg.contains("fill=\"#94a3b8\"") && svg.contains("fill=\"#2563eb\"") && svg.contains(">+1.26 pt<") && svg.contains("The first hike"));
    }
}

/// 쿠션 그림(산점도): 해마다 점 하나, 가로 = 그해 이자 쿠션(시작 금리 ÷ 듀레이션, %p), 세로 = 그해 금리 변화(%p).
/// 대각선(상승 = 쿠션) 위쪽에 놓인 해는 값 손실이 한 해 이자를 넘은 해다: 손실 해는 진한 점, 나머지는 회색, 아직 안 끝난 해는 파랑.
/// "채권 펀드는 언제 돈을 잃나"를 규칙 하나(선)와 실제 해들(점)로 한 번에 보인다(Cleveland & McGill: 같은 축 위치 비교)
pub fn cushion(rows: &[(i32, f64, f64, bool, bool)], marks: &[String], title: &str) -> String {
    let (l, r, top) = (52.0, 18.0, T + 26.0);
    let h = H + 40.0;
    let bottom = h - B - 18.0;
    let xmax = (rows.iter().map(|x| x.1).fold(0.5, f64::max) * 2.0).ceil() / 2.0;
    let ymax = rows.iter().map(|x| x.2).fold(0.5, f64::max).ceil().max(xmax.ceil());
    let ymin = rows.iter().map(|x| x.2).fold(-0.5, f64::min).floor();
    let x = |v: f64| l + v / xmax * (W - l - r);
    let y = |v: f64| top + (ymax - v) / (ymax - ymin) * (bottom - top);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    // 범례
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{INK}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Fund lost money that year</text><circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{GRAY}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Made money</text>",
                   l, T + 6.0, l + 10.0, T + 10.0, l + 214.0, T + 6.0, l + 224.0, T + 10.0);
    // 눈금
    let mut v = ymin;
    while v <= ymax + 1e-9 {
        let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       W - r, y(v), y(v), if v == 0.0 { "#cbd5e1" } else { "#eef2f7" }, l - 6.0, y(v) + 4.0, if v > 0.0 { format!("+{v}") } else { format!("{v}") });
        v += 1.0;
    }
    let mut v = 0.0;
    while v <= xmax + 1e-9 {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>", x(v), bottom + 16.0, v);
        v += if xmax > 2.0 { 1.0 } else { 0.5 };
    }
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">Cushion: starting yield ÷ duration (points)</text>", (l + W - r) / 2.0, bottom + 34.0);
    let _ = write!(s, "<text transform=\"translate(14 {:.1}) rotate(-90)\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">Change in yield that year (points)</text>", (top + bottom) / 2.0);
    // 대각선: 상승 = 쿠션
    let xe = xmax.min(ymax);
    let _ = write!(s, "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#ea580c\" stroke-width=\"2\" stroke-dasharray=\"5 4\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"#c2410c\" font-size=\"13\">Rise = cushion</text>",
                   x(0.0), y(0.0), x(xe), y(xe), x(xe) - 4.0, y(xe) - 6.0);
    for (yr, c, ch, loss, full) in rows {
        let (fill, rad) = if !full { (BLUE, 6.0) } else if *loss { (INK, 5.0) } else { (GRAY, 4.0) };
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{rad}\" fill=\"{fill}\" stroke=\"#fff\" stroke-width=\"1.5\"/>", x(*c), y(*ch));
        if !full || marks.iter().any(|m| m == &yr.to_string()) {
            let lab = if *full { yr.to_string() } else { format!("{yr} so far") };
            let right = x(*c) < W - r - 90.0;
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"{}\" fill=\"{}\" font-size=\"13\" font-weight=\"600\" stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\">{lab}</text>",
                           if right { x(*c) + 9.0 } else { x(*c) - 9.0 }, y(*ch) + 4.0, if right { "start" } else { "end" }, if *full { INK } else { BLUE });
        }
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod cushion_test {
    #[test]
    fn cushion_colors_losses_and_labels_marks() {
        let rows = vec![(2021, 0.1, 0.8, true, true), (2020, 0.3, -1.2, false, true), (2026, 0.65, -0.3, false, false)];
        let svg = super::cushion(&rows, &["2021".to_string()], "t");
        assert!(svg.contains(">2021<") && svg.contains("2026 so far") && svg.contains("Rise = cushion") && !svg.contains(">2020<"));
    }
}

/// 속도 그림(산점도): 하락마다 점 하나, 가로 = 기록에서 처음 threshold 아래로 닫힌 날까지 걸린 날(빨리 빠졌나), 세로 = 끝난 깊이.
/// deep 선 아래(약세장까지 간 것)는 진한 점, 그 위에서 멈춘 것은 회색, 아직 진행 중인 하락은 파랑(지금까지 깊이).
/// "빨리 빠지면 더 깊이 가나"를 사례 전부로(Cleveland & McGill: 같은 축 위치 비교). marks: 이름표 달 고점 해
pub fn speed(rows: &[(String, f64, f64, bool)], deep: f64, marks: &[String], title: &str) -> String {
    let (l, r, top) = (52.0, 18.0, T + 26.0);
    let h = H + 40.0;
    let bottom = h - B - 18.0;
    let xmax = (rows.iter().map(|x| x.1).fold(30.0, f64::max) / 60.0).ceil() * 60.0;
    let ymin = (rows.iter().map(|x| x.2).fold(-deep, f64::min) * 10.0).floor() / 10.0;
    let x = |v: f64| l + v / xmax * (W - l - r);
    let y = |v: f64| top + v / ymin * (bottom - top);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{INK}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Became a bear market</text><circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{GRAY}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Stopped short</text>",
                   l, T + 6.0, l + 10.0, T + 10.0, l + 190.0, T + 6.0, l + 200.0, T + 10.0);
    let mut v = 0.0;
    while v >= ymin - 1e-9 {
        let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       W - r, y(v), y(v), l - 6.0, y(v) + 4.0, pct(v));
        v -= 0.1;
    }
    let mut v = 0.0;
    while v <= xmax + 1e-9 {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>", x(v), bottom + 16.0, v);
        v += 60.0;
    }
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">Days from the record to the first 10% close</text>", (l + W - r) / 2.0, bottom + 34.0);
    let _ = write!(s, "<text transform=\"translate(14 {:.1}) rotate(-90)\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">How far it fell in the end</text>", (top + bottom) / 2.0);
    let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#ea580c\" stroke-width=\"2\" stroke-dasharray=\"5 4\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"#c2410c\" font-size=\"13\">Bear market line</text>",
                   W - r, y(-deep), y(-deep), W - r, y(-deep) - 5.0);
    for (name, d, low, open) in rows {
        let (fill, rad) = if *open { (BLUE, 6.0) } else if *low <= -deep { (INK, 5.0) } else { (GRAY, 4.0) };
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{rad}\" fill=\"{fill}\" stroke=\"#fff\" stroke-width=\"1.5\"/>", x(*d), y(*low));
        if *open || marks.iter().any(|m| name.starts_with(m.as_str())) {
            let lab = if *open { format!("{} so far", &name[..4.min(name.len())]) } else { name[..4.min(name.len())].to_string() };
            let right = x(*d) < W - r - 90.0;
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"{}\" fill=\"{}\" font-size=\"13\" font-weight=\"600\">{lab}</text>",
                           if right { x(*d) + 9.0 } else { x(*d) - 9.0 }, y(*low) + 4.0, if right { "start" } else { "end" }, if *open { BLUE } else { INK });
        }
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod speed_test {
    #[test]
    fn speed_marks_open_and_deep() {
        let rows = vec![("2020-02-19".to_string(), 8.0, -0.339, false), ("2015-05-21".to_string(), 95.0, -0.142, false), ("2026-10-06".to_string(), 45.0, -0.116, true)];
        let svg = super::speed(&rows, 0.20, &["2020".to_string()], "t");
        assert!(svg.contains(">2020<") && svg.contains("2026 so far") && !svg.contains(">2015<") && svg.contains("Bear market line"));
    }
}

/// 이자 대 가격(해마다 막대 둘): 파랑 = 그해 번 이자(0에서 위로), 주황 = 가격 하락(0에서 아래로), 회색 = 가격 상승(이자 위에 쌓음),
/// 검은 눈금 = 그해 총수익(둘의 합), 0선 아래 눈금이 손실 난 해. 독자는 눈금을 읽는다(2026-11-23 콜드 리드). 폭포(이자 꼭대기에서 깎기)는 주황이 파랑을 가려 버렸다: "이자와 가격 중 무엇이 이겼나"를 해마다 길이로(Cleveland & McGill).
/// rows: (해, 이자 몫, 가격 몫, 총수익, 해가 다 찼나). marks: 총수익을 이름표로 달 해. 다 차지 않은 해는 'so far'.
pub fn split(rows: &[(i32, f64, f64, f64, bool)], marks: &[String], title: &str) -> String {
    const ORANGE: &str = "#eb6834";
    let (l, r, top) = (40.0, 12.0, T + 30.0);
    let h = H + 30.0;
    let bottom = h - B;
    let hi = rows.iter().map(|x| x.1 + x.2.max(0.0)).fold(0.05, f64::max) * 100.0;
    let lo = rows.iter().map(|x| x.2.min(0.0)).fold(-0.05, f64::min) * 100.0;
    let step = if hi - lo > 40.0 { 10.0 } else { 5.0 };
    let (lo, hi) = ((lo / step).floor() * step, (hi / step).ceil() * step);
    let n = rows.len() as f64;
    let bw = (W - l - r) / n;
    let x = |i: usize| l + i as f64 * bw;
    let y = |v: f64| top + (hi - v * 100.0) / (hi - lo) * (bottom - top);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    // 범례(색만으로 뜻을 말하지 않게 글자로)
    let items = [(BLUE, "Interest earned"), (ORANGE, "Price drop"), (GRAY, "Price gain")];
    let mut lx = l;
    for (c, name) in items {
        let _ = write!(s, "<rect x=\"{lx:.1}\" y=\"6\" width=\"11\" height=\"11\" rx=\"2\" fill=\"{c}\"/><text x=\"{:.1}\" y=\"16\" fill=\"{INK}\" font-size=\"13\">{name}</text>", lx + 15.0);
        lx += 22.0 + name.len() as f64 * 7.0;
    }
    let _ = write!(s, "<line x1=\"{lx:.1}\" x2=\"{:.1}\" y1=\"11.5\" y2=\"11.5\" stroke=\"#0f172a\" stroke-width=\"2.5\"/><text x=\"{:.1}\" y=\"16\" fill=\"{INK}\" font-size=\"13\">Year's return</text>", lx + 12.0, lx + 16.0);
    let mut v = lo;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}{v}%</text>",
                       W - r, y(v / 100.0), y(v / 100.0), if v == 0.0 { INK } else { "#eef2f7" }, l - 5.0, y(v / 100.0) + 4.0, if v > 0.0 { "+" } else { "" });
        v += step;
    }
    for (i, (_, inc, pr, ret, _)) in rows.iter().enumerate() {
        let (bx, w) = (x(i) + 0.6, (bw - 1.2).max(1.0));
        let _ = write!(s, "<rect x=\"{bx:.1}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"{:.1}\" fill=\"{BLUE}\"/>", y(*inc), y(0.0) - y(*inc));
        if *pr < 0.0 {
            let _ = write!(s, "<rect x=\"{bx:.1}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"{:.1}\" fill=\"{ORANGE}\"/>", y(0.0), y(*pr) - y(0.0));
        } else {
            let _ = write!(s, "<rect x=\"{bx:.1}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"{:.1}\" fill=\"{GRAY}\"/>", y(inc + pr), y(*inc) - y(inc + pr));
        }
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#0f172a\" stroke-width=\"2.5\"/>", x(i) - 0.5, x(i) + bw + 0.5, y(*ret), y(*ret));
    }
    for (i, (yr, inc, pr, ret, full)) in rows.iter().enumerate() {
        if !(!full || marks.iter().any(|m| m == &yr.to_string())) { continue; }
        let lab = format!("{yr}{} {}{}", if *full { "" } else { " so far" }, if *ret > 0.0 { "+" } else { "" }, pct(*ret));
        // 다 차지 않은 해(오른쪽 끝)는 이름표가 왼쪽 막대들 위로 뻗으니, 가까운 16해 막대 꼭대기보다 위에 둔다
        let near = rows[i.saturating_sub(16)..=i].iter().map(|x| x.1 + x.2.max(0.0)).fold(0.0, f64::max);
        let ly = if !full { y(near + 0.03) } else if *ret < 0.0 { y(pr.min(0.0)) + 15.0 } else { y(inc + pr.max(0.0)) - 6.0 }; // 이익 난 해는 막대 위, 손실 난 해는 주황 아래(2026-11-23: 주황 아래 '+7%'가 손실로 읽혔다)
        let anchor = if x(i) > W - 90.0 { "end" } else if x(i) < l + 40.0 { "start" } else { "middle" };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{ly:.1}\" text-anchor=\"{anchor}\" fill=\"{}\" font-size=\"13\" font-weight=\"600\" stroke=\"#fff\" stroke-width=\"4\" paint-order=\"stroke\">{lab}</text>",
                       x(i) + bw / 2.0, if *full { "#0f172a" } else { BLUE });
    }
    // 해마다 막대가 좁아 휴대폰에서 연도를 찾기 어렵다(2026-11-23 콜드 리드): 처음·끝 대신 10년마다 눈금과 연도
    for (i, (yr, ..)) in rows.iter().enumerate() {
        if yr % 10 != 0 { continue; }
        let cx = x(i) + bw / 2.0;
        let _ = write!(s, "<line x1=\"{cx:.1}\" x2=\"{cx:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\"/><text x=\"{cx:.1}\" y=\"{}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{yr}</text>", bottom, bottom + 4.0, h - 6.0);
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod split_test {
    #[test]
    fn split_draws_parts_and_labels() {
        let rows = vec![(2021, 0.01, -0.05, -0.04, true), (2020, 0.015, 0.05, 0.065, true), (2026, 0.035, -0.054, -0.019, false)];
        let svg = super::split(&rows, &["2021".to_string()], "t");
        assert!(svg.contains("2021 -4%") && svg.contains("2026 so far -2%") && !svg.contains(">2020 "));
        assert!(super::split(&rows, &["2020".to_string()], "t").contains("2020 +7%"));
        assert_eq!(svg.matches("#eb6834").count(), 3); // 범례 1 + 하락 막대 2
    }
}

/// 받쳐 주기 그림: 주식이 기록에서 크게 빠진 하락마다 한 줄(왼쪽 이름 = 해 · 주식 하락폭), 같은 날짜 사이 채권 펀드 총수익을 0에서 뻗는 막대로.
/// 막대 색은 그 사이 금리 방향(내림 파랑, 오름 주황, 범례 글자로도). "주식이 빠질 때 채권은 오르나"와 "언제 아닌가"를 한 장에(2026-11-24 글)
pub fn catch(rows: &[(String, f64, f64, bool)], title: &str) -> String {
    const ORANGE: &str = "#eb6834";
    let (row, top) = (26.0, T + 44.0);
    let h = top + rows.len() as f64 * row + B + 6.0;
    let l = rows.iter().map(|r| r.0.chars().count()).max().map(|n| n as f64 * 7.4 + 12.0).unwrap_or(90.0).max(90.0);
    let r = 50.0;
    let lo = (rows.iter().map(|x| x.1).fold(0.0, f64::min) * 20.0).floor() / 20.0;  // 손실 쪽은 5% 단위로(10% 단위면 왼쪽이 비었다)
    let hi = (rows.iter().map(|x| x.1).fold(0.0, f64::max) * 10.0).ceil() / 10.0;
    let lo = if lo < 0.0 { lo.min(-0.05) } else { 0.0 };
    let x = |v: f64| l + (v - lo) / (hi - lo).max(1e-9) * (W - l - r);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    // 범례: 색만으로 뜻을 말하지 않게 글자로
    let mut lx = 4.0;
    for (c, name) in [(BLUE, "Yields fell"), (ORANGE, "Yields rose")] {
        let _ = write!(s, "<rect x=\"{lx:.1}\" y=\"6\" width=\"11\" height=\"11\" rx=\"2\" fill=\"{c}\"/><text x=\"{:.1}\" y=\"16\" fill=\"{INK}\" font-size=\"13\">{name}</text>", lx + 15.0);
        lx += 30.0 + name.len() as f64 * 7.0;
    }
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"12\">Stock fall</text><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"12\">Bond fund, same days</text>", l - 8.0, top - 10.0, x(0.0) + 4.0, top - 10.0);
    let mut v = (lo * 10.0).ceil() / 10.0;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"12\">{}{}</text>",
                       x(v), x(v), top - 4.0, h - B, if v.abs() < 1e-9 { INK } else { "#eef2f7" }, x(v), h - 8.0, if v > 1e-9 { "+" } else { "" }, pct(v));
        v += 0.1;
    }
    for (k, (name, b, dy, open)) in rows.iter().enumerate() {
        let y = top + k as f64 * row + 5.0;
        let c = if *dy < 0.0 { BLUE } else { ORANGE };
        let (x0, x1) = (x(b.min(0.0)), x(b.max(0.0)));
        let val = format!("{}{:.1}%", if *b > 0.0 { "+" } else if *b < 0.0 { "-" } else { "" }, (b * 100.0).abs());
        // 손실 값이 왼쪽 이름과 부딪히면 0선 오른쪽에 적는다(2022 · -25% 옆 -12.8%가 붙어 읽혔다)
        let (tx, anchor) = if *b >= 0.0 || x0 - 5.0 - val.len() as f64 * 7.0 < l { (x1.max(x(0.0)) + 5.0, "start") } else { (x0 - 5.0, "end") };
        let w = if *open { " font-weight=\"600\"" } else { "" };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\"{w}>{}</text><rect x=\"{x0:.1}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"14\" rx=\"2\" fill=\"{c}\"/><text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{INK}\" font-size=\"12\"{w}>{val}</text>",
                       l - 8.0, y + 12.0, name.replace('&', "&amp;"), (x1 - x0).max(1.5), y + 11.0);
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod catch_test {
    #[test]
    fn catch_colors_by_yield_direction() {
        let rows = vec![("2008–09 · -57%".to_string(), 0.177, -1.96, false), ("2022 · -25%".to_string(), -0.128, 2.48, false), ("2026 · -12% so far".to_string(), 0.018, -0.21, true)];
        let svg = super::catch(&rows, "t");
        assert!(svg.contains(">+17.7%<") && svg.contains(">-12.8%<") && svg.contains("fill=\"#eb6834\"") && svg.contains("Yields rose") && svg.contains("so far"));
    }
}

/// 반등 작은 그림들: 사례마다 한 칸(두 줄씩), 반등 달 앞뒤 months달의 주 계열(파랑)과 also 계열(주황 점선), 반등 달 세로선.
/// 칸 제목은 해와 'also가 반등 뒤 6달 동안 바뀐 폭'(없으면 'core n/a'). 모든 칸이 같은 세로 눈금이라 크기를 견줄 수 있다.
/// cases: (이름, [(반등 대비 달, 주 값)], [(달, also 값)], 제목 꼬리)
pub fn rebound_grid(cases: &[(String, Vec<(f64, f64)>, Vec<(f64, f64)>, String)], months: f64, names: (&str, &str), unit: &str, title: &str) -> String {
    let cols = 2usize;
    let rows = (cases.len() + cols - 1) / cols;
    let (pw, ph, gap_y, top) = (W / cols as f64, 118.0, 10.0, 30.0);
    let hh = top + rows as f64 * (ph + gap_y) + 4.0;
    let all: Vec<f64> = cases.iter().flat_map(|c| c.1.iter().chain(c.2.iter()).map(|q| q.1)).collect();
    let lo = all.iter().cloned().fold(0.0_f64, f64::min).floor();
    let hi = all.iter().cloned().fold(0.0_f64, f64::max).ceil();
    let mut s = format!("<svg viewBox=\"0 0 {W} {hh}\" width=\"{W}\" height=\"{hh}\" style=\"width:100%;max-width:560px;height:auto\" role=\"img\" xmlns=\"http://www.w3.org/2000/svg\" font-family=\"system-ui,sans-serif\" font-size=\"16\"><title>{}</title>", title.replace('&', "&amp;").replace('<', "&lt;"));
    // 범례(맨 위)
    let _ = write!(s, "<line x1=\"8\" x2=\"30\" y1=\"14\" y2=\"14\" stroke=\"{BLUE}\" stroke-width=\"2.5\"/><text x=\"35\" y=\"18\" fill=\"{INK}\" font-size=\"13\">{}</text><line x1=\"190\" x2=\"212\" y1=\"14\" y2=\"14\" stroke=\"#eb6834\" stroke-width=\"2\" stroke-dasharray=\"4 3\"/><text x=\"217\" y=\"18\" fill=\"{INK}\" font-size=\"13\">{}</text>",
        names.0.replace('&', "&amp;"), names.1.replace('&', "&amp;"));
    let step = if hi - lo > 8.0 { 4.0 } else { 2.0 };
    for (k, (name, a, b, tail)) in cases.iter().enumerate() {
        let (cx, cy) = ((k % cols) as f64 * pw, top + (k / cols) as f64 * (ph + gap_y));
        let (l, r, t, bt) = (cx + 34.0, cx + pw - 8.0, cy + 20.0, cy + ph - 16.0);
        let x = |m: f64| l + (m + months) / (2.0 * months) * (r - l);
        let y = |v: f64| t + (hi - v) / (hi - lo) * (bt - t);
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\"><tspan font-weight=\"700\">{}</tspan> {}</text>", cx + 6.0, cy + 12.0, name, tail.replace('&', "&amp;"));
        let mut v = (lo / step).ceil() * step + 0.0;
        while v <= hi + 1e-9 {
            let _ = write!(s, "<line x1=\"{l:.1}\" x2=\"{r:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"11\">{v:.0}{unit}</text>",
                y(v), y(v), if v.abs() < 1e-9 { "#cbd5e1" } else { "#eef2f7" }, l - 3.0, y(v) + 4.0);
            v += step;
            if v.abs() < 1e-9 { v = 0.0; } // -0% 대신 0%
        }
        let x0 = x(0.0);
        let _ = write!(s, "<line x1=\"{x0:.1}\" x2=\"{x0:.1}\" y1=\"{t:.1}\" y2=\"{bt:.1}\" stroke=\"{INK}\" stroke-dasharray=\"3 3\"/>");
        for (m, lab) in [(-months, format!("-{}y", months / 12.0)), (0.0, "0".to_string()), (months, format!("+{}y", months / 12.0))] {
            let anchor = if m < 0.0 { "start" } else if m > 0.0 { "end" } else { "middle" };
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{INK}\" font-size=\"11\">{lab}</text>", x(m), bt + 13.0);
        }
        let line = |p: &[(f64, f64)]| p.iter().map(|q| format!("{:.1},{:.1}", x(q.0), y(q.1))).collect::<Vec<_>>().join(" ");
        if !b.is_empty() { let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"#eb6834\" stroke-width=\"1.8\" stroke-dasharray=\"4 3\"/>", line(b)); }
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{BLUE}\" stroke-width=\"2.2\"/>", line(a));
        if let Some(last) = a.last() { let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"2.8\" fill=\"{BLUE}\"/>", x(last.0), y(last.1)); }
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod rebound_grid_tests {
    #[test]
    fn grid_has_one_panel_per_case() {
        let c = vec![("1973".to_string(), vec![(-1.0, 3.0), (0.0, 4.0), (1.0, 6.0)], vec![(0.0, 3.0)], "core +0.6 pt in 6 mo".to_string()),
                     ("2026".to_string(), vec![(0.0, 3.8)], vec![], "no core data".to_string())];
        let s = super::rebound_grid(&c, 24.0, ("Headline", "Core"), "%", "t");
        assert!(s.starts_with("<svg") && s.ends_with("</svg>"));
        assert_eq!(s.matches("font-weight=\"700\"").count(), 2);
        assert!(s.contains("core +0.6 pt in 6 mo") && s.contains("stroke-dasharray=\"4 3\""));
    }
}

/// 화살표 줄: 줄마다 시작 값 → 끝 값 화살표. 내려가면 파랑, 오르면 주황, 열린 줄(--open, 아직 끝나지 않은 사례)은 회색 점선.
/// 끝에 값 글자. 눈금은 수준값 %(부호 없음). 한 줄에 점 셋을 겹치던 그림을 대신한다(물가 반등, 2026-11-26 독자: 색 셋을 읽어야 결론이 보였다)
pub fn arrows(rows: &[(String, f64, f64)], open: &[String], names: (&str, &str), title: &str) -> String {
    let all: Vec<f64> = rows.iter().flat_map(|r| [r.1, r.2]).collect();
    let lo = all.iter().cloned().fold(0.0_f64, f64::min) - 0.01;
    let hi = all.iter().cloned().fold(0.0_f64, f64::max) + 0.015;
    let (l, r, t, b) = (118.0, 30.0, T + 8.0, 52.0);
    let x = |v: f64| l + (v - lo) / (hi - lo).max(1e-9) * (W - l - r);
    let rh = (H - t - b) / rows.len().max(1) as f64;
    let mut s = head(title);
    s.push_str("<defs><marker id=\"ah-b\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"5\" markerHeight=\"5\" orient=\"auto\"><path d=\"M0,0L10,5L0,10z\" fill=\"#2563eb\"/></marker><marker id=\"ah-o\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"5\" markerHeight=\"5\" orient=\"auto\"><path d=\"M0,0L10,5L0,10z\" fill=\"#eb6834\"/></marker><marker id=\"ah-g\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"5\" markerHeight=\"5\" orient=\"auto\"><path d=\"M0,0L10,5L0,10z\" fill=\"#94a3b8\"/></marker></defs>");
    let step = [0.02, 0.05].iter().cloned().find(|st| (hi - lo) / st <= 8.0).unwrap_or(0.05);
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{t}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"12\">{:.0}%</text>",
            x(v), x(v), H - b, if v.abs() < 1e-9 { INK } else { "#eef2f7" }, x(v), H - b + 14.0, (v * 100.0).round() + 0.0);
        v += step;
    }
    for (i, (name, a, z)) in rows.iter().enumerate() {
        let y = t + rh * (i as f64 + 0.5);
        let is_open = open.iter().any(|o| o == name);
        let (col, mk) = if is_open { ("#94a3b8", "ah-g") } else if z < a { ("#2563eb", "ah-b") } else { ("#eb6834", "ah-o") };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text>", l - 8.0, y + 4.0, name.replace('&', "&amp;"));
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"3.5\" fill=\"{col}\"/>", x(*a));
        // 아주 짧은 화살표(거의 그대로)는 화살촉만 보이게 최소 길이를 둔다
        let dir = if z >= a { 1.0 } else { -1.0 };
        let x2 = if (x(*z) - x(*a)).abs() < 8.0 { x(*a) + dir * 8.0 } else { x(*z) };
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{y:.1}\" y2=\"{y:.1}\" stroke=\"{col}\" stroke-width=\"2.5\"{} marker-end=\"url(#{mk})\"/>", x(*a), x2, if is_open { " stroke-dasharray=\"4 3\"" } else { "" });
        let (tx, anchor) = if dir > 0.0 { (x2 + 8.0, "start") } else { (x2 - 8.0, "end") };
        let _ = write!(s, "<text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{INK}\" font-size=\"12\">{:.1}%</text>", y + 4.0, z * 100.0);
    }
    let ly = H - 10.0;
    let _ = write!(s, "<line x1=\"{l}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#2563eb\" stroke-width=\"2.5\" marker-end=\"url(#ah-b)\"/><text x=\"{:.1}\" y=\"{ly:.1}\" fill=\"{INK}\" font-size=\"13\">{}</text><line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eb6834\" stroke-width=\"2.5\" marker-end=\"url(#ah-o)\"/><text x=\"{:.1}\" y=\"{ly:.1}\" fill=\"{INK}\" font-size=\"13\">{}</text>",
        l + 18.0, ly - 4.0, ly - 4.0, l + 24.0, names.0.replace('&', "&amp;"), l + 150.0, l + 168.0, ly - 4.0, ly - 4.0, l + 174.0, names.1.replace('&', "&amp;"));
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod arrows_tests {
    #[test]
    fn color_by_direction() {
        let r = vec![("a".to_string(), 0.03, 0.01), ("b".to_string(), 0.04, 0.10), ("c".to_string(), 0.038, 0.039)];
        let s = super::arrows(&r, &["c".to_string()], ("Lower", "Higher"), "t");
        assert!(s.contains("url(#ah-b)") && s.contains("url(#ah-o)") && s.contains("url(#ah-g)"));
        assert!(s.contains("10.0%") && s.contains("stroke-dasharray=\"4 3\""));
    }
}

#[cfg(test)]
mod val_label_cents_test {
    #[test]
    fn small_dollars_show_cents() {
        assert_eq!(super::val_label(3.642, "$", 0), "$3.64");
        assert_eq!(super::val_label(3901.2, "$", 0), "$3,901");
    }
}
