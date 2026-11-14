//! 채권 펀드 손익분기: 금리가 jump만큼 오르면 값이 바로 약 듀레이션 × jump만큼 떨어지고(볼록성 반영),
//! 그 뒤로는 더 높은 수익률로 불어난다. 금리가 그대로였을 경로와 만나는 해가 손익분기다(대략 듀레이션 근처).
//! 단순화: 펀드 수익률은 시장 금리 변화를 바로 따르고, 이후 금리는 그대로라고 본다.

/// 금리 상승 직후 값의 변화(비율). 수정 듀레이션 d, 볼록성 c(없으면 d²/2 근사 대신 0), 상승폭 dy(0.01 = 1%p)
pub fn jump_loss(d: f64, c: f64, dy: f64) -> f64 {
    -d * dy + 0.5 * c * dy * dy
}

/// t년 뒤 두 경로의 값(시작 1): (금리 그대로, 금리 상승). y0는 연 수익률(0.045 = 4.5%)
pub fn values(d: f64, c: f64, y0: f64, dy: f64, t: f64) -> (f64, f64) {
    ((1.0 + y0).powf(t), (1.0 + jump_loss(d, c, dy)) * (1.0 + y0 + dy).powf(t))
}

/// 손익분기(년): 상승 경로가 그대로 경로를 따라잡는 때
pub fn breakeven(d: f64, c: f64, y0: f64, dy: f64) -> Option<f64> {
    let k = 1.0 + jump_loss(d, c, dy);
    if dy <= 0.0 || k <= 0.0 { return None; }
    Some(-(k.ln()) / ((1.0 + y0 + dy).ln() - (1.0 + y0).ln()))
}

/// 만기 고정 채권 펀드의 총수익 지수(시작 100): 금리 계열(%)의 달마다 마지막 값으로, 매달 그 금리를 쿠폰으로 하는
/// m년 만기 액면 채권을 사서 한 달 뒤 금리로 다시 값을 매기고(반년 복리) 한 달 이자를 더한다(Swinkels 2019의 방식).
/// 수수료·세금은 없다. 1962년부터 있는 국채 금리로 '채권 펀드가 크게 빠진 때와 회복에 걸린 시간'을 잴 수 있다.
pub fn total_return(pts: &[crate::series::Point], m: f64) -> Vec<crate::series::Point> {
    let mut ends: Vec<&crate::series::Point> = Vec::new();
    for p in pts {
        match ends.last() {
            Some(l) if l.date[..7] == p.date[..7] => { *ends.last_mut().unwrap() = p; }
            _ => ends.push(p),
        }
    }
    let mut out = Vec::with_capacity(ends.len());
    let mut idx = 100.0;
    for (k, p) in ends.iter().enumerate() {
        if k > 0 {
            let (y0, y1) = (ends[k - 1].value / 100.0, p.value / 100.0);
            let n = 2.0 * (m - 1.0 / 12.0);
            let v = (1.0 + y1 / 2.0).powf(-n);
            let price = if y1.abs() < 1e-12 { y0 / 2.0 * n + 1.0 } else { y0 / y1 * (1.0 - v) + v };
            idx *= price + y0 / 12.0;
        }
        out.push(crate::series::Point { date: p.date.clone(), day: p.day, value: idx });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakeven_near_duration() {
        // 듀레이션 6년, 4.5% → 5.5%: 바로 약 6% 손실, 손익분기는 약 6년
        let l = jump_loss(6.0, 0.0, 0.01);
        assert!((l + 0.06).abs() < 1e-9);
        let b = breakeven(6.0, 0.0, 0.045, 0.01).unwrap();
        assert!(b > 5.5 && b < 7.0, "{b}");
        let (a, j) = values(6.0, 0.0, 0.045, 0.01, b);
        assert!((a - j).abs() < 1e-9);
    }

    #[test]
    fn total_return_flat_and_jump() {
        use crate::series::Point;
        let p = |d: &str, v: f64| Point { date: d.into(), day: crate::series::day_number(d).unwrap(), value: v };
        // 금리가 그대로면 한 달에 이자만큼(5%/12) 는다
        let flat = total_return(&[p("2020-01-31", 5.0), p("2020-02-29", 5.0)], 7.0);
        assert!((flat[1].value / flat[0].value - (1.0 + 0.05 / 12.0)).abs() < 1e-9);
        // 7년 만기에서 금리가 1%p 오르면 약 듀레이션(6년 안팎)만큼 빠진다
        let up = total_return(&[p("2020-01-31", 5.0), p("2020-02-15", 5.5), p("2020-02-29", 6.0)], 7.0);
        assert_eq!(up.len(), 2);
        let r = up[1].value / up[0].value - 1.0;
        assert!(r < -0.05 && r > -0.065, "{r}");
    }
}
