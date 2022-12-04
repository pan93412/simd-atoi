use std::ffi::{c_char, c_int, CString};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

extern "C" {
    fn atoi(s: *const c_char) -> c_int;
}

fn leetcode_0ms_atoi(s: &str) -> i32 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }
    let mut push = String::new();
    let mut chars = s.chars();
    let symbol = &s[0..1] != "-";

    if !symbol || &s[0..1] == "+" {
        chars.next();
    }

    let mut started = false;

    for c in chars {
        if !started && c == '0' {
        } else if c.is_numeric() {
            started = true;
            push.push(c);
        } else {
            break;
        }
    }

    let parsed = push.parse::<i128>().unwrap_or_else(|_| {
        if s.len() > 36 {
            if symbol {
                i32::MAX as i128
            } else {
                i32::MIN as i128
            }
        } else {
            0
        }
    });

    if parsed > i32::MAX as i128 {
        if symbol {
            i32::MAX
        } else {
            i32::MIN
        }
    } else if !symbol {
        -(parsed as i32)
    } else {
        parsed as i32
    }
}

fn leetcode_3ms_atoi(s: &str) -> i32 {
    let s = s.trim();

    let mut positive = true;
    let mut break_non_digit = false;
    let mut rv: u32 = 0;

    for c in s.bytes() {
        match c {
            b'+' => {
                if !break_non_digit {
                    break_non_digit = true;
                } else {
                    break;
                }
            }
            b'-' => {
                if !break_non_digit {
                    positive = false;
                    break_non_digit = true;
                } else {
                    break;
                }
            }
            b'0'..=b'9' => {
                break_non_digit = true;

                if c == b'0' && rv == 0 {
                    continue;
                }

                let digit = c as u32 - 48;

                if rv > (<u32>::MAX - digit) / 10 {
                    if positive {
                        return <i32>::MAX;
                    }

                    return <i32>::MIN;
                }

                rv = rv * 10 + digit;

                if (positive && rv >= <i32>::MAX as u32)
                    || (!positive && rv >= <i32>::MAX as u32 + 1)
                {
                    if positive {
                        return <i32>::MAX;
                    }

                    return <i32>::MIN;
                }
            }
            _ => break,
        }
    }

    if !positive {
        return -(rv as i32);
    }

    rv as i32
}

fn leetcode_8ms_atoi(s: &str) -> i32 {
    const BLANK: u8 = b' ';
    const MINUS: u8 = b'-';
    const PLUS: u8 = b'+';
    const DIGITS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
    // EXPONENTS[0] = 10^0
    // EXPONENTS[1] = 10^1
    // EXPONENTS[2] = 10^2
    // And so on...
    const EXPONENTS: [i32; 10] = [
        1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
    ];

    fn my_atoi(s: &str) -> i32 {
        let bytes = s.as_bytes();
        let begin_index = bytes.iter().position(|c| *c != BLANK);
        if begin_index.is_none() {
            return 0;
        }

        let mut begin_index = begin_index.unwrap();
        let is_sign_character = is_sign(bytes[begin_index]);
        let mut is_positive = true;
        if is_sign_character {
            is_positive = PLUS == bytes[begin_index];
            begin_index += 1;
        }

        if begin_index == bytes.len() {
            return 0;
        }

        let begin_index = first_non_zero_digit_index(bytes, begin_index);
        let begin_index = match begin_index {
            Some(begin_index) => begin_index,
            None => bytes.len(),
        };

        if begin_index == bytes.len() {
            return 0;
        }

        let end_index = first_non_digit_index(bytes, begin_index);
        let end_index = match end_index {
            Some(end_index) => end_index,
            None => bytes.len(),
        };

        let digit_count = end_index - begin_index;
        if digit_count == 0 {
            return 0;
        }

        let mut exponent = digit_count - 1;
        let mut result: i32 = 0;
        for digit in bytes.iter().take(end_index).skip(begin_index) {
            if exponent >= EXPONENTS.len() {
                return out_of_range(is_positive);
            }

            let power_result = EXPONENTS[exponent];
            let digit = digit_to_integer(*digit).unwrap() as i32;
            let product_result = power_result.overflowing_mul(digit);
            if product_result.1 {
                return out_of_range(is_positive);
            }

            let addition_result = result.overflowing_add(product_result.0);
            if addition_result.1 {
                return out_of_range(is_positive);
            }

            result = addition_result.0;
            if exponent > 0 {
                exponent -= 1;
            }
        }

        if !is_positive {
            let product_result = result.overflowing_mul(-1);
            if product_result.1 {
                return out_of_range(is_positive);
            }

            result = product_result.0;
        }

        result
    }

    fn is_sign(c: u8) -> bool {
        matches!(c, MINUS | PLUS)
    }

    fn is_digit(search_character: u8) -> bool {
        let result = DIGITS.iter().find(|digit| **digit == search_character);
        result.is_some()
    }

    fn digit_to_integer(digit: u8) -> Option<usize> {
        DIGITS.iter().position(|c| *c == digit)
    }

    fn out_of_range(is_positive: bool) -> i32 {
        if is_positive {
            i32::MAX
        } else {
            i32::MIN
        }
    }

    fn first_non_digit_index(bytes: &[u8], begin_index: usize) -> Option<usize> {
        for p in bytes.iter().skip(begin_index).enumerate() {
            if !is_digit(*p.1) {
                // enumerate always starts at zero.
                return Some(p.0 + begin_index);
            }
        }

        None
    }

    fn first_non_zero_digit_index(bytes: &[u8], begin_index: usize) -> Option<usize> {
        for p in bytes.iter().skip(begin_index).enumerate() {
            let c = *p.1;
            if is_digit(c) {
                let zero_digit = DIGITS[0];
                if zero_digit != c {
                    // enumerate always starts at zero.
                    return Some(p.0 + begin_index);
                }
            } else {
                return None;
            }
        }

        None
    }

    my_atoi(s)
}

pub fn atoi_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("atoi benchmark");

    for input in &["1", "12345", "123456789", "29291948", "292389411"] {
        let cstr = CString::new(input.as_bytes()).unwrap();

        group.bench_with_input(BenchmarkId::new("simd_atoi", input), input, |b, input| {
            b.iter(|| black_box(atoi::atoi(black_box(input))))
        });

        group.bench_with_input(BenchmarkId::new("parse_atoi", input), input, |b, input| {
            b.iter(|| black_box(black_box(input).parse::<u32>().unwrap()))
        });

        group.bench_with_input(BenchmarkId::new("c_atoi", input), &cstr, |b, input| {
            b.iter(|| black_box(unsafe { atoi(black_box(input).as_ptr()) }))
        });

        group.bench_with_input(
            BenchmarkId::new("leetcode_0ms_atoi", input),
            input,
            |b, input| b.iter(|| black_box(leetcode_0ms_atoi(black_box(input)))),
        );

        group.bench_with_input(
            BenchmarkId::new("leetcode_3ms_atoi", input),
            input,
            |b, input| b.iter(|| black_box(leetcode_3ms_atoi(black_box(input)))),
        );

        group.bench_with_input(
            BenchmarkId::new("leetcode_8ms_atoi", input),
            input,
            |b, input| b.iter(|| black_box(leetcode_8ms_atoi(black_box(input)))),
        );
    }
}

criterion_group!(benches, atoi_benchmark);
criterion_main!(benches);
