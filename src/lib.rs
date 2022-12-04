#![feature(portable_simd)]
#![feature(int_roundings)]
#![feature(iter_array_chunks)]

use std::simd::Simd;

type SimdT = u32;
// u32 最多可以有 ≈ 9 位數，但 SIMD 只支援 8 個管道
// 這部份我們之後會進行拆分。
const SIMD_LANE_LEN: usize = 8;

const SIMD_TIMES_TABLE: Simd<SimdT, SIMD_LANE_LEN> = Simd::from_array(create_times_table());
const SIMD_CTOI_TABLE: Simd<SimdT, SIMD_LANE_LEN> = Simd::from_array([0x30; SIMD_LANE_LEN]);


/// 將文字轉成數字
pub fn atoi(s: &str) -> SimdT {
    let v = s.as_bytes();
    let mut result = Simd::<SimdT, SIMD_LANE_LEN>::splat(0);

    for (index, vector) in bytes_to_vectors(v).into_iter().enumerate() {
        let base = MAX_BLOCKS - index - 1;
        let times = 10u32
            .checked_pow((base * SIMD_LANE_LEN) as u32)
            .unwrap_or(0);
        let mut simd_vector = Simd::from_array(vector);

        // 對每個 binary 乘以 0x30
        simd_vector ^= SIMD_CTOI_TABLE;
        // 然後根據 TIMES_TABLE 乘上 10 倍數
        simd_vector *= SIMD_TIMES_TABLE;
        // 假如是 [[0, ..., 1], [2, ..., 9]]，則前者應乘以 10^8，後者應乘以 10^0
        simd_vector *= Simd::splat(times);

        result += simd_vector;
    }

    let a = result.to_array();
    a.iter().sum()
}

/// 將輸入的片段 char code 轉成符合 `SIMD_LANE_LEN` 長度的向量
///
/// 假設輸入 `1, 2, 3`，這段命令會把它轉成 `00000123`。
/// 得到的結果可以直接與 TimesTable 相乘，最後 .sum()
/// 即可得到結果。
pub fn bytes_to_vector(s: &[u8]) -> [SimdT; SIMD_LANE_LEN] {
    let mut result = ['0' as SimdT; SIMD_LANE_LEN];

    assert!(s.len() <= 8);

    for (i, c) in s.iter().enumerate() {
        result[(SIMD_LANE_LEN - s.len() + i)] = *c as SimdT;
    }

    result
}

const MAX_LANES: usize = 64;
const MAX_BLOCKS: usize = MAX_LANES / SIMD_LANE_LEN;

/// 將輸入的 v 轉成數個 Vectors
///
/// 比如有 9 個 v 且 `SIMD_LANE_LEN == 8`，
/// 則會切成 `[(第 1 個元素), (後 8 個元素)]`
///
/// 為了不要在 heap 分配，實際上我們回傳的是一個固定長度的陣列。
/// 長度是根據 `MAX_LANES` 推算的，而預設元素是一個無效果的空陣列 (`[0; SIMD_LANE_LEN]`)
/// `foreach` 之後 `sum` 不會影響最終結果～
pub fn bytes_to_vectors(v: &[u8]) -> [[SimdT; SIMD_LANE_LEN]; MAX_BLOCKS] {
    let mut result = [['0' as SimdT; SIMD_LANE_LEN]; MAX_BLOCKS];

    // FIXME: more meaningful spliting
    v.rchunks(SIMD_LANE_LEN).enumerate().for_each(|(i, v)| {
        result[SIMD_LANE_LEN - i - 1] = bytes_to_vector(v);
    });

    result
}

/// 建立乘法表。
///
/// 它會產生出像這樣的結果：
///
///     [1000_0000, 100_0000, 10_0000, 1_0000,
///           1000,      100,      10,      1]
///
/// 乘以 `bytes_to_vector()` 的值即可得到可以直接加總的答案。
const fn create_times_table<const LEN: usize>() -> [SimdT; LEN] {
    let mut t = [10 as SimdT; LEN];
    let mut p = 0;

    while p < LEN {
        t[p] = t[p].pow((LEN - p - 1) as u32);
        p += 1;
    }

    t
}

#[cfg(test)]
mod tests {
    use std::simd::Simd;

    use crate::{
        create_times_table, SimdT, MAX_LANES, SIMD_CTOI_TABLE, SIMD_LANE_LEN, SIMD_TIMES_TABLE,
    };

    #[test]
    fn test_create_times_table() {
        assert_eq!(
            create_times_table::<8>(),
            [1000_0000, 100_0000, 10_0000, 1_0000, 1000, 100, 10, 1]
        );
    }

    #[test]
    fn test_bytes_to_vector() {
        // ASCII('0') = 48
        assert_eq!(crate::bytes_to_vector(b"123"), [48, 48, 48, 48, 48, 49, 50, 51]);
    }

    #[test]
    fn test_bytes_to_vectors() {
        let td = {
            let mut td = [['0' as SimdT; SIMD_LANE_LEN]; MAX_LANES / SIMD_LANE_LEN];

            td[6] = [48, 48, 48, 48, 48, 48, 48, 49];
            td[7] = [50, 51, 52, 53, 54, 55, 56, 57];

            td
        };

        assert_eq!(crate::bytes_to_vectors(b"123456789"), td);
    }

    #[test]
    fn test_atoi() {
        assert_eq!(crate::atoi("123456789"), 123456789);
        assert_eq!(crate::atoi("567"), 567);
        assert_eq!(crate::atoi("114514"), 114514);
        assert_eq!(crate::atoi("100002"), 100002);
        assert_eq!(crate::atoi("000004"), 4);
    }

    #[test]
    fn test_times_table() {
        let t = Simd::from_array([1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            (t * SIMD_TIMES_TABLE).to_array(),
            [10000000, 2000000, 300000, 40000, 5000, 600, 70, 8]
        );

        let t = Simd::from_array([0, 0, 0, 0, 0, 1, 2, 3]);
        assert_eq!(
            (t * SIMD_TIMES_TABLE).to_array(),
            [0, 0, 0, 0, 0, 100, 20, 3]
        );
    }

    #[test]
    fn test_ctoi_table() {
        let t = Simd::from_array([b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7'].map(u32::from));
        assert_eq!((t ^ SIMD_CTOI_TABLE).to_array(), [0, 1, 2, 3, 4, 5, 6, 7]);

        let t = Simd::from_array([b'0', b'0', b'0', b'0', b'0', b'1', b'2', b'3'].map(u32::from));
        assert_eq!((t ^ SIMD_CTOI_TABLE).to_array(), [0, 0, 0, 0, 0, 1, 2, 3]);
    }
}
